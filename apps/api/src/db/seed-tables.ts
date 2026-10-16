/**
 * What this app seeds: the tables a snapshot holds, parents before children, and what a fresh
 * clone gets when there is no snapshot.
 */

import { fileURLToPath } from 'url';
import type { PgTable } from 'drizzle-orm/pg-core';
import { notes } from './schema';
import type { SeedTable } from './seed-core';
import type { DatabaseConnection } from './types';

/** Where snapshots are kept: one timestamp-named folder each. Not committed. */
export const SEED_DATA_ROOT = fileURLToPath(new URL('seed-data', import.meta.url));

/** The tables a snapshot holds, parents before children. */
export const SEED_TABLES: readonly SeedTable[] = [{ name: 'notes', table: notes }];

/** Tables no snapshot holds that point at seeded rows, so they are emptied too. */
export const ALSO_CLEAR: readonly PgTable[] = [];

/**
 * What a fresh clone is seeded with, once the tables are emptied: enough to see every page with
 * something on it.
 * @param db - The database being seeded
 */
export async function seedFallback(db: DatabaseConnection): Promise<void> {
	await db.insert(notes).values([
		{
			title: 'Read docs/removing-notes.md',
			body: 'It lists every file the example takes, and what to change.',
		},
		{ title: 'Replace the notes', body: 'The example resource leaves in one commit.' },
	]);
}
