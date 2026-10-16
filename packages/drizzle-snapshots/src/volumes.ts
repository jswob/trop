/**
 * Docker volume and container names for one compose project's databases. Every name starts with
 * the project, so two checkouts on one machine never list, prune, restore or delete each other's
 * snapshots. Before this, every project named its snapshots `postgres_snapshot_*`: one project's
 * `db:base` replaced another's, a bare `db:restore` could load another project's data, and pruning
 * kept the newest four across all of them.
 */

import type { DbTargetName } from './config';

type Env = Record<string, string | undefined>;

export interface ProjectVolumes {
	/** The dev database's data volume, created by compose. */
	main: string;
	/** The e2e database's data volume, created by compose. */
	e2e: string;
	/** Start of every snapshot of the dev database, the base included. */
	snapshotPrefix: string;
	/** The protected baseline: never pruned, replaced only by `db:base`. */
	base: string;
	/** Start of the safety copies a restore makes of the dev database. */
	backupPrefix: string;
	/** The e2e baseline, restored before every e2e test. */
	e2eBase: string;
}

export interface Snapshot {
	volume: string;
	/** What the user types: `base`, `before_push_1790711757`, a hand-given name. */
	name: string;
	isBase: boolean;
	/** Unix seconds from the name, or null for a hand-named snapshot. */
	createdAt: number | null;
}

/**
 * The compose project, which every volume and container name starts with.
 * @param env - Environment to read, `process.env` by default
 */
export function composeProject(env: Env = process.env): string {
	const project = env.COMPOSE_PROJECT_NAME;
	if (!project) {
		throw new Error('COMPOSE_PROJECT_NAME is not set; add it to the root .env');
	}
	return project;
}

export function projectVolumes(project: string): ProjectVolumes {
	return {
		main: `${project}_postgres_data`,
		e2e: `${project}_postgres_e2e_data`,
		snapshotPrefix: `${project}_postgres_snapshot_`,
		base: `${project}_postgres_snapshot_base`,
		backupPrefix: `${project}_postgres_data_backup_`,
		e2eBase: `${project}_e2e_snapshot_base`,
	};
}

/**
 * The compose container that serves a database, as `docker-compose.yml` names it.
 * @param target - Which database
 * @param env - Environment to read, `process.env` by default
 */
export function containerFor(target: DbTargetName, env: Env = process.env): string {
	const project = composeProject(env);
	if (target === 'test') return `${project}-postgres-test`;
	if (target === 'e2e') return `${project}-postgres-e2e`;
	return `${project}-${env.POSTGRES_CONTAINER_NAME || 'postgres'}`;
}

/**
 * A name every project shared before snapshots carried the project. Which project made one cannot
 * be told from the name, so nothing here prunes or deletes them; a restore takes one only when its
 * full name is typed.
 */
export function isLegacySnapshot(volume: string): boolean {
	return volume.startsWith('postgres_snapshot_') || volume === 'e2e_snapshot_base';
}

function timestampOf(name: string): number | null {
	const match = /(?:^|_)(\d{9,})$/.exec(name);
	return match ? Number(match[1]) : null;
}

/**
 * This project's snapshots: the base first, then the newest first, hand-named ones last.
 * @param volumes - Every volume on the machine
 * @param names - This project's names
 */
export function projectSnapshots(volumes: readonly string[], names: ProjectVolumes): Snapshot[] {
	const snapshots = volumes
		.filter(volume => volume.startsWith(names.snapshotPrefix))
		.map(volume => {
			const name = volume.slice(names.snapshotPrefix.length);
			const isBase = volume === names.base;
			return { volume, name, isBase, createdAt: isBase ? null : timestampOf(name) };
		});
	return snapshots.sort((a, b) => {
		if (a.isBase !== b.isBase) return a.isBase ? -1 : 1;
		if (a.createdAt !== null && b.createdAt !== null) return b.createdAt - a.createdAt;
		if (a.createdAt !== null) return -1;
		if (b.createdAt !== null) return 1;
		return a.name.localeCompare(b.name);
	});
}

/**
 * The timestamped snapshots beyond the newest `keep`. The base and hand-named snapshots are never
 * pruned: someone named them on purpose.
 * @param snapshots - This project's snapshots
 * @param keep - How many timestamped snapshots survive
 */
export function snapshotsToPrune(snapshots: readonly Snapshot[], keep: number): Snapshot[] {
	return snapshots
		.filter(snapshot => !snapshot.isBase && snapshot.createdAt !== null)
		.sort((a, b) => (b.createdAt ?? 0) - (a.createdAt ?? 0))
		.slice(keep);
}

/**
 * The snapshot a typed name means: `base`, a snapshot's name, a bare timestamp, `snapshot_<ts>`
 * (how the list printed them before), or a full volume name. A pre-project volume is found only by
 * its full name.
 * @param query - What the user typed
 * @param volumes - Every volume on the machine
 * @param names - This project's names
 */
export function findSnapshot(
	query: string,
	volumes: readonly string[],
	names: ProjectVolumes
): Snapshot | null {
	const snapshots = projectSnapshots(volumes, names);
	const bare = query.replace(/^snapshot_/, '');
	const found =
		snapshots.find(s => s.volume === query || s.name === query) ??
		snapshots.find(s => s.name === bare) ??
		snapshots.find(s => s.createdAt !== null && String(s.createdAt) === bare);
	if (found) return found;

	if (isLegacySnapshot(query) && volumes.includes(query)) {
		return { volume: query, name: query, isBase: false, createdAt: timestampOf(query) };
	}
	return null;
}

/**
 * This project's safety copies, newest first.
 * @param volumes - Every volume on the machine
 * @param names - This project's names
 */
export function projectBackups(volumes: readonly string[], names: ProjectVolumes): string[] {
	return volumes
		.filter(volume => volume.startsWith(names.backupPrefix))
		.sort((a, b) => (timestampOf(b) ?? 0) - (timestampOf(a) ?? 0));
}
