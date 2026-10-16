import type { AnyPgTable } from 'drizzle-orm/pg-core';
import type { FieldResolver, FieldResolverContext, FieldResolvers } from '../types';
import { FieldResolverError } from '../utils/errors';

/**
 * Options for resolving fields into a model object.
 */
export interface ResolveFieldsOptions<TModel, TTable extends AnyPgTable> {
	/** Base field resolvers defined in the fixture */
	baseResolvers: FieldResolvers<TModel, TTable>;
	/** Trait resolvers to apply in order (later traits override earlier) */
	traitResolvers: FieldResolvers<TModel, TTable>[];
	/** Direct overrides from build()/create() call */
	overrides: Partial<TModel>;
	/** Context passed to resolver functions */
	context: FieldResolverContext<TTable>;
}

/**
 * Result of merging resolvers, including source tracking for explain().
 */
export interface MergeResult<TModel, TTable extends AnyPgTable> {
	/** Merged resolvers with traits applied */
	merged: FieldResolvers<TModel, TTable>;
	/** Source of each field: 'base', 'trait', or 'override' */
	sources: Partial<Record<keyof TModel, 'base' | 'trait' | 'override'>>;
}

/**
 * Merges base resolvers with trait resolvers and tracks override keys.
 *
 * This is a pure, synchronous function that determines which resolver wins
 * for each field. Used by explain() to show field sources without executing resolvers.
 *
 * Priority order (highest to lowest):
 * 1. Overrides (direct values from build()/create())
 * 2. Traits (later traits override earlier)
 * 3. Base resolvers
 *
 * @param baseResolvers - Base field resolvers from fixture definition
 * @param traitResolvers - Array of trait resolvers in application order
 * @param overrideKeys - Keys that have explicit overrides
 * @returns Merged resolvers and source map
 *
 * @example
 * ```ts
 * const { merged, sources } = mergeResolvers(
 *   { name: 'Base', email: () => 'base@test.com' },
 *   [{ name: 'Trait Name' }],
 *   ['email']
 * );
 * // sources = { name: 'trait', email: 'override' }
 * ```
 */
export function mergeResolvers<TModel, TTable extends AnyPgTable>(
	baseResolvers: FieldResolvers<TModel, TTable>,
	traitResolvers: FieldResolvers<TModel, TTable>[],
	overrideKeys: (keyof TModel)[]
): MergeResult<TModel, TTable> {
	const merged: FieldResolvers<TModel, TTable> = {};
	const sources: Partial<Record<keyof TModel, 'base' | 'trait' | 'override'>> = {};

	// Step 1: Add base resolvers
	for (const key of Object.keys(baseResolvers) as (keyof TModel)[]) {
		merged[key] = baseResolvers[key];
		sources[key] = 'base';
	}

	// Step 2: Apply traits in order (later traits override earlier)
	for (const traitResolver of traitResolvers) {
		for (const key of Object.keys(traitResolver) as (keyof TModel)[]) {
			merged[key] = traitResolver[key];
			sources[key] = 'trait';
		}
	}

	// Step 3: Mark override sources (resolver not used, but track source)
	for (const key of overrideKeys) {
		sources[key] = 'override';
	}

	return { merged, sources };
}

/**
 * Checks if a value is a resolver function (not a plain value).
 */
function isResolverFunction<T, TTable extends AnyPgTable>(
	resolver: FieldResolver<T, TTable>
): resolver is (ctx: FieldResolverContext<TTable>) => T | Promise<T> {
	return typeof resolver === 'function';
}

/**
 * Resolves all fields into a model object.
 *
 * Executes field resolvers with correct priority:
 * 1. Overrides (direct values) - skip resolver entirely
 * 2. Traits (later traits override earlier) - execute resolver
 * 3. Base resolvers - execute resolver
 *
 * Fields without a resolver AND no override are omitted from the result.
 * This matches Drizzle's insert behavior where omitted fields use defaults.
 *
 * @param options - Resolution options including resolvers, overrides, and context
 * @returns Promise resolving to the complete model object
 * @throws {FieldResolverError} When a resolver function throws
 *
 * @example
 * ```ts
 * const user = await resolveFields({
 *   baseResolvers: {
 *     name: ({ sequence }) => `User ${sequence}`,
 *     email: 'default@test.com',
 *   },
 *   traitResolvers: [],
 *   overrides: { email: 'custom@test.com' },
 *   context: { use, sequence: 1 },
 * });
 * // user = { name: 'User 1', email: 'custom@test.com' }
 * ```
 */
export async function resolveFields<TModel, TTable extends AnyPgTable>(
	options: ResolveFieldsOptions<TModel, TTable>
): Promise<TModel> {
	const { baseResolvers, traitResolvers, overrides, context } = options;

	const overrideKeys = Object.keys(overrides) as (keyof TModel)[];

	const { merged } = mergeResolvers(baseResolvers, traitResolvers, overrideKeys);

	const allFieldNames = new Set<keyof TModel>([
		...(Object.keys(merged) as (keyof TModel)[]),
		...overrideKeys,
	]);

	const result = {} as TModel;

	for (const fieldName of allFieldNames) {
		try {
			if (Object.hasOwn(overrides, fieldName)) {
				result[fieldName] = overrides[fieldName] as TModel[typeof fieldName];
				continue;
			}

			const resolver = merged[fieldName];

			if (resolver === undefined) {
				continue;
			}

			if (isResolverFunction(resolver)) {
				result[fieldName] = (await resolver(context)) as TModel[typeof fieldName];
			} else {
				result[fieldName] = resolver as TModel[typeof fieldName];
			}
		} catch (error) {
			if (error instanceof Error) {
				throw new FieldResolverError(String(fieldName), error);
			}
			throw new FieldResolverError(String(fieldName), new Error(String(error)));
		}
	}

	return result;
}
