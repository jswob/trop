/**
 * Utilities for extracting metadata from Drizzle tables.
 */
import { getTableName as drizzleGetTableName, getTableColumns } from 'drizzle-orm';
import type { AnyPgTable } from 'drizzle-orm/pg-core';

/**
 * Get the table name from a Drizzle table definition.
 */
export function getTableName(table: AnyPgTable): string {
	return drizzleGetTableName(table);
}

/**
 * Get the primary key column name from a Drizzle table definition.
 *
 * Returns undefined if no primary key is found.
 * Note: This only detects single-column primary keys defined with .primaryKey().
 */
export function getPrimaryKeyColumn(table: AnyPgTable): string | undefined {
	const columns = getTableColumns(table);
	for (const [name, column] of Object.entries(columns)) {
		const pgColumn = column;
		if (pgColumn.primary) {
			return name;
		}
	}
	return undefined;
}
