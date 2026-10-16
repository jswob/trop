import { describe, it, expect, vi, expectTypeOf } from 'vitest';
import { FixtureBuilder } from '../core/fixture-builder';
import { SequenceCounter } from '../core/sequence';
import { users } from '../fixtures/test-schema';
import type { DrizzleDatabase, FixtureConfig, TraitAugmentations } from '../types';
import { HookExecutionError } from '../utils/errors';

const mockDb = {} as DrizzleDatabase;

describe('Trait afterMake Augmentations', () => {
	describe('Basic augmentation behavior', () => {
		it('trait afterMake receives data and params', async () => {
			const afterMake = vi.fn().mockReturnValue({});

			type UserTraits = { withParam: string };
			// eslint-disable-next-line @typescript-eslint/no-empty-object-type -- Reason: Test type setup - empty augmentation type for no-augmentation test case
			type UserAugmentations = TraitAugmentations<UserTraits> & { withParam: {} };

			const config: FixtureConfig<typeof users, UserTraits, UserAugmentations> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test User',
					email: 'test@example.com',
				},
				traits: {
					withParam: {
						afterMake,
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			await builder.trait('withParam', 'test-param').build();

			expect(afterMake).toHaveBeenCalledWith(
				expect.objectContaining({
					// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment -- Reason: Vitest expect.objectContaining returns any type
					data: expect.objectContaining({
						id: 1,
						name: 'Test User',
						email: 'test@example.com',
					}),
					params: 'test-param',
					// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment -- Reason: Vitest expect.any returns any type
					use: expect.any(Function),
					db: mockDb,
				})
			);
		});

		it('trait afterMake return value is merged into result', async () => {
			type UserTraits = { withMeta: never };
			type UserAugmentations = TraitAugmentations<UserTraits> & {
				withMeta: { metadata: { key: string } };
			};

			const config: FixtureConfig<typeof users, UserTraits, UserAugmentations> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test User',
					email: 'test@example.com',
				},
				traits: {
					withMeta: {
						afterMake: () => ({
							metadata: { key: 'value' },
						}),
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const result = await builder.trait('withMeta').build();

			expect(result).toMatchObject({
				id: 1,
				name: 'Test User',
				metadata: { key: 'value' },
			});
		});

		it('trait afterMake can use params to customize augmentation', async () => {
			type UserTraits = { withRole: { role: string; level: number } };
			type UserAugmentations = TraitAugmentations<UserTraits> & {
				withRole: { computedRole: string };
			};

			const config: FixtureConfig<typeof users, UserTraits, UserAugmentations> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test User',
					email: 'test@example.com',
				},
				traits: {
					withRole: {
						afterMake: ({ params }) => ({
							computedRole: `${params.role}-level-${params.level}`,
						}),
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const result = await builder.trait('withRole', { role: 'admin', level: 5 }).build();

			expect(result.computedRole).toBe('admin-level-5');
		});
	});

	describe('Multiple trait augmentations', () => {
		it('multiple trait afterMakes merge in order (later sees earlier augmentations)', async () => {
			const callOrder: string[] = [];

			type UserTraits = { first: never; second: never };
			type UserAugmentations = TraitAugmentations<UserTraits> & {
				first: { firstProp: string };
				second: { secondProp: string };
			};

			const config: FixtureConfig<typeof users, UserTraits, UserAugmentations> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test User',
					email: 'test@example.com',
				},
				traits: {
					first: {
						afterMake: ({ data }) => {
							callOrder.push('first');
							expect(data).not.toHaveProperty('secondProp');
							return { firstProp: 'from-first' };
						},
					},
					second: {
						afterMake: ({ data }) => {
							callOrder.push('second');
							// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access -- Reason: Testing augmentation behavior - accessing dynamically added properties requires type assertion
							expect((data as any).firstProp).toBe('from-first');
							return { secondProp: 'from-second' };
						},
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const result = await builder.trait('first').trait('second').build();

			expect(callOrder).toEqual(['first', 'second']);
			expect(result).toMatchObject({
				id: 1,
				firstProp: 'from-first',
				secondProp: 'from-second',
			});
		});

		it('later trait augmentation can override earlier trait augmentation', async () => {
			type UserTraits = { first: never; second: never };
			type UserAugmentations = TraitAugmentations<UserTraits> & {
				first: { sharedProp: string };
				second: { sharedProp: string };
			};

			const config: FixtureConfig<typeof users, UserTraits, UserAugmentations> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test User',
					email: 'test@example.com',
				},
				traits: {
					first: {
						afterMake: () => ({
							sharedProp: 'from-first',
						}),
					},
					second: {
						afterMake: () => ({
							sharedProp: 'from-second',
						}),
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const result = await builder.trait('first').trait('second').build();

			expect(result.sharedProp).toBe('from-second');
		});
	});

	describe('Trait with fields and afterMake', () => {
		it('trait without afterMake but with fields - no hook call, fields still applied', async () => {
			const afterMake = vi.fn();

			type UserTraits = { admin: never };

			const config: FixtureConfig<typeof users, UserTraits> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test User',
					role: 'user',
					email: 'test@example.com',
				},
				traits: {
					admin: {
						fields: {
							role: 'admin',
						},
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const result = await builder.trait('admin').build();

			expect(result.role).toBe('admin');
			expect(afterMake).not.toHaveBeenCalled();
		});

		it('trait with both fields and afterMake - both are applied', async () => {
			type UserTraits = { admin: never };
			type UserAugmentations = TraitAugmentations<UserTraits> & {
				admin: { permissions: string[] };
			};

			const config: FixtureConfig<typeof users, UserTraits, UserAugmentations> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test User',
					role: 'user',
					email: 'test@example.com',
				},
				traits: {
					admin: {
						fields: {
							role: 'admin',
						},
						afterMake: () => ({
							permissions: ['read', 'write', 'delete'],
						}),
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const result = await builder.trait('admin').build();

			expect(result.role).toBe('admin');
			expect(result.permissions).toEqual(['read', 'write', 'delete']);
		});
	});

	describe('Async afterMake', () => {
		it('trait afterMake with async function works', async () => {
			type UserTraits = { withAsyncData: never };
			type UserAugmentations = TraitAugmentations<UserTraits> & {
				withAsyncData: { asyncValue: string };
			};

			const config: FixtureConfig<typeof users, UserTraits, UserAugmentations> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test User',
					email: 'test@example.com',
				},
				traits: {
					withAsyncData: {
						afterMake: async () => {
							await new Promise(r => setTimeout(r, 10));
							return { asyncValue: 'loaded' };
						},
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const result = await builder.trait('withAsyncData').build();

			expect(result.asyncValue).toBe('loaded');
		});
	});

	describe('Combined global and trait afterMake', () => {
		it('global afterMake + trait afterMake both execute in order', async () => {
			const callOrder: string[] = [];

			type UserTraits = { withAug: never };
			type UserAugmentations = TraitAugmentations<UserTraits> & {
				withAug: { traitProp: string };
			};

			const config: FixtureConfig<typeof users, UserTraits, UserAugmentations> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test User',
					email: 'test@example.com',
				},
				hooks: {
					afterMake: () => {
						callOrder.push('global');
					},
				},
				traits: {
					withAug: {
						afterMake: () => {
							callOrder.push('trait');
							return { traitProp: 'value' };
						},
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			await builder.trait('withAug').build();

			expect(callOrder).toEqual(['global', 'trait']);
		});
	});

	describe('Error handling', () => {
		it('trait afterMake throws error - HookExecutionError with trait name', async () => {
			type UserTraits = { failing: never };
			type UserAugmentations = TraitAugmentations<UserTraits> & {
				// eslint-disable-next-line @typescript-eslint/no-empty-object-type -- Reason: Test type setup - empty augmentation type for error handling test case
				failing: {};
			};

			const config: FixtureConfig<typeof users, UserTraits, UserAugmentations> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test User',
					email: 'test@example.com',
				},
				traits: {
					failing: {
						afterMake: () => {
							throw new Error('trait hook failed');
						},
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());

			await expect(builder.trait('failing').build()).rejects.toThrow(HookExecutionError);
			await expect(builder.trait('failing').build()).rejects.toThrow(
				'Hook "trait.failing.afterMake" failed: trait hook failed'
			);
		});

		it('HookExecutionError includes trait name in hookName', async () => {
			type UserTraits = { brokenTrait: never };
			type UserAugmentations = TraitAugmentations<UserTraits> & {
				// eslint-disable-next-line @typescript-eslint/no-empty-object-type -- Reason: Test type setup - empty augmentation type for error handling test case
				brokenTrait: {};
			};

			const config: FixtureConfig<typeof users, UserTraits, UserAugmentations> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test User',
					email: 'test@example.com',
				},
				traits: {
					brokenTrait: {
						afterMake: () => {
							throw new Error('oops');
						},
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());

			try {
				await builder.trait('brokenTrait').build();
			} catch (e) {
				expect(e).toBeInstanceOf(HookExecutionError);
				expect((e as HookExecutionError).hookName).toBe('trait.brokenTrait.afterMake');
			}
		});
	});

	describe('Null/undefined augmentation handling', () => {
		it('trait afterMake returns null - handled gracefully (no merge)', async () => {
			type UserTraits = { nullTrait: never };
			type UserAugmentations = TraitAugmentations<UserTraits> & {
				// eslint-disable-next-line @typescript-eslint/no-empty-object-type -- Reason: Test type setup - empty augmentation type for null-return test case
				nullTrait: {};
			};

			const config: FixtureConfig<typeof users, UserTraits, UserAugmentations> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test User',
					email: 'test@example.com',
				},
				traits: {
					nullTrait: {
						afterMake: () => null as unknown as Record<string, never>,
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const result = await builder.trait('nullTrait').build();

			expect(result.id).toBe(1);
			expect(result.name).toBe('Test User');
		});

		it('trait afterMake returns undefined - handled gracefully (no merge)', async () => {
			type UserTraits = { undefinedTrait: never };
			type UserAugmentations = TraitAugmentations<UserTraits> & {
				// eslint-disable-next-line @typescript-eslint/no-empty-object-type -- Reason: Test type setup - empty augmentation type for undefined-return test case
				undefinedTrait: {};
			};

			const config: FixtureConfig<typeof users, UserTraits, UserAugmentations> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test User',
					email: 'test@example.com',
				},
				traits: {
					undefinedTrait: {
						afterMake: () => undefined as unknown as Record<string, never>,
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const result = await builder.trait('undefinedTrait').build();

			expect(result.id).toBe(1);
			expect(result.name).toBe('Test User');
		});

		it('empty augmentation object {} - merged (no-op)', async () => {
			type UserTraits = { emptyTrait: never };
			type UserAugmentations = TraitAugmentations<UserTraits> & {
				// eslint-disable-next-line @typescript-eslint/no-empty-object-type -- Reason: Test type setup - empty augmentation type for empty-return test case
				emptyTrait: {};
			};

			const config: FixtureConfig<typeof users, UserTraits, UserAugmentations> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test User',
					email: 'test@example.com',
				},
				traits: {
					emptyTrait: {
						afterMake: () => ({}),
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const result = await builder.trait('emptyTrait').build();

			expect(result.id).toBe(1);
			expect(result.name).toBe('Test User');
		});
	});

	describe('Type tests', () => {
		it('augmentation types are correct', async () => {
			type UserTraits = { withMeta: never };
			type UserAugmentations = TraitAugmentations<UserTraits> & {
				withMeta: { metadata: { count: number } };
			};

			const config: FixtureConfig<typeof users, UserTraits, UserAugmentations> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test User',
					email: 'test@example.com',
				},
				traits: {
					withMeta: {
						afterMake: () => ({
							metadata: { count: 42 },
						}),
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const result = await builder.trait('withMeta').build();

			expectTypeOf(result.id).toBeNumber();
			expectTypeOf(result.name).toBeString();
			expectTypeOf(result.metadata).toEqualTypeOf<{ count: number }>();
		});

		it('multiple augmentations produce intersection type', async () => {
			type UserTraits = { first: never; second: never };
			type UserAugmentations = TraitAugmentations<UserTraits> & {
				first: { firstProp: boolean };
				second: { secondProp: number };
			};

			const config: FixtureConfig<typeof users, UserTraits, UserAugmentations> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test User',
					email: 'test@example.com',
				},
				traits: {
					first: {
						afterMake: () => ({ firstProp: true }),
					},
					second: {
						afterMake: () => ({ secondProp: 123 }),
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const result = await builder.trait('first').trait('second').build();

			expectTypeOf(result.id).toBeNumber();
			expectTypeOf(result.firstProp).toBeBoolean();
			expectTypeOf(result.secondProp).toBeNumber();
		});
	});

	describe('buildList with trait augmentations', () => {
		it('trait afterMake executes for each item in buildList', async () => {
			const afterMakeCalls: number[] = [];

			type UserTraits = { tracked: never };
			type UserAugmentations = TraitAugmentations<UserTraits> & {
				tracked: { callIndex: number };
			};

			const config: FixtureConfig<typeof users, UserTraits, UserAugmentations> = {
				table: users,
				fields: {
					id: ({ sequence }) => sequence,
					name: 'Test User',
					email: 'test@example.com',
				},
				traits: {
					tracked: {
						afterMake: ({ data }) => {
							afterMakeCalls.push(data.id);
							return { callIndex: afterMakeCalls.length };
						},
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const results = await builder.trait('tracked').buildList(3);

			expect(afterMakeCalls).toHaveLength(3);
			expect(results[0]!.callIndex).toBeDefined();
			expect(results[1]!.callIndex).toBeDefined();
			expect(results[2]!.callIndex).toBeDefined();
		});
	});
});
