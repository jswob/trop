import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { Mock } from 'vitest';

import { createFixture } from '../core/create-fixture';
import { users, posts } from '../fixtures/test-schema';
import type { DrizzleDatabase, FixtureFunction } from '../types';
import { FieldResolverError } from '../utils/errors';

function createMockDb(): DrizzleDatabase & {
	insert: Mock;
	_values: Mock;
	_returning: Mock;
} {
	let insertId = 1;
	const _returning = vi.fn().mockImplementation(() =>
		Promise.resolve([
			{
				id: insertId++,
				name: 'Test User',
				email: 'test@example.com',
				role: 'user',
				createdAt: new Date(),
			},
		])
	);
	const _values = vi.fn().mockReturnValue({ returning: _returning });
	const insertFn = vi.fn().mockReturnValue({ values: _values });

	return {
		insert: insertFn,
		_values,
		_returning,
	} as unknown as DrizzleDatabase & {
		insert: Mock;
		_values: Mock;
		_returning: Mock;
	};
}

describe('Fixture Composition (use helper)', () => {
	let mockDb: DrizzleDatabase & { insert: Mock; _values: Mock; _returning: Mock };

	beforeEach(() => {
		mockDb = createMockDb();
	});

	describe('Basic Composition', () => {
		it('field resolver can use use(fixture).build()', async () => {
			const userFixture = createFixture({
				table: users,
				fields: {
					name: ({ sequence }) => `User ${sequence}`,
					email: ({ sequence }) => `user${sequence}@example.com`,
				},
			});

			const postFixture = createFixture({
				table: posts,
				fields: {
					title: 'Test Post',
					content: 'Content',
					userId: async ({ use }) => {
						const user = await use(userFixture).build();
						return user.id;
					},
				},
			});

			const post = await postFixture(mockDb).build();

			expect(post.title).toBe('Test Post');
			expect(post.userId).toBeUndefined();
		});

		it('field resolver can use use(fixture).create()', async () => {
			let createdUserId = 1;
			const mockDbWithUser = {
				...mockDb,
				insert: vi.fn().mockImplementation(table => ({
					values: vi.fn().mockReturnValue({
						returning: vi.fn().mockImplementation(() => {
							if (table === users) {
								return Promise.resolve([
									{
										id: createdUserId++,
										name: 'Created User',
										email: 'user@example.com',
										role: 'user',
										createdAt: new Date(),
									},
								]);
							}
							return Promise.resolve([
								{
									id: 1,
									title: 'Test Post',
									content: 'Content',
									userId: 1,
									status: 'draft',
									createdAt: new Date(),
								},
							]);
						}),
					}),
				})),
			} as unknown as DrizzleDatabase;

			const userFixture = createFixture({
				table: users,
				fields: {
					name: ({ sequence }) => `User ${sequence}`,
					email: ({ sequence }) => `user${sequence}@example.com`,
				},
			});

			const postFixture = createFixture({
				table: posts,
				fields: {
					title: 'Test Post',
					content: 'Content',
					userId: async ({ use }) => {
						const user = await use(userFixture).create();
						return user.id;
					},
				},
			});

			const post = await postFixture(mockDbWithUser).create();

			expect(post.title).toBe('Test Post');
			expect(post.userId).toBe(1);
		});

		it('trait afterMake can use use(fixture).build()', async () => {
			const userFixture = createFixture({
				table: users,
				fields: {
					name: ({ sequence }) => `User ${sequence}`,
					email: ({ sequence }) => `user${sequence}@example.com`,
				},
			});

			type PostTraits = { withAuthor: never };
			type PostAugmentations = { withAuthor: { author: { id: number | undefined; name: string } } };

			const postFixture = createFixture<typeof posts, PostTraits, PostAugmentations>({
				table: posts,
				fields: {
					title: 'Test Post',
					content: 'Content',
				},
				traits: {
					withAuthor: {
						afterMake: async ({ use }) => {
							const author = await use(userFixture).build();
							return { author: { id: author.id, name: author.name } };
						},
					},
				},
			});

			const post = await postFixture(mockDb).trait('withAuthor').build();

			expect(post.title).toBe('Test Post');
			expect(post.author.name).toBe('User 2');
		});

		it('trait afterMake can use use(fixture).create()', async () => {
			let createdUserId = 1;
			const mockDbWithUser = {
				...mockDb,
				insert: vi.fn().mockImplementation(table => ({
					values: vi.fn().mockReturnValue({
						returning: vi.fn().mockImplementation(() => {
							if (table === users) {
								return Promise.resolve([
									{
										id: createdUserId++,
										name: 'Created User',
										email: 'user@example.com',
										role: 'user',
										createdAt: new Date(),
									},
								]);
							}
							return Promise.resolve([
								{
									id: 1,
									title: 'Test Post',
									content: 'Content',
									userId: null,
									status: 'draft',
									createdAt: new Date(),
								},
							]);
						}),
					}),
				})),
			} as unknown as DrizzleDatabase;

			const userFixture = createFixture({
				table: users,
				fields: {
					name: ({ sequence }) => `User ${sequence}`,
					email: ({ sequence }) => `user${sequence}@example.com`,
				},
			});

			type PostTraits = { withAuthor: never };
			type PostAugmentations = { withAuthor: { author: { id: number; name: string } } };

			const postFixture = createFixture<typeof posts, PostTraits, PostAugmentations>({
				table: posts,
				fields: {
					title: 'Test Post',
					content: 'Content',
				},
				traits: {
					withAuthor: {
						afterMake: async ({ use }) => {
							const author = await use(userFixture).create();
							return { author: { id: author.id, name: author.name } };
						},
					},
				},
			});

			const post = await postFixture(mockDbWithUser).trait('withAuthor').create();

			expect(post.title).toBe('Test Post');
			expect(post.author.id).toBe(1);
			expect(post.author.name).toBe('Created User');
		});

		it('global afterMake can use use(fixture).build()', async () => {
			const userFixture = createFixture({
				table: users,
				fields: {
					name: ({ sequence }) => `User ${sequence}`,
					email: ({ sequence }) => `user${sequence}@example.com`,
				},
			});

			let relatedUser: { id: number | undefined; name: string } | null = null;

			const postFixture = createFixture({
				table: posts,
				fields: {
					title: 'Test Post',
					content: 'Content',
				},
				hooks: {
					afterMake: async ({ use }) => {
						const user = await use(userFixture).build();
						relatedUser = { id: user.id, name: user.name };
					},
				},
			});

			await postFixture(mockDb).build();

			expect(relatedUser).not.toBeNull();
			expect(relatedUser!.name).toBe('User 2');
		});
	});

	describe('Circular Dependency Detection', () => {
		it('direct cycle A -> A throws error with circular dependency message', async () => {
			// Create self-referential fixture for circular dependency test
			const selfRefFixture: FixtureFunction<typeof users, any, any> = createFixture({
				table: users,
				fields: {
					name: async ({ use }) => {
						const other = await use(selfRefFixture).build();
						return `Ref to ${other.name}`;
					},
					email: 'test@example.com',
				},
			});

			await expect(selfRefFixture(mockDb).build()).rejects.toThrow(FieldResolverError);
			await expect(selfRefFixture(mockDb).build()).rejects.toThrow('Circular dependency detected');
		});

		it('indirect cycle A -> B -> A throws with full chain in message', async () => {
			// Create wrapper object to enable circular reference
			const fixtures: {
				user?: FixtureFunction<typeof users, any, any>;
			} = {};

			const postFixture = createFixture({
				table: posts,
				fields: {
					title: 'Post',
					content: 'Content',
					userId: async ({ use }) => {
						const user = await use(fixtures.user!).build();
						return user.id;
					},
				},
			});

			fixtures.user = createFixture({
				table: users,
				fields: {
					name: async ({ use }) => {
						const post = await use(postFixture).build();
						return `User for ${post.title}`;
					},
					email: 'test@example.com',
				},
			});

			await expect(fixtures.user(mockDb).build()).rejects.toThrow('Circular dependency detected');

			try {
				await fixtures.user(mockDb).build();
			} catch (error) {
				expect((error as Error).message).toContain('users');
				expect((error as Error).message).toContain('posts');
			}
		});

		it('triple cycle A -> B -> C -> A shows all three in message', async () => {
			// Create wrapper object to enable circular reference
			const fixtures: {
				a?: FixtureFunction<typeof users, any, any>;
			} = {};

			const fixtureB = createFixture({
				table: posts,
				fields: {
					title: 'B',
					content: 'Content B',
				},
			});

			const fixtureC = createFixture({
				table: posts,
				fields: {
					title: async ({ use }) => {
						const a = await use(fixtures.a!).build();
						return `C refs ${a.name}`;
					},
					content: 'Content C',
				},
			});

			fixtures.a = createFixture({
				table: users,
				fields: {
					name: async ({ use }) => {
						await use(fixtureB).build();
						const c = await use(fixtureC).build();
						return `A refs ${c.title}`;
					},
					email: 'a@example.com',
				},
			});

			await expect(fixtures.a(mockDb).build()).rejects.toThrow('Circular dependency detected');
		});

		it('error message includes table names (not "anonymous")', async () => {
			// Create self-referential fixture for circular dependency test
			const selfRefFixture: FixtureFunction<typeof users, any, any> = createFixture({
				table: users,
				fields: {
					name: async ({ use }) => {
						const other = await use(selfRefFixture).build();
						return `Ref to ${other.name}`;
					},
					email: 'test@example.com',
				},
			});

			try {
				await selfRefFixture(mockDb).build();
			} catch (error) {
				expect((error as Error).message).not.toContain('anonymous');
				expect((error as Error).message).toContain('users');
			}
		});
	});

	describe('Deep Nesting', () => {
		it('3-level nesting A -> B -> C works', async () => {
			const fixtureC = createFixture({
				table: users,
				fields: {
					name: 'Level C',
					email: 'c@example.com',
				},
			});

			const fixtureB = createFixture({
				table: posts,
				fields: {
					title: async ({ use }) => {
						const c = await use(fixtureC).build();
						return `B sees ${c.name}`;
					},
					content: 'B content',
				},
			});

			const fixtureA = createFixture({
				table: users,
				fields: {
					name: async ({ use }) => {
						const b = await use(fixtureB).build();
						return `A sees ${b.title}`;
					},
					email: 'a@example.com',
				},
			});

			const result = await fixtureA(mockDb).build();

			expect(result.name).toBe('A sees B sees Level C');
		});

		it('multiple children at same level work', async () => {
			const childA = createFixture({
				table: users,
				fields: {
					name: 'Child A',
					email: 'child-a@example.com',
				},
			});

			const childB = createFixture({
				table: users,
				fields: {
					name: 'Child B',
					email: 'child-b@example.com',
				},
			});

			const parent = createFixture({
				table: posts,
				fields: {
					title: async ({ use }) => {
						const a = await use(childA).build();
						const b = await use(childB).build();
						return `${a.name} & ${b.name}`;
					},
					content: 'Parent',
				},
			});

			const result = await parent(mockDb).build();

			expect(result.title).toBe('Child A & Child B');
		});
	});

	describe('Shared State', () => {
		it('sequence counter increments across nested fixtures', async () => {
			const sequences: number[] = [];

			const inner = createFixture({
				table: users,
				fields: {
					name: ({ sequence }) => {
						sequences.push(sequence);
						return `Inner ${sequence}`;
					},
					email: 'inner@example.com',
				},
			});

			const outer = createFixture({
				table: posts,
				fields: {
					title: async ({ sequence, use }) => {
						sequences.push(sequence);
						const innerResult = await use(inner).build();
						return `Outer ${sequence} -> ${innerResult.name}`;
					},
					content: 'Content',
				},
			});

			await outer(mockDb).build();

			expect(sequences).toEqual([1, 2]);
			expect(sequences[0]).toBe(1);
			expect(sequences[1]).toBe(2);
		});

		it('same db instance used throughout composition', async () => {
			const dbInstances: DrizzleDatabase[] = [];

			type UserTraits = { tracking: never };
			type UserAugmentations = { tracking: { tracked: boolean } };

			const userFixture = createFixture<typeof users, UserTraits, UserAugmentations>({
				table: users,
				fields: {
					name: 'User',
					email: 'user@example.com',
				},
				traits: {
					tracking: {
						afterMake: ({ db }) => {
							dbInstances.push(db);
							return { tracked: true };
						},
					},
				},
			});

			type PostTraits = { withTrackedUser: never };
			type PostAugmentations = { withTrackedUser: { trackedUser: boolean } };

			const postFixture = createFixture<typeof posts, PostTraits, PostAugmentations>({
				table: posts,
				fields: {
					title: 'Post',
					content: 'Content',
				},
				traits: {
					withTrackedUser: {
						afterMake: async ({ db, use }) => {
							dbInstances.push(db);
							await use(userFixture).trait('tracking').build();
							return { trackedUser: true };
						},
					},
				},
			});

			await postFixture(mockDb).trait('withTrackedUser').build();

			expect(dbInstances.length).toBe(2);
			expect(dbInstances[0]).toBe(dbInstances[1]);
			expect(dbInstances[0]).toBe(mockDb);
		});
	});

	describe('Edge Cases', () => {
		it('multiple use() calls in same field resolver create separate entities', async () => {
			let callCount = 0;

			const userFixture = createFixture({
				table: users,
				fields: {
					name: ({ sequence }) => {
						callCount++;
						return `User ${sequence}`;
					},
					email: ({ sequence }) => `user${sequence}@example.com`,
				},
			});

			const postFixture = createFixture({
				table: posts,
				fields: {
					title: async ({ use }) => {
						const user1 = await use(userFixture).build();
						const user2 = await use(userFixture).build();
						return `${user1.name} and ${user2.name}`;
					},
					content: 'Content',
				},
			});

			const result = await postFixture(mockDb).build();

			expect(callCount).toBe(2);
			expect(result.title).toBe('User 2 and User 3');
		});

		it('use() in trait without params works', async () => {
			const userFixture = createFixture({
				table: users,
				fields: {
					name: 'User',
					email: 'user@example.com',
				},
			});

			type PostTraits = { withUser: never };
			type PostAugmentations = { withUser: { user: { name: string } } };

			const postFixture = createFixture<typeof posts, PostTraits, PostAugmentations>({
				table: posts,
				fields: {
					title: 'Post',
					content: 'Content',
				},
				traits: {
					withUser: {
						afterMake: async ({ use }) => {
							const user = await use(userFixture).build();
							return { user: { name: user.name } };
						},
					},
				},
			});

			const result = await postFixture(mockDb).trait('withUser').build();

			expect(result.user.name).toBe('User');
		});

		it('use() with trait chaining works', async () => {
			type UserTraits = { admin: never };

			const userFixture = createFixture<typeof users, UserTraits>({
				table: users,
				fields: {
					name: 'User',
					email: 'user@example.com',
					role: 'user',
				},
				traits: {
					admin: {
						fields: {
							role: 'admin',
						},
					},
				},
			});

			const postFixture = createFixture({
				table: posts,
				fields: {
					title: async ({ use }) => {
						const admin = await use(userFixture).trait('admin').build();
						return `Post by ${admin.role}`;
					},
					content: 'Content',
				},
			});

			const result = await postFixture(mockDb).build();

			expect(result.title).toBe('Post by admin');
		});

		it('diamond dependency pattern does not throw (shared descendant)', async () => {
			const base = createFixture({
				table: users,
				fields: {
					name: 'Base',
					email: 'base@example.com',
				},
			});

			const leftChild = createFixture({
				table: posts,
				fields: {
					title: async ({ use }) => {
						const b = await use(base).build();
						return `Left: ${b.name}`;
					},
					content: 'Left',
				},
			});

			const rightChild = createFixture({
				table: posts,
				fields: {
					title: async ({ use }) => {
						const b = await use(base).build();
						return `Right: ${b.name}`;
					},
					content: 'Right',
				},
			});

			const parent = createFixture({
				table: users,
				fields: {
					name: async ({ use }) => {
						const left = await use(leftChild).build();
						const right = await use(rightChild).build();
						return `${left.title} | ${right.title}`;
					},
					email: 'parent@example.com',
				},
			});

			const result = await parent(mockDb).build();

			expect(result.name).toBe('Left: Base | Right: Base');
		});

		it('separate fixture calls have independent sequences', async () => {
			const userFixture = createFixture({
				table: users,
				fields: {
					name: ({ sequence }) => `User ${sequence}`,
					email: 'user@example.com',
				},
			});

			const resultA = await userFixture(mockDb).build();
			const resultB = await userFixture(mockDb).build();

			expect(resultA.name).toBe('User 1');
			expect(resultB.name).toBe('User 1');
		});
	});
});
