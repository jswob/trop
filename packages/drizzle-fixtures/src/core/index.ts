/**
 * Core module for drizzle-fixtures.
 *
 * Contains the field resolution engine, sequence utilities, and helper functions.
 */

export { resolveFields, mergeResolvers } from './field-resolver';
export type { ResolveFieldsOptions, MergeResult } from './field-resolver';

export { createSequenceCounter, SequenceCounter } from './sequence';

export { createPlaceholderUseHelper } from './use-placeholder';

export { FixtureBuilder } from './fixture-builder';

export { createFixture } from './create-fixture';

export { composeFactory } from './compose-factory';
export type { ComposedFactory, BoundFactory, FixturesMap } from './compose-factory';
