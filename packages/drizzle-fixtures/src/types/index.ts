import type { InferSelectModel } from 'drizzle-orm';
import type { AnyPgTable, PgDatabase } from 'drizzle-orm/pg-core';

import type { FixtureInternalOptions } from './internal';

// ============================================
// CORE TYPES
// ============================================

export type FixtureMode = 'build' | 'create';

/** Database connection type - PostgreSQL focused for now */
// eslint-disable-next-line @typescript-eslint/no-explicit-any -- Reason: Drizzle ORM compatibility - PgDatabase generic params aren't relevant to fixture operations
export type DrizzleDatabase = PgDatabase<any, any>;

/** Context passed to field resolvers */
// eslint-disable-next-line @typescript-eslint/no-unused-vars -- Reason: TTable reserved for future use to enable table-specific context features
export interface FieldResolverContext<TTable extends AnyPgTable> {
	use: UseHelper;
	sequence: number;
}

/** A field value or resolver function */
export type FieldResolver<T, TTable extends AnyPgTable> =
	T | ((ctx: FieldResolverContext<TTable>) => T | Promise<T>);

/** Map of field resolvers for all model fields */
export type FieldResolvers<TModel, TTable extends AnyPgTable> = {
	[K in keyof TModel]?: FieldResolver<TModel[K], TTable>;
};

// ============================================
// TRAIT TYPES
// ============================================

/** Constraint: augmentation keys must match trait keys */
export type TraitAugmentations<TTraits> = {
	[K in keyof TTraits]?: Record<string, unknown>;
};

/** Extract params for a trait (never -> undefined for no-param traits) */
export type TraitParams<TTraits, TName extends keyof TTraits> = TTraits[TName] extends never
	? undefined
	: TTraits[TName];

/** Context passed to trait's afterMake hook */
// eslint-disable-next-line @typescript-eslint/no-unused-vars -- Reason: TTable reserved for future use to enable table-specific context features
export interface TraitAfterMakeContext<TModel, TParams, TTable extends AnyPgTable> {
	data: TModel;
	params: TParams;
	use: UseHelper;
	db: DrizzleDatabase;
}

/** Single trait definition */
export interface TraitDefinition<TModel, TParams, TAugmentation, TTable extends AnyPgTable> {
	fields?: FieldResolvers<TModel, TTable>;
	afterMake?: (
		ctx: TraitAfterMakeContext<TModel, TParams, TTable>
	) => TAugmentation | Promise<TAugmentation>;
}

/** Full traits configuration object */
export type TraitsConfig<
	TTraits extends Record<string, unknown>,
	TAugmentations extends TraitAugmentations<TTraits>,
	TTable extends AnyPgTable,
> = {
	[K in keyof TTraits]: TraitDefinition<
		InferSelectModel<TTable>,
		TraitParams<TTraits, K>,
		// eslint-disable-next-line @typescript-eslint/no-empty-object-type -- Reason: Intentional empty object type for "no augmentations" base case
		TAugmentations[K] extends Record<string, unknown> ? TAugmentations[K] : {},
		TTable
	>;
};

// ============================================
// HOOK TYPES
// ============================================

export interface BeforeMakeContext {
	mode: FixtureMode;
}

export interface AfterMakeContext<TModel> {
	data: TModel;
	mode: FixtureMode;
	use: UseHelper;
}

export interface BeforeCreateContext<TModel> {
	data: TModel;
}

export interface AfterCreateContext<TModel> {
	data: TModel;
}

export interface LifecycleHooks<TModel> {
	beforeMake?: (ctx: BeforeMakeContext) => void | Promise<void>;
	afterMake?: (ctx: AfterMakeContext<TModel>) => void | Promise<void>;
	beforeCreate?: (ctx: BeforeCreateContext<TModel>) => void | Promise<void>;
	afterCreate?: (ctx: AfterCreateContext<TModel>) => void | Promise<void>;
}

// ============================================
// BUILDER TYPES
// ============================================

/** Recursive merge of augmentations from applied traits */
type MergeAugmentations<
	TAugmentations,
	TApplied extends readonly (keyof TAugmentations)[],
