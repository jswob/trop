/**
 * FixtureBuilder - Core class for building fixture instances.
 *
 * Implements the builder pattern with immutable trait chaining.
 * Each trait() call returns a NEW builder instance with updated types.
 */
import { getTableName, type InferSelectModel, type InferInsertModel } from 'drizzle-orm';
import type { AnyPgTable } from 'drizzle-orm/pg-core';
import { createUseHelper } from '../context/use-context';
import { executeHook, executeTraitAfterMake } from '../hooks';
import type {
	FixtureConfig,
	FixtureBuilder as IFixtureBuilder,
	FixtureExplanation,
	DrizzleDatabase,
	TraitAugmentations,
	ComputeAugmentedType,
	Prettify,
	FieldResolvers,
	FixtureMode,
	TraitAfterMakeContext,
	BeforeCreateContext,
	AfterCreateContext,
	UseHelper,
} from '../types';

import { TraitNotFoundError, DatabaseOperationError } from '../utils/errors';
import { resolveFields, mergeResolvers } from './field-resolver';
import type { SequenceCounter } from './sequence';

/**
 * Implementation of the FixtureBuilder interface.
 *
 * This class is immutable - trait() returns a new instance rather than mutating.
 */
export class FixtureBuilder<
	TTable extends AnyPgTable,
	// eslint-disable-next-line @typescript-eslint/no-empty-object-type -- Reason: Generic defaults for optional traits/augmentations
	TTraits extends Record<string, unknown> = {},
	// eslint-disable-next-line @typescript-eslint/no-empty-object-type -- Reason: Generic defaults for optional traits/augmentations
	TAugmentations extends TraitAugmentations<TTraits> = {},
	TApplied extends readonly (keyof TTraits)[] = readonly [],
