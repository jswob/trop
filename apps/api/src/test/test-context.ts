import { TransactionRollbackError } from 'drizzle-orm';
import type { ExtractTablesWithRelations } from 'drizzle-orm';
import type { PgQueryResultHKT, PgTransaction } from 'drizzle-orm/pg-core';
import type { PostgresJsDatabase } from 'drizzle-orm/postgres-js';
import { test as baseTest } from 'vitest';

import type * as schema from '../db/schema';
import { getTestDb } from './db';
import { factory as baseFactory } from './fixtures';

type TestTransaction = PgTransaction<
	PgQueryResultHKT,
	typeof schema,
	ExtractTablesWithRelations<typeof schema>
>;
type BoundFactory = ReturnType<typeof baseFactory>;

interface TestFixtures {
	tx: TestTransaction;
	factory: BoundFactory;
	db: PostgresJsDatabase<typeof schema>;
}

export const test = baseTest.extend<TestFixtures>({
	// eslint-disable-next-line no-empty-pattern -- Reason: Vitest fixture API requires object destructuring even when not using dependencies
	db: async ({}, use) => {
		await use(getTestDb());
	},

	// eslint-disable-next-line no-empty-pattern -- Reason: Vitest fixture API requires object destructuring even when not using dependencies
	tx: async ({}, use) => {
		const db = getTestDb();
		try {
			await db.transaction(async tx => {
				await use(tx);
				tx.rollback();
			});
		} catch (error) {
			if (!(error instanceof TransactionRollbackError)) {
				throw error;
			}
		}
	},

	factory: async ({ tx }, use) => {
		const session = baseFactory(tx);
		await use(session);
	},
});

export { expect, describe, beforeAll, afterAll, beforeEach, afterEach } from 'vitest';
