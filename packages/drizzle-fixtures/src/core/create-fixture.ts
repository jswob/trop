/**
 * createFixture - Factory function for creating fixture functions.
 *
 * This is the main entry point for defining fixtures. It returns a function
 * that accepts a database connection and returns a FixtureBuilder.
 *
 * @example
 * ```typescript
 * const userFixture = createFixture({
 *   table: users,
 *   fields: {
 *     name: ({ sequence }) => `User ${sequence}`,
 *     email: ({ sequence }) => `user${sequence}@example.com`,
 *   },
 * });
 *
 * // Usage:
 * const user = await userFixture(db).build();
 * ```
 */
import { getTableName } from 'drizzle-orm';
import type { AnyPgTable } from 'drizzle-orm/pg-core';

import type { FixtureConfig, FixtureFunction, DrizzleDatabase, TraitAugmentations } from '../types';
import type { FixtureInternalOptions } from '../types/internal';
import { ValidationError } from '../utils/errors';
import { FixtureBuilder } from './fixture-builder';
import { SequenceCounter } from './sequence';

/**
 * Creates a fixture function from a configuration object.
 *
 * @param config - The fixture configuration including table, fields, traits, and hooks
 * @returns A function that accepts a database connection and returns a FixtureBuilder
 *
 * @throws ValidationError if the table is not a valid Drizzle table object
 */
export function createFixture<
	TTable extends AnyPgTable,
	// eslint-disable-next-line @typescript-eslint/no-empty-object-type -- Reason: Generic defaults for optional traits/augmentations
	TTraits extends Record<string, unknown> = {},
	// eslint-disable-next-line @typescript-eslint/no-empty-object-type -- Reason: Generic defaults for optional traits/augmentations
	TAugmentations extends TraitAugmentations<TTraits> = {},
>(
	config: FixtureConfig<TTable, TTraits, TAugmentations>
): FixtureFunction<TTable, TTraits, TAugmentations> {
	if (!config.table) {
		throw new ValidationError('table', 'Must be a valid Drizzle table object');
	}

	let tableName: string;
	try {
		tableName = getTableName(config.table);
		if (typeof tableName !== 'string' || tableName.length === 0) {
			throw new ValidationError('table', 'Must be a valid Drizzle table object');
		}
	} catch (error) {
		if (error instanceof ValidationError) {
			throw error;
		}
		throw new ValidationError('table', 'Must be a valid Drizzle table object');
	}

	const fixture = (
		db: DrizzleDatabase,
		internalOptions?: FixtureInternalOptions
	): ReturnType<FixtureFunction<TTable, TTraits, TAugmentations>> => {
		if (!db) {
			throw new ValidationError('db', 'Database connection is required');
		}

		const sequence = internalOptions?.sequence ?? new SequenceCounter();
		const resolutionStack = internalOptions?.resolutionStack ?? [];

		return new FixtureBuilder<TTable, TTraits, TAugmentations, readonly []>(
			config,
			db,
			sequence,
			[],
			new Map(),
			resolutionStack
		);
	};

	Object.defineProperty(fixture, '_tableName', {
		value: tableName,
		writable: false,
		enumerable: false,
	});

	return fixture as FixtureFunction<TTable, TTraits, TAugmentations>;
}