> = TApplied extends readonly []
	? // eslint-disable-next-line @typescript-eslint/no-empty-object-type -- Reason: Intentional empty object type for "no augmentations" base case
		{}
	: TApplied extends readonly [
				infer First extends keyof TAugmentations,
				...infer Rest extends readonly (keyof TAugmentations)[],
		  ]
		? // eslint-disable-next-line @typescript-eslint/no-empty-object-type -- Reason: Intentional empty object type for "no augmentations" base case
			(TAugmentations[First] extends Record<string, unknown> ? TAugmentations[First] : {}) &
				MergeAugmentations<TAugmentations, Rest>
		: // eslint-disable-next-line @typescript-eslint/no-empty-object-type -- Reason: Intentional empty object type for "no augmentations" base case
			{};

/** Compute final type: model + merged augmentations */
export type ComputeAugmentedType<
	TModel,
	TAugmentations,
	TApplied extends readonly (keyof TAugmentations)[],
> = TModel & MergeAugmentations<TAugmentations, TApplied>;

/** Helper for cleaner IDE tooltips */
export type Prettify<T> = { [K in keyof T]: T[K] } & {};

/** Explanation returned by explain() */
export interface FixtureExplanation {
	tableName: string;
	appliedTraits: string[];
	fieldSources: Record<string, 'base' | 'trait' | 'override'>;
}

/** Builder tracks applied traits via TApplied tuple */
export interface FixtureBuilder<
	TModel,
	TTraits extends Record<string, unknown>,
	TAugmentations extends TraitAugmentations<TTraits>,
	TApplied extends readonly (keyof TTraits)[] = readonly [],
> {
	/** Apply a trait - returns NEW builder (immutable) */
	trait<TName extends keyof TTraits>(
		name: TName,
		...params: TTraits[TName] extends never ? [] : [TTraits[TName]]
	): FixtureBuilder<TModel, TTraits, TAugmentations, readonly [...TApplied, TName]>;

	/** Build in-memory object */
	build(
		overrides?: Partial<TModel>
	): Promise<Prettify<ComputeAugmentedType<TModel, TAugmentations, TApplied>>>;
	buildList(
		count: number,
		overrides?: Partial<TModel>
	): Promise<Prettify<ComputeAugmentedType<TModel, TAugmentations, TApplied>>[]>;

	/** Build and persist to database */
	create(
		overrides?: Partial<TModel>
	): Promise<Prettify<ComputeAugmentedType<TModel, TAugmentations, TApplied>>>;
	createList(
		count: number,
		overrides?: Partial<TModel>
	): Promise<Prettify<ComputeAugmentedType<TModel, TAugmentations, TApplied>>[]>;

	/** Preview and debug */
	dryRun(): Promise<Prettify<ComputeAugmentedType<TModel, TAugmentations, TApplied>>>;
	explain(): FixtureExplanation;
}

// Re-export internal types (separated to avoid circular dependencies)
export type { FixtureInternalOptions };

/** Fixture function returned by createFixture */
export type FixtureFunction<
	TTable extends AnyPgTable,
	TTraits extends Record<string, unknown>,
	TAugmentations extends TraitAugmentations<TTraits>,
> = (
	db: DrizzleDatabase,
	internalOptions?: FixtureInternalOptions
) => FixtureBuilder<InferSelectModel<TTable>, TTraits, TAugmentations, readonly []>;

/** Helper for composing fixtures */
export type UseHelper = <
	TTable extends AnyPgTable,
	TTraits extends Record<string, unknown>,
	TAugmentations extends TraitAugmentations<TTraits>,
>(
	fixture: FixtureFunction<TTable, TTraits, TAugmentations>
) => FixtureBuilder<InferSelectModel<TTable>, TTraits, TAugmentations, readonly []>;

// ============================================
// CONFIG TYPES
// ============================================

/** Configuration for createFixture() */
export interface FixtureConfig<
	TTable extends AnyPgTable,
	// eslint-disable-next-line @typescript-eslint/no-empty-object-type -- Reason: Generic defaults for optional traits/augmentations
	TTraits extends Record<string, unknown> = {},
	// eslint-disable-next-line @typescript-eslint/no-empty-object-type -- Reason: Generic defaults for optional traits/augmentations
	TAugmentations extends TraitAugmentations<TTraits> = {},
> {
	table: TTable;
	fields: FieldResolvers<InferSelectModel<TTable>, TTable>;
	traits?: TraitsConfig<TTraits, TAugmentations, TTable>;
	hooks?: LifecycleHooks<InferSelectModel<TTable>>;
}
