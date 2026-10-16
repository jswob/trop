import { sql } from 'drizzle-orm';

/**
 * The database's clock, for a timestamp written on update. A column's `defaultNow()` is evaluated
 * by Postgres; setting `new Date()` on update mixes in the host's clock, and the two drift —
 * containerised databases routinely run tens of milliseconds apart from their host. When the
 * database's clock leads, a freshly updated row reports an `updatedAt` before its own `createdAt`.
 */
export function dbNow() {
	return sql`now()`;
}