> implements IFixtureBuilder<InferSelectModel<TTable>, TTraits, TAugmentations, TApplied> {
	private readonly config: FixtureConfig<TTable, TTraits, TAugmentations>;
	private readonly db: DrizzleDatabase;
	private readonly sequence: SequenceCounter;
	private readonly appliedTraitNames: readonly string[];
	private readonly appliedTraitParams: ReadonlyMap<string, unknown>;
	private readonly resolutionStack: readonly string[];

	constructor(
		config: FixtureConfig<TTable, TTraits, TAugmentations>,
		db: DrizzleDatabase,
		sequence: SequenceCounter,
		appliedTraitNames: readonly string[] = [],
		appliedTraitParams: ReadonlyMap<string, unknown> = new Map(),
		resolutionStack: readonly string[] = []
	) {
		this.config = config;
		this.db = db;
		this.sequence = sequence;
		this.appliedTraitNames = appliedTraitNames;
		this.appliedTraitParams = appliedTraitParams;
		this.resolutionStack = resolutionStack;
	}

	private createUseHelperForContext(): UseHelper {
		return createUseHelper({
			db: this.db,
			sequence: this.sequence,
			resolutionStack: [...this.resolutionStack],
		});
	}

	/**
	 * Collect field resolvers from all applied traits.
	 *
	 * Validates that each trait exists - throws TraitNotFoundError if not.
	 * Returns resolvers in application order (later traits override earlier).
	 */
	private collectTraitResolvers(): FieldResolvers<InferSelectModel<TTable>, TTable>[] {
		type TModel = InferSelectModel<TTable>;
		const traitResolvers: FieldResolvers<TModel, TTable>[] = [];

		for (const traitName of this.appliedTraitNames) {
			const trait = this.config.traits?.[traitName as keyof TTraits];

			if (!trait) {
				const available = Object.keys(this.config.traits ?? {});
				throw new TraitNotFoundError(traitName, available);
			}

			if (trait.fields) {
				traitResolvers.push(trait.fields);
			}
		}

		return traitResolvers;
	}

	/**
	 * Apply a trait - returns NEW builder with the trait applied.
	 *
	 * The original builder is NOT mutated. This enables method chaining
	 * while preserving immutability for predictable behavior.
	 */
	trait<TName extends keyof TTraits>(
		name: TName,
		...params: TTraits[TName] extends never ? [] : [TTraits[TName]]
	): IFixtureBuilder<
		InferSelectModel<TTable>,
		TTraits,
		TAugmentations,
		readonly [...TApplied, TName]
	> {
		const newParams = new Map(this.appliedTraitParams);
		if (params.length > 0) {
			newParams.set(name as string, params[0]);
		}

		return new FixtureBuilder(
			this.config,
			this.db,
			this.sequence,
			[...this.appliedTraitNames, name as string],
			newParams,
			this.resolutionStack
		) as unknown as IFixtureBuilder<
			InferSelectModel<TTable>,
			TTraits,
			TAugmentations,
			readonly [...TApplied, TName]
		>;
	}

	/**
	 * Build an in-memory object by resolving all field values.
	 *
	 * This does NOT persist to database - use create() for that.
	 */
	async build(
		overrides?: Partial<InferSelectModel<TTable>>
	): Promise<Prettify<ComputeAugmentedType<InferSelectModel<TTable>, TAugmentations, TApplied>>> {
		type TModel = InferSelectModel<TTable>;
		const mode: FixtureMode = 'build';

		// 1. Execute beforeMake hook
		await executeHook('beforeMake', this.config.hooks?.beforeMake, { mode });

		const seq = this.sequence.next();
		const context = {
			use: this.createUseHelperForContext(),
			sequence: seq,
		};

		// 2. Resolve fields (base -> traits -> overrides)
		const traitResolvers = this.collectTraitResolvers();

		const result = await resolveFields<TModel, TTable>({
			baseResolvers: this.config.fields,
			traitResolvers,
			overrides: overrides ?? {},
			context,
		});

		// 3. Execute global afterMake hook (frozen data to prevent mutation)
		await executeHook('afterMake', this.config.hooks?.afterMake, {
			data: Object.freeze({ ...result }),
			mode,
			use: this.createUseHelperForContext(),
		});

		// 4. Execute trait afterMake hooks and collect augmentations
		let augmented = result as Record<string, unknown>;
		for (const traitName of this.appliedTraitNames) {
			const trait = this.config.traits?.[traitName as keyof TTraits];
			if (trait?.afterMake) {
				const params = this.appliedTraitParams.get(traitName);
				const augmentation = await executeTraitAfterMake<TModel, unknown, unknown, TTable>(
					traitName,
					trait.afterMake as (
						ctx: TraitAfterMakeContext<TModel, unknown, TTable>
					) => Promise<unknown>,
					{
						data: augmented as TModel,
						params,
						use: this.createUseHelperForContext(),
						db: this.db,
					}
				);
				if (augmentation != null) {
					augmented = { ...augmented, ...(augmentation as Record<string, unknown>) };
				}
			}
		}

		return augmented as Prettify<ComputeAugmentedType<TModel, TAugmentations, TApplied>>;
	}

	/**
	 * Build multiple in-memory objects.
	 *
	 * Each item gets a unique sequence number.
	 */
	async buildList(
		count: number,
		overrides?: Partial<InferSelectModel<TTable>>
	): Promise<Prettify<ComputeAugmentedType<InferSelectModel<TTable>, TAugmentations, TApplied>>[]> {
		return Promise.all(Array.from({ length: count }, () => this.build(overrides)));
	}

	/**
	 * Create and persist to database.
	 *
	 * Hook execution order:
	 * 1. beforeMake({ mode: 'create' })
	 * 2. Field resolution (base -> traits -> overrides)
	 * 3. afterMake({ data: resolvedData, mode: 'create', use })
	 * 4. beforeCreate({ data: resolvedData })
	 * 5. DB INSERT with .returning()
	 * 6. afterCreate({ data: insertedData })
	 * 7. Trait afterMake hooks (using insertedData - POST-INSERT data)
	 */
	async create(
		overrides?: Partial<InferSelectModel<TTable>>
	): Promise<Prettify<ComputeAugmentedType<InferSelectModel<TTable>, TAugmentations, TApplied>>> {
		type TModel = InferSelectModel<TTable>;
		const mode: FixtureMode = 'create';

		// 1. Execute beforeMake hook
		await executeHook('beforeMake', this.config.hooks?.beforeMake, { mode });

		const seq = this.sequence.next();
		const context = {
			use: this.createUseHelperForContext(),
			sequence: seq,
		};

		// 2. Resolve fields (base -> traits -> overrides)
		const traitResolvers = this.collectTraitResolvers();

		const resolvedData = await resolveFields<TModel, TTable>({
			baseResolvers: this.config.fields,
			traitResolvers,
			overrides: overrides ?? {},
			context,
		});

		// 3. Execute global afterMake hook (frozen data to prevent mutation)
		await executeHook('afterMake', this.config.hooks?.afterMake, {
			data: Object.freeze({ ...resolvedData }),
			mode,
			use: this.createUseHelperForContext(),
		});

		// 4. Execute beforeCreate hook
		await executeHook<BeforeCreateContext<TModel>>(
			'beforeCreate',
			this.config.hooks?.beforeCreate,
			{
				data: resolvedData,
			}
		);

		// 5. Insert into database
		let inserted: TModel;
		try {
			const [result] = await this.db
				.insert(this.config.table)
				.values(resolvedData as unknown as InferInsertModel<TTable>)
				.returning();
			inserted = result as TModel;
		} catch (error: unknown) {
			throw new DatabaseOperationError(getTableName(this.config.table), 'insert', error as Error);
		}

		// 6. Execute afterCreate hook
		await executeHook<AfterCreateContext<TModel>>('afterCreate', this.config.hooks?.afterCreate, {
			data: inserted,
		});

		// 7. Execute trait afterMake hooks with POST-INSERT data (inserted data for real IDs)
		// Key design decision: Trait afterMake receives database-returned data (not resolved data)
		// so that related records can be created with the real database ID
		let augmented = inserted as Record<string, unknown>;
		for (const traitName of this.appliedTraitNames) {
			const trait = this.config.traits?.[traitName as keyof TTraits];
			if (trait?.afterMake) {
				const params = this.appliedTraitParams.get(traitName);
				const augmentation = await executeTraitAfterMake<TModel, unknown, unknown, TTable>(
					traitName,
					trait.afterMake as (
						ctx: TraitAfterMakeContext<TModel, unknown, TTable>
					) => Promise<unknown>,
					{
						data: augmented as TModel,
						params,
						use: this.createUseHelperForContext(),
						db: this.db,
					}
				);
				if (augmentation != null) {
					augmented = { ...augmented, ...(augmentation as Record<string, unknown>) };
				}
			}
		}

		return augmented as Prettify<ComputeAugmentedType<TModel, TAugmentations, TApplied>>;
	}

	/**
	 * Create multiple records and persist to database.
	 *
	 * Each item gets a unique sequence number and full hook cycle.
	 * Records are inserted sequentially (not batched) to ensure
	 * consistent hook execution order.
	 */
	async createList(
		count: number,
		overrides?: Partial<InferSelectModel<TTable>>
	): Promise<Prettify<ComputeAugmentedType<InferSelectModel<TTable>, TAugmentations, TApplied>>[]> {
		const results: Prettify<
			ComputeAugmentedType<InferSelectModel<TTable>, TAugmentations, TApplied>
		>[] = [];
		for (let i = 0; i < count; i++) {
			results.push(await this.create(overrides));
		}
		return results;
	}

	/**
	 * Preview what build() would return without any side effects.
	 *
	 * Currently identical to build() since build() has no side effects,
	 * but semantically indicates "preview" intent.
	 */
	async dryRun(): Promise<
		Prettify<ComputeAugmentedType<InferSelectModel<TTable>, TAugmentations, TApplied>>
	> {
		return this.build();
	}

	/**
	 * Get explanation of how the fixture will resolve fields.
	 *
	 * Useful for debugging and understanding field sources.
	 * Throws TraitNotFoundError if any applied trait doesn't exist.
	 */
	explain(): FixtureExplanation {
		const traitResolvers = this.collectTraitResolvers();

		const { sources } = mergeResolvers(this.config.fields, traitResolvers, []);

		return {
			tableName: getTableName(this.config.table),
			appliedTraits: [...this.appliedTraitNames],
			fieldSources: sources as Record<string, 'base' | 'trait' | 'override'>,
		};
	}
}
