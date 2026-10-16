import type { SQL } from 'drizzle-orm';
import { sql } from 'drizzle-orm';
import type { AnyPgColumn } from 'drizzle-orm/pg-core';

/**
 * A `CASE` expression that gives each row its own value, for updating many rows to different
 * values in **one** statement instead of one `UPDATE` per row:
 *
 * ```ts
 * await db
 * 	.update(items)
 * 	.set({ position: caseBy(items.id, items.position, inputs.map(i => [i.id, i.position])) })
 * 	.where(inArray(items.id, inputs.map(i => i.id)));
 * ```
 *
 * Each value is cast to the target column's type, since Postgres reads an untyped `CASE` of
 * parameters as text. The `WHERE` must keep the update to the keys listed: a row whose key is
 * missing from `pairs` would be set to null.
 *
 * @param keyColumn - The column that tells the rows apart (usually the primary key)
 * @param valueColumn - The column being set; its SQL type is the cast
 * @param pairs - `[key, value]` per row; must not be empty
 */
export function caseBy<
	TKey extends string | number,
	TValue extends string | number | boolean | null,
>(
	keyColumn: AnyPgColumn,
	valueColumn: AnyPgColumn,
	pairs: readonly (readonly [TKey, TValue])[]
): SQL {
	if (pairs.length === 0) throw new Error('caseBy needs at least one pair');

	const type = sql.raw(valueColumn.getSQLType());
	const branches = pairs.map(
		([key, value]) => sql`when ${keyColumn} = ${key} then ${value}::${type}`
	);
	return sql`(case ${sql.join(branches, sql.raw(' '))} end)`;
}
