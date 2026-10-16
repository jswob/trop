/**
 * Constraints Drizzle's schema builder cannot express, applied idempotently.
 *
 * `drizzle-kit push` only knows about constraints the schema declares, so an
 * `EXCLUDE` constraint is invisible to it — and a push that recreates the table
 * takes the constraint with it. Rather than leaving that to a remembered manual
 * `psql` step, every path that shapes a database (`push`, `test-setup`)
 * re-applies the app's list afterwards.
 *
 * The app's production migration creates the same constraints; the catalog
 * check makes re-application a no-op there.
 */

import postgres from 'postgres';
import type { RawConstraint } from './config';

/**
 * Create any missing raw constraint on the target database.
 * @param connectionString - Postgres connection string of the target database.
 * @param constraints - The app's raw constraints.
 * @returns The names of the constraints that were created.
 */
export async function applyRawConstraints(
	connectionString: string,
	constraints: readonly RawConstraint[]
): Promise<string[]> {
	if (constraints.length === 0) return [];
	const sql = postgres(connectionString, { max: 1, connect_timeout: 15 });
	const created: string[] = [];

	try {
		for (const constraint of constraints) {
			const existing = await sql`
				SELECT 1 FROM pg_constraint WHERE conname = ${constraint.name}
			`;
			if (existing.length > 0) continue;

			await sql.unsafe(
				`ALTER TABLE ${constraint.table} ADD CONSTRAINT ${constraint.name} ${constraint.definition}`
			);
			created.push(constraint.name);
		}
	} finally {
		await sql.end();
	}

	return created;
}
