import { describe, expect, test } from 'vitest';
import {
	composeProject,
	containerFor,
	findSnapshot,
	isLegacySnapshot,
	projectBackups,
	projectSnapshots,
	projectVolumes,
	snapshotsToPrune,
} from './volumes';

const ours = projectVolumes('alpha');

/** A machine with two projects' volumes and the names every project shared before. */
const MACHINE = [
	'alpha_postgres_data',
	'alpha_postgres_snapshot_base',
	'alpha_postgres_snapshot_before_push_1790000300',
	'alpha_postgres_snapshot_1790000200',
	'alpha_postgres_snapshot_before_migration',
	'alpha_postgres_snapshot_before_push_1790000100',
	'alpha_postgres_data_backup_1790000050',
	'alpha_postgres_data_backup_1790000400',
	'alpha-2_postgres_snapshot_base',
	'alpha-2_postgres_snapshot_before_push_1790000999',
	'beta_postgres_snapshot_base',
	'beta_postgres_snapshot_before_push_1790000500',
	'beta_postgres_data_backup_1790000600',
	'postgres_snapshot_base',
	'postgres_snapshot_before_push_1790000700',
	'e2e_snapshot_base',
];

describe('projectVolumes', () => {
	test('every name starts with the project', () => {
		expect(projectVolumes('alpha')).toEqual({
			main: 'alpha_postgres_data',
			e2e: 'alpha_postgres_e2e_data',
			snapshotPrefix: 'alpha_postgres_snapshot_',
			base: 'alpha_postgres_snapshot_base',
			backupPrefix: 'alpha_postgres_data_backup_',
			e2eBase: 'alpha_e2e_snapshot_base',
		});
	});

	test('a missing project is refused, not guessed', () => {
		expect(() => composeProject({})).toThrow(/COMPOSE_PROJECT_NAME/);
	});

	test('containers are named as compose names them', () => {
		const env = { COMPOSE_PROJECT_NAME: 'alpha', POSTGRES_CONTAINER_NAME: 'postgres' };
		expect(containerFor('app', env)).toBe('alpha-postgres');
		expect(containerFor('test', env)).toBe('alpha-postgres-test');
		expect(containerFor('e2e', env)).toBe('alpha-postgres-e2e');
	});
});

describe('projectSnapshots', () => {
	test('lists only this project: base, newest first, hand-named last', () => {
		expect(projectSnapshots(MACHINE, ours).map(s => s.name)).toEqual([
			'base',
			'before_push_1790000300',
			'1790000200',
			'before_push_1790000100',
			'before_migration',
		]);
	});

	test('a project whose name extends ours is not ours', () => {
		const volumes = projectSnapshots(MACHINE, ours).map(s => s.volume);
		expect(volumes.some(v => v.startsWith('alpha-2_'))).toBe(false);
	});
});

describe('snapshotsToPrune', () => {
	test('prunes only our oldest timestamped snapshots', () => {
		const pruned = snapshotsToPrune(projectSnapshots(MACHINE, ours), 1);
		expect(pruned.map(s => s.volume)).toEqual([
			'alpha_postgres_snapshot_1790000200',
			'alpha_postgres_snapshot_before_push_1790000100',
		]);
	});

	test('never prunes the base or a hand-named snapshot', () => {
		const pruned = snapshotsToPrune(projectSnapshots(MACHINE, ours), 0).map(s => s.name);
		expect(pruned).not.toContain('base');
		expect(pruned).not.toContain('before_migration');
	});
});

describe('findSnapshot', () => {
	test('finds by name, bare timestamp, the old list form and full volume name', () => {
		expect(findSnapshot('base', MACHINE, ours)?.volume).toBe('alpha_postgres_snapshot_base');
		expect(findSnapshot('before_migration', MACHINE, ours)?.volume).toBe(
			'alpha_postgres_snapshot_before_migration'
		);
		expect(findSnapshot('1790000300', MACHINE, ours)?.volume).toBe(
			'alpha_postgres_snapshot_before_push_1790000300'
		);
		expect(findSnapshot('snapshot_1790000200', MACHINE, ours)?.volume).toBe(
			'alpha_postgres_snapshot_1790000200'
		);
		expect(findSnapshot('alpha_postgres_snapshot_base', MACHINE, ours)?.isBase).toBe(true);
	});

	test("never finds another project's snapshot", () => {
		expect(findSnapshot('1790000500', MACHINE, ours)).toBeNull();
		expect(findSnapshot('beta_postgres_snapshot_base', MACHINE, ours)).toBeNull();
	});

	test('a pre-project snapshot is found only by its full name', () => {
		expect(findSnapshot('1790000700', MACHINE, ours)).toBeNull();
		expect(findSnapshot('postgres_snapshot_before_push_1790000700', MACHINE, ours)).toEqual({
			volume: 'postgres_snapshot_before_push_1790000700',
			name: 'postgres_snapshot_before_push_1790000700',
			isBase: false,
			createdAt: 1790000700,
		});
	});
});

describe('isLegacySnapshot', () => {
	test('recognises the names every project shared', () => {
		expect(MACHINE.filter(isLegacySnapshot)).toEqual([
			'postgres_snapshot_base',
			'postgres_snapshot_before_push_1790000700',
			'e2e_snapshot_base',
		]);
	});
});

describe('projectBackups', () => {
	test("lists our safety copies newest first, never another project's", () => {
		expect(projectBackups(MACHINE, ours)).toEqual([
			'alpha_postgres_data_backup_1790000400',
			'alpha_postgres_data_backup_1790000050',
		]);
	});
});
