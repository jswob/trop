import { describe, it, expect, vi } from 'vitest';

import { users, posts } from '../fixtures/test-schema';
import type { DrizzleDatabase, FixtureConfig } from '../types';
import { FixtureBuilder } from './fixture-builder';
import { SequenceCounter } from './sequence';

// Mock database for testing
const mockDb = {} as DrizzleDatabase;

describe('FixtureBuilder', () => {
	describe('build()', () => {
		it('returns object with resolved fields', async () => {
			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test User',
					email: 'test@example.com',
					role: 'user',
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const result = await builder.build();

			expect(result).toEqual({
				id: 1,
				name: 'Test User',
				email: 'test@example.com',
				role: 'user',
			});
		});

		it('calls resolvers with sequence number', async () => {
			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: ({ sequence }) => sequence,
					name: ({ sequence }) => `User ${sequence}`,
					email: ({ sequence }) => `user${sequence}@example.com`,
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const result = await builder.build();

			expect(result).toEqual({
				id: 1,
				name: 'User 1',
				email: 'user1@example.com',
			});
		});

		it('increments sequence for each build() call', async () => {
			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: ({ sequence }) => sequence,
					name: ({ sequence }) => `User ${sequence}`,
				},
			};

			const sequence = new SequenceCounter();
			const builder = new FixtureBuilder(config, mockDb, sequence);

			const first = await builder.build();
			const second = await builder.build();
			const third = await builder.build();

			expect(first.id).toBe(1);
			expect(second.id).toBe(2);
			expect(third.id).toBe(3);
		});

		it('applies overrides correctly', async () => {
			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: ({ sequence }) => sequence,
					name: 'Default Name',
					email: 'default@example.com',
					role: 'user',
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const result = await builder.build({
				name: 'Custom Name',
				role: 'admin',
			});

			expect(result).toEqual({
				id: 1,
				name: 'Custom Name',
				email: 'default@example.com',
				role: 'admin',
			});
		});

		it('override prevents resolver from running', async () => {
			const nameSpy = vi.fn().mockReturnValue('Resolved Name');

			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: 1,
					name: nameSpy,
					email: 'test@example.com',
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const result = await builder.build({ name: 'Override Name' });

			expect(nameSpy).not.toHaveBeenCalled();
			expect(result.name).toBe('Override Name');
		});

		it('supports async resolvers', async () => {
			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: 1,
					name: async ({ sequence }) => {
						await new Promise(resolve => setTimeout(resolve, 1));
						return `Async User ${sequence}`;
					},
					email: 'test@example.com',
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const result = await builder.build();

			expect(result.name).toBe('Async User 1');
		});
	});

	describe('buildList()', () => {
		it('returns array of specified count', async () => {
			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: ({ sequence }) => sequence,
					name: 'Test User',
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const results = await builder.buildList(3);

			expect(results).toHaveLength(3);
		});

		it('each item has different sequence number', async () => {
			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: ({ sequence }) => sequence,
					name: ({ sequence }) => `User ${sequence}`,
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const results = await builder.buildList(3);

			expect(results[0]!.id).toBe(1);
			expect(results[1]!.id).toBe(2);
			expect(results[2]!.id).toBe(3);
			expect(results[0]!.name).toBe('User 1');
			expect(results[1]!.name).toBe('User 2');
			expect(results[2]!.name).toBe('User 3');
		});

		it('applies same overrides to all items', async () => {
			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: ({ sequence }) => sequence,
					name: 'Default',
					role: 'user',
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const results = await builder.buildList(2, { role: 'admin' });

			expect(results[0]!.role).toBe('admin');
			expect(results[1]!.role).toBe('admin');
		});

		it('returns empty array for count 0', async () => {
			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test',
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const results = await builder.buildList(0);

			expect(results).toEqual([]);
		});
	});

	describe('trait()', () => {
		it('returns NEW builder instance (immutability)', () => {
			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test',
				},
			};

			const original = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const withTrait = original.trait('admin' as never);

			expect(withTrait).not.toBe(original);
			expect(withTrait).toBeInstanceOf(FixtureBuilder);
		});

		it('original builder is unchanged after trait()', () => {
			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test',
				},
			};

			const original = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const originalExplanation = original.explain();

			// Apply trait
			original.trait('admin' as never);

			// Original should be unchanged
			expect(original.explain().appliedTraits).toEqual(originalExplanation.appliedTraits);
		});

		it('trait() can be chained', () => {
			type UserTraits = { admin: never; verified: never };
			const config: FixtureConfig<typeof users, UserTraits> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test',
				},
				traits: {
					admin: { fields: { role: 'admin' } },
					verified: { fields: {} },
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const chained = builder.trait('admin').trait('verified');

			expect(chained.explain().appliedTraits).toEqual(['admin', 'verified']);
		});
	});

	describe('dryRun()', () => {
		it('returns same result as build()', async () => {
			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test User',
					email: 'test@example.com',
				},
			};

			const sequence = new SequenceCounter();
			const builder1 = new FixtureBuilder(config, mockDb, sequence);
			const builder2 = new FixtureBuilder(config, mockDb, new SequenceCounter());

			// Reset sequence for comparison
			const buildResult = await builder2.build();
			const dryRunResult = await builder1.dryRun();

			expect(dryRunResult).toEqual(buildResult);
		});
	});

	describe('explain()', () => {
		it('returns correct table name', () => {
			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test',
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const explanation = builder.explain();

			expect(explanation.tableName).toBe('users');
		});

		it('returns applied traits', () => {
			type UserTraits = { admin: never; verified: never };
			const config: FixtureConfig<typeof users, UserTraits> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test',
				},
				traits: {
					admin: { fields: { role: 'admin' } },
					verified: { fields: {} },
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter(), [
				'admin',
				'verified',
			]);
			const explanation = builder.explain();

			expect(explanation.appliedTraits).toEqual(['admin', 'verified']);
		});

		it('returns empty appliedTraits for fresh builder', () => {
			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test',
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const explanation = builder.explain();

			expect(explanation.appliedTraits).toEqual([]);
		});

		it('returns field sources for base fields', () => {
			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test',
					email: 'test@example.com',
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const explanation = builder.explain();

			expect(explanation.fieldSources).toEqual({
				id: 'base',
				name: 'base',
				email: 'base',
			});
		});

		it('works with posts table', () => {
			const config: FixtureConfig<typeof posts> = {
				table: posts,
				fields: {
					id: 1,
					title: 'Test Post',
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const explanation = builder.explain();

			expect(explanation.tableName).toBe('posts');
			expect(explanation.fieldSources).toEqual({
				id: 'base',
				title: 'base',
			});
		});
	});
});
