import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'fs';
import { tmpdir } from 'os';
import { join } from 'path';
import { asc } from 'drizzle-orm';
import { vi } from 'vitest';
import { afterEach, beforeEach, describe, expect, test } from '../test/test-context';
import { notes } from './schema';
import { type SeedTable, seedDatabase, snapshotFolders, toInsertRow } from './seed-core';
import type { DatabaseConnection } from './types';

const TABLES: readonly SeedTable[] = [{ name: 'notes', table: notes }];
const quiet = () => {};

let root: string;

beforeEach(() => {
	root = mkdtempSync(join(tmpdir(), 'seed-core-'));
});

afterEach(() => {
	rmSync(root, { recursive: true, force: true });
});

function writeSnapshot(name: string, rows: Record<string, unknown>[]) {
	mkdirSync(join(root, name));
	writeFileSync(join(root, name, 'notes.json'), JSON.stringify(rows));
}

function exported(id: string, title: string) {
	return {
		id,
		title,
		body: '',
		created_at: '2026-01-02T10:00:00+00:00',
		updated_at: '2026-01-02T10:00:00+00:00',
	};
}

function titlesIn(db: DatabaseConnection) {
	return db
		.select({ title: notes.title })
		.from(notes)
		.orderBy(asc(notes.title))
		.then(rows => rows.map(row => row.title));
}

describe('snapshotFolders', () => {
	test('lists timestamp folders newest first, and none for a missing folder', () => {
		mkdirSync(join(root, '1000'));
		mkdirSync(join(root, '20000'));
		mkdirSync(join(root, 'notes'));

		expect(snapshotFolders(root)).toEqual(['20000', '1000']);
		expect(snapshotFolders(join(root, 'missing'))).toEqual([]);
	});
});

test('toInsertRow camelCases the keys and revives the timestamps', () => {
	expect(toInsertRow({ parent_record_id: 'x', created_at: '2026-01-02T10:00:00Z' })).toEqual({
		parentRecordId: 'x',
		createdAt: new Date('2026-01-02T10:00:00Z'),
	});
});

describe('seedDatabase', () => {
	test('empties the tables and loads the newest snapshot', async ({ tx, factory }) => {
		await factory.note.create({ title: 'Already here' });
		writeSnapshot('1000', [exported('00000000-0000-0000-0000-000000000001', 'Old')]);
		writeSnapshot('2000', [
			exported('00000000-0000-0000-0000-000000000002', 'Ada'),
			exported('00000000-0000-0000-0000-000000000003', 'Grace'),
		]);

		const outcome = await seedDatabase(tx, { root, tables: TABLES, log: quiet });

		expect(outcome).toEqual({ from: 'snapshot', name: '2000' });
		expect(await titlesIn(tx)).toEqual(['Ada', 'Grace']);
	});

	test('hands the rows to a table’s own insert or preparation', async ({ tx }) => {
		writeSnapshot('1000', [
			exported('00000000-0000-0000-0000-000000000002', 'Ada'),
			exported('00000000-0000-0000-0000-000000000003', 'Grace'),
		]);
		const insertRaw = vi.fn(() => Promise.resolve());

		await seedDatabase(tx, {
			root,
			tables: [{ name: 'notes', table: notes, insertRaw }],
			log: quiet,
		});
		await seedDatabase(tx, {
			root,
			tables: [{ name: 'notes', table: notes, prepare: rows => rows.slice(1) }],
			log: quiet,
		});

		expect(insertRaw).toHaveBeenCalledWith(tx, [
			exported('00000000-0000-0000-0000-000000000002', 'Ada'),
			exported('00000000-0000-0000-0000-000000000003', 'Grace'),
		]);
		expect(await titlesIn(tx)).toEqual(['Grace']);
	});

	test('without a snapshot or a fallback, leaves the database as it is', async ({
		tx,
		factory,
	}) => {
		await factory.note.create({ title: 'Already here' });

		const outcome = await seedDatabase(tx, { root, tables: TABLES, log: quiet });

		expect(outcome).toEqual({ from: 'nothing' });
		expect(await titlesIn(tx)).toEqual(['Already here']);
	});

	test('without a snapshot, empties the tables and seeds the fallback', async ({ tx, factory }) => {
		await factory.note.create({ title: 'Already here' });

		const outcome = await seedDatabase(tx, {
			root,
			tables: TABLES,
			fallback: async db => {
				await db.insert(notes).values({ title: 'Sample' });
			},
			log: quiet,
		});

		expect(outcome).toEqual({ from: 'fallback' });
		expect(await titlesIn(tx)).toEqual(['Sample']);
	});
});
