/**
 * Factory composition for managing multiple fixtures.
 *
 * Provides a way to compose multiple fixtures into a single factory
 * that can be bound to a database connection (or transaction).
 */
import type { InferSelectModel } from 'drizzle-orm';
import type { AnyPgTable } from 'drizzle-orm/pg-core';
import type {
	DrizzleDatabase,
	FixtureBuilder,
	FixtureFunction,
	TraitAugmentations,
} from '../types';

/**
 * Map of fixture name to fixture function.
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any -- Reason: Generic fixture map must accept any table/traits/augmentations
export type FixturesMap = Record<string, FixtureFunction<AnyPgTable, any, any>>;

/**
 * Helper type to extract the builder type from a fixture function.
 */
type ExtractBuilder<F> =
	F extends FixtureFunction<
		infer TTable extends AnyPgTable,
		infer TTraits extends Record<string, unknown>,
		// eslint-disable-next-line @typescript-eslint/no-explicit-any -- Reason: TraitAugmentations inference requires flexible typing
		infer TAugmentations extends TraitAugmentations<any>
	>
		? FixtureBuilder<InferSelectModel<TTable>, TTraits, TAugmentations, readonly []>
		: never;

/**
 * Bound factory session with fixture access.
 *
 * Each property is a FixtureBuilder for the corresponding fixture.
 * Use with db.transaction() for automatic cleanup via rollback.
 */
export type BoundFactory<T extends FixturesMap> = {
	[K in keyof T]: ExtractBuilder<T[K]>;
};

/**
 * Factory function that creates a bound factory session for a database.
 */
export type ComposedFactory<T extends FixturesMap> = (db: DrizzleDatabase) => BoundFactory<T>;

/**
 * Composes multiple fixtures into a single factory.
 *
 * Each call to factory(db) creates a new session bound to that database connection.
 * For test cleanup, wrap fixture creation in a transaction and rollback.
 *
 * @example
 * ```typescript
 * const factory = composeFactory({
 *   user: userFixture,
 *   post: postFixture,
 * });
 *
 * // Use with transactions for automatic cleanup
 * await db.transaction(async (tx) => {
 *   const session = factory(tx);
 *
 *   const user = await session.user.create();
 *   const post = await session.post.create({ authorId: user.id });
 *
 *   // Test assertions here...
 *
 *   // Rollback cleans up all created entities
 *   tx.rollback();
 * });
 * ```
 *
 * @param fixtures - Map of fixture names to fixture functions
 * @returns A factory function that creates bound sessions
 */
export function composeFactory<T extends FixturesMap>(fixtures: T): ComposedFactory<T> {
	return (db: DrizzleDatabase): BoundFactory<T> => {
		// Create the bound factory object
		const boundFactory = {} as BoundFactory<T>;

		// Add fixture accessors as getters (lazy initialization)
		for (const name of Object.keys(fixtures) as (keyof T & string)[]) {
			// eslint-disable-next-line @typescript-eslint/no-non-null-assertion -- Reason: name is from Object.keys(fixtures), so fixtures[name] is always defined
			const fixture = fixtures[name]!;
			Object.defineProperty(boundFactory, name, {
				get: () => fixture(db, {}),
				enumerable: true,
			});
		}

		return boundFactory;
	};
}
