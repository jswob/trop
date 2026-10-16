import type { PgDatabase } from 'drizzle-orm/pg-core';

/** Abstracted database connection type that accepts both the main db and test transactions. */
// eslint-disable-next-line @typescript-eslint/no-explicit-any -- Reason: PgDatabase<any, any> is the common base type for both PostgresJsDatabase and PgTransaction, required for DI testability
export type DatabaseConnection = PgDatabase<any, any>;
