/**
 * `toolkit patch`: a fix to an installed package, made in the app that found the bug and shipped
 * with it the same day, as Bun's own patch (`patches/`, `patchedDependencies`).
 *
 * `bun patch <name>@<version>` makes an editable copy at the root's `node_modules/<name>`, with
 * either linker. The marker file records what is being patched, so a guard can let an edit through
 * there and `check` can say a patch was left half made. Committing the patch first adopts the edits
 * made to the package's copies in `.claude/` (an agent edits the skill it reads), then syncs the
 * copies from the patched package, so `check` holds. `toolkit upstream` replays the patch into the
 * package's home.
 *
 * A patch is made for one version, and Bun drops it in silence once another is installed. `carry`
 * takes each one to the version installed: deleted when that version has the fix, made again when
 * it still applies, and left for a hand when it does not.
 */
import { spawnSync } from 'node:child_process';
import {
	existsSync,
	mkdirSync,
	mkdtempSync,
	readFileSync,
	realpathSync,
	rmSync,
	writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

import { LOCK_FILE } from './assets';
import { copyOf, readLock, sourcePaths, sync, type Synced } from './claude';
import { installedCopies, readManifest, rootManifest } from './repo';

/** Runs a command in a folder; a failure throws. */
export type Run = (command: string, args: string[], cwd: string) => void;

/** What is being patched: each package's version and its editable copy, from the repo's root. */
export type Patching = Record<string, { version: string; dir: string }>;

/** Where the marker is kept, under the repo. Inside `node_modules`, so it is never committed. */
export const MARKER_FILE = 'node_modules/.toolkit/patching.json';

export const runCommand: Run = (command, args, cwd) => {
	const result = spawnSync(command, args, { cwd, stdio: 'inherit' });
	if (result.status !== 0) {
		throw new Error(`${command} ${args.join(' ')} failed (exit ${String(result.status)})`);
	}
};

export function readPatching(repo: string): Patching {
	const file = path.join(repo, MARKER_FILE);
	return existsSync(file) ? (JSON.parse(readFileSync(file, 'utf8')) as Patching) : {};
}

function writePatching(repo: string, patching: Patching): void {
	const file = path.join(repo, MARKER_FILE);
	if (Object.keys(patching).length === 0) {
		rmSync(file, { force: true });
		return;
	}
	mkdirSync(path.dirname(file), { recursive: true });
	writeFileSync(file, `${JSON.stringify(patching, null, '\t')}\n`, 'utf8');
}

/** A `patchedDependencies` key, `<name>@<version>`, split; a scoped name keeps its leading `@`. */
export function splitKey(key: string): { name: string; version: string } {
	const at = key.lastIndexOf('@');
	if (at <= 0) throw new Error(`${key} is not <name>@<version>`);
	return { name: key.slice(0, at), version: key.slice(at + 1) };
}

/** The one installed version of `name` that can be patched. */
function patchableVersion(repo: string, name: string): string {
	const copies = installedCopies(repo, name);
	if (copies.length === 0) throw new Error(`${name} is not installed`);

	const own = copies.find(copy => copy.workspace);
	if (own !== undefined) {
		throw new Error(
			`${name} is this repo's own source at ${path.relative(realpathSync(repo), own.root)}, or linked to it: edit it there`
		);
	}

	const versions = [...new Set(copies.map(copy => copy.version))];
	if (versions.length > 1) {
		throw new Error(`several versions of ${name} are installed (${versions.join(', ')})`);
	}
	return versions[0] ?? '0.0.0';
}

/**
 * Make the installed copy of a package editable.
 * @param repo - The repo's root
 * @param name - The package
 * @param run - Runs `bun patch`
 */
export function startPatch(
	repo: string,
	name: string,
	run: Run = runCommand
): { version: string; dir: string; started: boolean } {
	const patching = readPatching(repo);
	const current = patching[name];
	if (current !== undefined) return { ...current, started: false };

	const version = patchableVersion(repo, name);
	run('bun', ['patch', `${name}@${version}`], repo);

	const entry = { version, dir: `node_modules/${name}` };
	writePatching(repo, { ...patching, [name]: entry });
	return { ...entry, started: true };
}

/**
 * Write each copy in `.claude/` that was edited since the last sync into the editable package, at
 * the file it was copied from. Copies synced at another version are not edits, and are refused.
 */
function adoptCopies(repo: string, name: string, entry: Patching[string]): string[] {
	const root = path.join(repo, entry.dir);
	const claude = readManifest(root)?.claude;
	if (claude === undefined) return [];

	const sources = sourcePaths({ name, root, claude });
	const copies = copyOf(repo, Object.keys(sources));
	const edited = Object.entries(sources).flatMap(([key, file]) => {
		const copy = copies[key];
		return copy !== undefined && copy !== readFileSync(file, 'utf8') ? [{ key, file, copy }] : [];
	});
	if (edited.length === 0) return [];

	const synced = readLock(repo).packages[name]?.version;
	if (synced !== entry.version) {
		throw new Error(
			`the copies of ${name} in .claude/ were synced at ${synced ?? 'no version'}, not at ${entry.version}, ` +
				`so they are not edits of it (${LOCK_FILE})`
		);
	}

	for (const { file, copy } of edited) writeFileSync(file, copy, 'utf8');
	return edited.map(({ key }) => key).sort();
}

/**
 * Write the patch: the edits made in the editable copy, and in the package's copies in `.claude/`.
 * @param repo - The repo's root
 * @param name - The package being patched
 * @param run - Runs `bun patch --commit`
 * @param adopt - Whether the copies' edits go into the patch; `carry` makes a patch again on another
 *   version, and the copies are the old version's
 * @returns The patch file (`null` when nothing changed), the copies it took, and the sync after it
 */
export function commitPatch(
	repo: string,
	name: string,
	run: Run = runCommand,
	adopt = true
): { patch: string | null; adopted: string[]; synced: Synced[] } {
	const patching = readPatching(repo);
	const entry = patching[name];
	if (entry === undefined)
		throw new Error(`${name} is not being patched: bun toolkit patch ${name}`);

	const adopted = adopt ? adoptCopies(repo, name, entry) : [];
	run('bun', ['patch', '--commit', entry.dir], repo);
	reinstall(repo, name, patching, run);

	const patch = rootManifest(repo).patchedDependencies?.[`${name}@${entry.version}`] ?? null;
	return { patch, adopted, synced: sync(repo, [name]).synced };
}

/**
 * The editable copy goes, and Bun installs the package again, with the patch if there is one.
 *
 * Under the isolated linker `bun patch` leaves the editable copy where the root's link to Bun's
 * store was, as a plain folder, and no later install replaces it: after a bump the root would still
 * resolve the old version.
 */
function reinstall(repo: string, name: string, patching: Patching, run: Run): void {
	const { [name]: entry, ...rest } = patching;
	if (entry === undefined) return;

	rmSync(path.join(repo, entry.dir), { recursive: true, force: true });
	run('bun', ['install'], repo);
	writePatching(repo, rest);
}

/**
 * Throw the edits away: the editable copy goes, and Bun installs the package again.
 * @param repo - The repo's root
 * @param name - The package being patched
 * @param run - Runs `bun install`
 */
export function abortPatch(repo: string, name: string, run: Run = runCommand): void {
	const patching = readPatching(repo);
	if (patching[name] === undefined) throw new Error(`${name} is not being patched`);

	reinstall(repo, name, patching, run);
}

/**
 * Where a patch stands against what is installed.
 * - `current`: made for the installed version, so Bun applies it
 * - `linked`: the package runs on its source, without the patch; `check` names the link
 * - `missing`: the package is not installed
 * - `several`: several versions are installed, and none is the patch's
 * - `fixed`: the installed version has the fix
 * - `applies`: it does not, and the patch applies to it
 * - `conflicts`: it does not, and the patch does not apply to it
 */
export type Standing =
	'current' | 'linked' | 'missing' | 'several' | 'fixed' | 'applies' | 'conflicts';

/** One patch: its key split, its file, the versions installed, and where it stands. */
export type PatchStanding = {
	name: string;
	version: string;
	file: string;
	installed: string[];
	standing: Standing;
};

/** Whether git could apply a patch to an installed copy, or take it back out; nothing changes. */
function applies(repo: string, file: string, root: string, reverse: boolean): boolean {
	const real = realpathSync(repo);
	const args = ['apply', '--check', `--directory=${path.relative(real, root)}`];
	const result = spawnSync('git', [...args, ...(reverse ? ['-R'] : []), file], {
		cwd: real,
		stdio: 'ignore',
	});
	return result.status === 0;
}

/**
 * Where each of the repo's patches stands. A patch whose version is no longer installed is held
 * against the version that is: when git can take it back out, that version has the fix.
 * @param repo - The repo's root
 */
export function patchStandings(repo: string): PatchStanding[] {
	const patched = rootManifest(repo).patchedDependencies ?? {};

	return Object.entries(patched).map(([key, file]) => {
		const { name, version } = splitKey(key);
		const copies = installedCopies(repo, name);
		const installed = [...new Set(copies.map(copy => copy.version))];
		const at = (standing: Standing): PatchStanding => ({
			name,
			version,
			file,
			installed,
			standing,
		});

		const copy = copies[0];
		if (copies.some(each => each.workspace)) return at('linked');
		if (copy === undefined) return at('missing');
		if (installed.includes(version)) return at('current');
		if (installed.length > 1) return at('several');
		if (applies(repo, file, copy.root, true)) return at('fixed');
		return at(applies(repo, file, copy.root, false) ? 'applies' : 'conflicts');
	});
}

/** What is wrong with a patch, or `null` when nothing is. */
function standingProblem(patch: PatchStanding): string | null {
	const { name, version, file, installed } = patch;
	const made = `${file} was made for ${name}@${version}, and Bun drops it on ${installed.join(', ')}`;
	switch (patch.standing) {
		case 'current':
		case 'linked':
			return null;
		case 'missing':
			return `${file}: ${name} is not installed`;
		case 'several':
			return `${made}: make it again for one of them (bun toolkit patch ${name})`;
		case 'fixed':
			return `${made}, which has the fix: bun toolkit carry deletes it`;
		case 'applies':
			return `${made}, which it still applies to: bun toolkit carry makes it again`;
		case 'conflicts':
			return `${made}, which it does not apply to: make it again by hand (bun toolkit patch ${name})`;
	}
}

/** Why the patches made for another version than the one installed are not in order. */
export function staleProblems(repo: string): string[] {
	return patchStandings(repo).flatMap(patch => standingProblem(patch) ?? []);
}

/**
 * Why the repo's patches are not in order. Empty when they are.
 *
 * A patch made for a version that is no longer installed is dropped by Bun in silence, and nothing
 * else removes its key: the new version either carries the fix, and the patch is deleted, or it
 * does not, and the patch is made again on it.
 * @param repo - The repo's root
 */
export function patchProblems(repo: string): string[] {
	return [
		...staleProblems(repo),
		...Object.keys(readPatching(repo)).map(
			name => `${name} is being patched: bun toolkit patch ${name} --commit, or --abort`
		),
	];
}

/** The patches removed from `patchedDependencies` and from disk. */
function forget(repo: string, patches: PatchStanding[]): void {
	const file = path.join(repo, 'package.json');
	const text = readFileSync(file, 'utf8');
	const manifest = JSON.parse(text) as Record<string, unknown> & {
		patchedDependencies?: Record<string, string>;
	};
	const patched = { ...manifest.patchedDependencies };
	for (const { name, version, file: patch } of patches) {
		delete patched[`${name}@${version}`];
		rmSync(path.join(repo, patch), { force: true });
	}
	if (Object.keys(patched).length === 0) delete manifest.patchedDependencies;
	else manifest.patchedDependencies = patched;

	const indent = /^[ \t]+/m.exec(text)?.[0] ?? '\t';
	writeFileSync(file, `${JSON.stringify(manifest, null, indent)}\n`, 'utf8');
}

/** What `carry` did to each patch Bun drops. */
export type Carried = {
	dropped: PatchStanding[];
	carried: { from: PatchStanding; patch: string | null }[];
	stuck: PatchStanding[];
};

/**
 * Take each patch to the version installed: a patch that version has the fix of is deleted, and one
 * that still applies to it is made again on it. The rest are left, and returned as stuck.
 * @param repo - The repo's root
 * @param run - Runs `bun patch` and `bun install`
 */
export function carry(repo: string, run: Run = runCommand): Carried {
	const making = Object.keys(readPatching(repo));
	if (making.length > 0) {
		throw new Error(`${making.join(', ')} is being patched: --commit or --abort it first`);
	}

	const all = patchStandings(repo);
	const dropped = all.filter(patch => patch.standing === 'fixed');
	const moving = all.filter(patch => patch.standing === 'applies');
	const stuck = all.filter(
		({ standing }) => standing === 'missing' || standing === 'several' || standing === 'conflicts'
	);
	if (dropped.length + moving.length === 0) return { dropped, carried: [], stuck };

	const scratch = mkdtempSync(path.join(tmpdir(), 'toolkit-carry-'));
	try {
		const kept = moving.map((patch, index) => {
			const file = path.join(scratch, `${String(index)}.patch`);
			writeFileSync(file, readFileSync(path.join(repo, patch.file)));
			return { patch, file };
		});
		forget(repo, [...dropped, ...moving]);
		if (moving.length === 0) run('bun', ['install'], repo);

		const carried = kept.map(({ patch, file }) => {
			const { dir } = startPatch(repo, patch.name, run);
			const result = spawnSync('git', ['apply', `--directory=${dir}`, file], {
				cwd: realpathSync(repo),
				encoding: 'utf8',
			});
			if (result.status !== 0) {
				abortPatch(repo, patch.name, run);
				throw new Error(
					`${patch.file} did not apply to ${patch.name}@${patch.installed.join(', ')} after all ` +
						`(git checkout -- package.json '${patch.file}' brings it back): ${result.stderr.trim()}`
				);
			}
			return { from: patch, patch: commitPatch(repo, patch.name, run, false).patch };
		});
		return { dropped, carried, stuck };
	} finally {
		rmSync(scratch, { recursive: true, force: true });
	}
}
