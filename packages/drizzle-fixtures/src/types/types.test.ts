import { pgTable, uuid, varchar } from 'drizzle-orm/pg-core';
import { describe, it, expectTypeOf } from 'vitest';
import type {
	FieldResolver,
	FieldResolverContext,
	TraitParams,
	ComputeAugmentedType,
	FixtureBuilder,
	Prettify,
} from './index';

// Mock table for type tests
const mockUsers = pgTable('users', {
	id: uuid('id').primaryKey(),
	name: varchar('name', { length: 255 }).notNull(),
	email: varchar('email', { length: 255 }),
});

describe('Type definitions', () => {
	describe('FieldResolver', () => {
		it('accepts direct values', () => {
			type Result = FieldResolver<string, typeof mockUsers>;
			expectTypeOf<'hello'>().toExtend<Result>();
		});

		it('accepts sync functions', () => {
			type Result = FieldResolver<string, typeof mockUsers>;
			type SyncFn = (ctx: FieldResolverContext<typeof mockUsers>) => string;
			expectTypeOf<SyncFn>().toExtend<Result>();
		});

		it('accepts async functions', () => {
			type Result = FieldResolver<string, typeof mockUsers>;
			type AsyncFn = (ctx: FieldResolverContext<typeof mockUsers>) => Promise<string>;
			expectTypeOf<AsyncFn>().toExtend<Result>();
		});
	});

	describe('TraitParams', () => {
		it('extracts undefined for never traits', () => {
			interface Traits {
				admin: never;
			}
			type Result = TraitParams<Traits, 'admin'>;
			expectTypeOf<Result>().toEqualTypeOf<undefined>();
		});

		it('extracts param type for parameterized traits', () => {
			interface Traits {
				withPosts: { count: number };
			}
			type Result = TraitParams<Traits, 'withPosts'>;
			expectTypeOf<Result>().toEqualTypeOf<{ count: number }>();
		});
	});

	describe('ComputeAugmentedType', () => {
		it('returns base model for empty traits', () => {
			type Model = { id: string; name: string };
			type Augs = { admin: { isAdmin: true } };
			type Result = ComputeAugmentedType<Model, Augs, readonly []>;
			expectTypeOf<Result>().toEqualTypeOf<Model>();
		});

		it('merges single augmentation', () => {
			type Model = { id: string };
			type Augs = { admin: { isAdmin: true } };
			type Result = Prettify<ComputeAugmentedType<Model, Augs, readonly ['admin']>>;
			expectTypeOf<Result>().toEqualTypeOf<{ id: string; isAdmin: true }>();
		});

		it('merges multiple augmentations', () => {
			type Model = { id: string };
			type Augs = { admin: { isAdmin: true }; vip: { vipLevel: number } };
			type Result = Prettify<ComputeAugmentedType<Model, Augs, readonly ['admin', 'vip']>>;
			expectTypeOf<Result>().toEqualTypeOf<{ id: string; isAdmin: true; vipLevel: number }>();
		});
	});

	describe('FixtureBuilder', () => {
		it('trait() returns builder with accumulated types', () => {
			type Traits = {
				admin: never;
				vip: { level: number };
			};
			type Augs = {
				admin: { isAdmin: true };
				vip: { vipLevel: number };
			};
			type Model = { id: string };

			type Builder = FixtureBuilder<Model, Traits, Augs, readonly []>;
			type AfterAdmin = ReturnType<Builder['trait']>;

			expectTypeOf<AfterAdmin>().toExtend<
				FixtureBuilder<Model, Traits, Augs, readonly ['admin']>
			>();
		});
	});
});
