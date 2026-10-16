/**
 * Seeding from a snapshot of real data. A snapshot is a folder named by its timestamp, holding one
 * `<table>.json` per table, each row as exported (snake_case columns). The newest one is loaded into
 * the tables the app lists, parents first, after they are emptied children first. Nothing here
 * knows the app's tables: `seed-tables.ts` lists them.
 */

import { readFileSync, readdirSync } from 'fs';
import { join } from 'path';
import type { PgTable } from 'drizzle-orm/pg-core';
import type { DatabaseConnection } from './types';

export type SeedRow = Record<string, unknown>;

export interface SeedTable {
	/** The table's name in the database, and its snapshot file's: `<name>.json`. */
	name: string;
	table: PgTable;
	/** Orders or adjusts the rows, camelCased, before they are inserted. */
	prepare?: (rows: SeedRow[]) => SeedRow[];
	/** Inserts the rows exactly as exported, in place of the chunked insert. */
	insertRaw?: (db: DatabaseConnection, rows: SeedRow[]) => Promise<void>;
}

export interface SeedOptions {
	/** The folder holding one timestamp-named folder per snapshot. */
	root: string;
	/** The tables a snapshot fills, parents before children. */
	tables: readonly SeedTable[];
	/** Tables no snapshot holds that are emptied too, before the others. */
	alsoClear?: readonly PgTable[];
	/**
	 * What to seed when there is no snapshot (a fresh clone), once the tables are emptied. Without
	 * one, the database is left as it is.
	 */
	fallback?: (db: DatabaseConnection) => Promise<void>;
	/** Where progress is written; `console.log` by default. */
	log?: (line: string) => void;
}

export type SeedOutcome = { from: 'snapshot'; name: string } | { from: 'fallback' | 'nothing' };

const CHUNK_SIZE = 50;
const TIMESTAMP_KEYS = new Set(['created_at', 'updated_at']);

/** The snapshots under `root`, newest first; none when the folder does not exist. */
export function snapshotFolders(root: string): string[] {
	let entries: string[];
	try {
		entries = readdirSync(root);
	} catch {
		return [];
	}
	return entries.filter(name => /^\d+$/.test(name)).sort((a, b) => Number(b) - Number(a));
}

/** A snapshot row as Drizzle takes it: camelCase keys, `created_at` and `updated_at` as dates. */
export function toInsertRow(row: SeedRow): SeedRow {
	const mapped: SeedRow = {};
	for (const [key, value] of Object.entries(row)) {
		const camelKey = key.replace(/_([a-z])/g, (_match, c: string) => c.toUpperCase());
		mapped[camelKey] =
			TIMESTAMP_KEYS.has(key) && typeof value === 'string' ? new Date(value) : value;
	}
	return mapped;
}

function readRows(snapshot: string, name: string): SeedRow[] {
	return JSON.parse(readFileSync(join(snapshot, `${name}.json`), 'utf8')) as SeedRow[];
}

async function chunkInsert(db: DatabaseConnection, table: PgTable, rows: SeedRow[]) {
	for (let i = 0; i < rows.length; i += CHUNK_SIZE) {
		const chunk = rows.slice(i, i + CHUNK_SIZE);
		await db
			.insert(table)

			.values(chunk)
			.onConflictDoNothing();
	}
}

async function clear(db: DatabaseConnection, options: SeedOptions) {
	const tables = [...(options.alsoClear ?? []), ...options.tables.map(t => t.table).reverse()];
	for (const table of tables) {
		await db.delete(table);
	}
}

/**
 * Seed `db` from the newest snapshot under `root`, or with `fallback` when there is none. Every
 * listed table is emptied first, so this is never pointed at a database whose rows matter.
 * @param db - The database to fill
 * @param options - The snapshots' folder, the tables, and what to do without a snapshot
 */
export async function seedDatabase(
	db: DatabaseConnection,
	options: SeedOptions
): Promise<SeedOutcome> {
	const log = options.log ?? console.log;
	const [latest] = snapshotFolders(options.root);

	if (latest === undefined) {
		if (!options.fallback) {
			log(`No snapshot in ${options.root}: nothing seeded, the database is left as it is.`);
			return { from: 'nothing' };
		}
		log(`No snapshot in ${options.root}: seeding the fallback data.`);
		await clear(db, options);
		await options.fallback(db);
		return { from: 'fallback' };
	}

	const snapshot = join(options.root, latest);
	log(`Seeding from snapshot ${latest}...`);
	await clear(db, options);

	for (const { name, table, prepare, insertRaw } of options.tables) {
		const raw = readRows(snapshot, name);
		if (insertRaw) {
			await insertRaw(db, raw);
		} else {
			const rows = raw.map(toInsertRow);
			await chunkInsert(db, table, prepare ? prepare(rows) : rows);
		}
		log(`  ${name}: ${raw.length} records`);
	}

	return { from: 'snapshot', name: latest };
}
