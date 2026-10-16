import { createQueryCounter } from '@jswob/drizzle-utils';
import { drizzle } from 'drizzle-orm/postgres-js';
import type { PostgresJsDatabase } from 'drizzle-orm/postgres-js';
import postgres from 'postgres';
import * as schema from '../db/schema';
import { dbTarget } from '../db/target';

let connection: postgres.Sql | null = null;
let db: PostgresJsDatabase<typeof schema> | null = null;
const counter = createQueryCounter();

export function getTestDb(): PostgresJsDatabase<typeof schema> {
	if (!db) {
		connection = postgres(dbTarget('test').url, {
			max: 5,
			idle_timeout: 60,
			ssl: false,
			debug: counter.debug,
		});
		db = drizzle(connection, { schema });
	}
	return db;
}

export async function closeTestDb(): Promise<void> {
	if (connection) {
		await connection.end({ timeout: 5 });
		connection = null;
		db = null;
	}
}

/**
 * Count the statements `run` sends to the test database (see `createQueryCounter`):
 *
 * ```ts
 * const queries = await countQueries(() => getMeetingsEnriched(tx));
 * expect(queries).toBe(7);
 * ```
 */
export const countQueries = counter.countQueries;
