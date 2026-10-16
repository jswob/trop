/**
 * @jswob/drizzle-fixtures
 *
 * Type-safe database fixtures for Drizzle ORM with traits, relationships, and transaction support.
 *
 * This package provides a powerful way to create test data and database fixtures
 * with full type safety, reusable traits, and complex relationship resolution.
 */

export type {
	// Core types
	FixtureMode,
	DrizzleDatabase,
	FieldResolverContext,
	FieldResolver,
	FieldResolvers,
	// Trait types
	TraitAugmentations,
	TraitParams,
	TraitAfterMakeContext,
	TraitDefinition,
	TraitsConfig,
	// Hook types
	BeforeMakeContext,
	AfterMakeContext,
	BeforeCreateContext,
	AfterCreateContext,
	LifecycleHooks,
	// Builder types
	ComputeAugmentedType,
	Prettify,
	FixtureExplanation,
	FixtureBuilder,
	FixtureFunction,
	UseHelper,
	// Config types
	FixtureConfig,
} from './types/index';

// Re-export error classes
export {
	FixtureError,
	TraitNotFoundError,
	FieldResolverError,
	CircularDependencyError,
	DatabaseOperationError,
	ValidationError,
	HookExecutionError,
} from './utils/errors';

// Re-export hook utilities
export { executeHook, executeTraitAfterMake } from './hooks';

// Re-export core utilities
export { resolveFields, mergeResolvers, createSequenceCounter, SequenceCounter } from './core';
export type { ResolveFieldsOptions, MergeResult } from './core';

// Main fixture creation function
export { createFixture } from './core';

// Factory composition
export { composeFactory } from './core';
export type { ComposedFactory, BoundFactory, FixturesMap } from './core';

// Re-export context utilities (use helper)
export { createUseHelper } from './context';
export type { UseContextOptions } from './context';

// Re-export table utilities
export { getTableName, getPrimaryKeyColumn } from './utils/table-utils';
