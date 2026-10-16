/**
 * Integration tests that validate README.md code examples work in practice.
 *
 * These tests mirror the examples in the README to ensure documentation
 * stays accurate and code examples actually work.
 */
import { eq } from 'drizzle-orm';
import type { PostgresJsDatabase } from 'drizzle-orm/postgres-js';
import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest';

import { createFixture, composeFactory } from '../index';
import type * as schema from './schema';
import { integrationUsers, integrationPosts } from './schema';
import {
	getIntegrationDb,
	waitForDatabase,
	createIntegrationTables,
	dropIntegrationTables,
	truncateIntegrationTables,
	closeConnection,
} from './setup';

describe('integration: README examples', () => {
	let db: PostgresJsDatabase<typeof schema>;

	beforeAll(async () => {
		db = getIntegrationDb();
		await waitForDatabase();
		await createIntegrationTables();
	}, 30000);

	afterEach(async () => {
		await truncateIntegrationTables();
	});

	afterAll(async () => {
		await dropIntegrationTables();
		await closeConnection();
	}, 30000);

	// ============================================================
	// Basic Fixture Creation (README: Basic Usage section)
	// ============================================================
	describe('Basic fixture creation', () => {
		it('should create fixture with field resolvers', async () => {
			// From README: Define a fixture with field resolvers
			const userFixture = createFixture<typeof integrationUsers>({
				table: integrationUsers,
				fields: {
					name: () => 'Test User',
					email: ({ sequence }) => `user${sequence}@example.com`,
					role: () => 'user',
				},
			});

			// Create in-memory object with defaults
			const user = await userFixture(db).build();

			expect(user.name).toBe('Test User');
			expect(user.email).toMatch(/user\d+@example\.com/);
			expect(user.role).toBe('user');
		});

		it('should allow field overrides', async () => {
			const userFixture = createFixture<typeof integrationUsers>({
				table: integrationUsers,
				fields: {
					name: () => 'Default Name',
					email: ({ sequence }) => `default${sequence}@example.com`,
					role: () => 'user',
				},
			});

			// Override specific fields (from README)
			const customUser = await userFixture(db).build({
				name: 'John Doe',
			});

			expect(customUser.name).toBe('John Doe');
		});

		it('should create and persist to database', async () => {
			const userFixture = createFixture<typeof integrationUsers>({
				table: integrationUsers,
				fields: {
					name: () => 'Persisted User',
					email: ({ sequence }) => `persisted${sequence}-${Date.now()}@example.com`,
					role: () => 'user',
				},
			});

			// Create and persist to database (from README)
			const savedUser = await userFixture(db).create();

			// Verify in database
			const [found] = await db
				.select()
				.from(integrationUsers)
				.where(eq(integrationUsers.id, savedUser.id));
			expect(found).toBeDefined();
			expect(found!.name).toBe('Persisted User');
		});
	});

	// ============================================================
	// Trait Usage (README: Traits section)
	// ============================================================
	describe('Trait usage', () => {
		it('should apply traits using builder pattern', async () => {
			type UserTraits = {
				admin: never;
			};

			const userFixture = createFixture<typeof integrationUsers, UserTraits>({
				table: integrationUsers,
				fields: {
					name: () => 'Regular User',
					email: ({ sequence }) => `trait${sequence}-${Date.now()}@example.com`,
					role: () => 'user',
				},
				traits: {
					admin: {
						fields: {
							role: () => 'admin',
						},
					},
				},
			});

			// Apply traits using builder pattern (from README)
			const adminUser = await userFixture(db).trait('admin').create();

			expect(adminUser.role).toBe('admin');
		});

		it('should combine traits with overrides', async () => {
			type UserTraits = {
				admin: never;
			};

			const userFixture = createFixture<typeof integrationUsers, UserTraits>({
				table: integrationUsers,
				fields: {
					name: () => 'Regular User',
					email: ({ sequence }) => `combined${sequence}-${Date.now()}@example.com`,
					role: () => 'user',
				},
				traits: {
					admin: {
						fields: {
							role: () => 'admin',
						},
					},
				},
			});

			// Combine traits with overrides (from README)
			const complexUser = await userFixture(db).trait('admin').create({ name: 'Custom Admin' });

			expect(complexUser.role).toBe('admin');
			expect(complexUser.name).toBe('Custom Admin');
		});
	});

	// ============================================================
	// composeFactory Usage (README: Factory Composition section)
	// ============================================================
	describe('composeFactory usage', () => {
		it('should compose multiple fixtures', async () => {
			const userFixture = createFixture<typeof integrationUsers>({
				table: integrationUsers,
				fields: {
					name: () => 'Factory User',
					email: ({ sequence }) => `factory${sequence}-${Date.now()}@example.com`,
					role: () => 'user',
				},
			});

			const postFixture = createFixture<typeof integrationPosts>({
				table: integrationPosts,
				fields: {
					title: () => 'Factory Post',
					content: () => 'Content',
					userId: async ({ use }) => {
						const user = await use(userFixture).create();
						return user.id;
					},
					status: () => 'draft',
				},
			});

			// From README: Compose multiple fixtures
			const factory = composeFactory({
				user: userFixture,
				post: postFixture,
			});

			// Use composed factory
			const session = factory(db);
			const user = await session.user.create();
			const post = await session.post.create({ userId: user.id });

			expect(user.name).toBe('Factory User');
			expect(post.title).toBe('Factory Post');
			expect(post.userId).toBe(user.id);
		});
	});

	// ============================================================
	// Lifecycle Hooks (README: Lifecycle Hooks section)
	// ============================================================
	describe('Lifecycle hooks', () => {
		it('should execute hooks in documented order', async () => {
			const hookOrder: string[] = [];

			// From README: Global lifecycle hooks
			const userFixture = createFixture<typeof integrationUsers>({
				table: integrationUsers,
				fields: {
					name: () => 'Hooked User',
					email: ({ sequence }) => `hooked${sequence}-${Date.now()}@example.com`,
					role: () => 'user',
				},
				hooks: {
					beforeMake: ({ mode }) => {
						hookOrder.push(`beforeMake:${mode}`);
					},
					afterMake: ({ mode }) => {
						hookOrder.push(`afterMake:${mode}`);
					},
					beforeCreate: () => {
						hookOrder.push('beforeCreate');
					},
					afterCreate: () => {
						hookOrder.push('afterCreate');
					},
				},
			});

			await userFixture(db).create();

			// Hooks execute in documented order
			expect(hookOrder).toEqual([
				'beforeMake:create',
				'afterMake:create',
				'beforeCreate',
				'afterCreate',
			]);
		});
	});

	// ============================================================
	// createList() Batch Creation (README: API Reference)
	// ============================================================
	describe('createList batch creation', () => {
		it('should create multiple records with createList', async () => {
			const userFixture = createFixture<typeof integrationUsers>({
				table: integrationUsers,
				fields: {
					name: ({ sequence }) => `User ${sequence}`,
					email: ({ sequence }) => `batch${sequence}-${Date.now()}@example.com`,
					role: () => 'user',
				},
			});

			// From README API: Create multiple records
			const users = await userFixture(db).createList(3);

			expect(users).toHaveLength(3);
			// Each gets unique sequence
			expect(users[0]!.name).toBe('User 1');
			expect(users[1]!.name).toBe('User 2');
			expect(users[2]!.name).toBe('User 3');
		});
	});

	// ============================================================
	// use() Helper for Relationships (README: Fixture Composition)
	// ============================================================
	describe('use() helper for relationships', () => {
		it('should compose fixtures with use() helper', async () => {
			const userFixture = createFixture<typeof integrationUsers>({
				table: integrationUsers,
				fields: {
					name: () => 'Author',
					email: ({ sequence }) => `author${sequence}-${Date.now()}@example.com`,
					role: () => 'user',
				},
			});

			// From README: Use another fixture to resolve relationships
			const postFixture = createFixture<typeof integrationPosts>({
				table: integrationPosts,
				fields: {
					title: () => 'My Post',
					content: () => 'Content here',
					// Use another fixture to resolve relationships
					userId: ({ use }) =>
						use(userFixture)
							.create()
							.then(user => user.id),
					status: () => 'draft',
				},
			});

			const post = await postFixture(db).create();

			// Post has a valid userId from the auto-created user
			expect(post.userId).toBeDefined();
			expect(typeof post.userId).toBe('string');

			// Verify the user was created
			const [foundUser] = await db
				.select()
				.from(integrationUsers)
				.where(eq(integrationUsers.id, post.userId));
			expect(foundUser).toBeDefined();
		});
	});

	// ============================================================
	// Transaction Support (README: Transaction-Based Cleanup)
	// ============================================================
	describe('Transaction-based cleanup', () => {
		it('should work with db.transaction() for cleanup', async () => {
			const userFixture = createFixture<typeof integrationUsers>({
				table: integrationUsers,
				fields: {
					name: () => 'Transaction User',
					email: ({ sequence }) => `tx${sequence}-${Date.now()}@example.com`,
					role: () => 'user',
				},
			});

			const factory = composeFactory({
				user: userFixture,
			});

			// From README: Transaction-based cleanup pattern
			// Note: We commit the transaction to verify it works
			// In tests, you would throw an error or call tx.rollback() to cleanup

			let userId: string | undefined;

			await db.transaction(async tx => {
				const session = factory(tx);
				const user = await session.user.create();
				userId = user.id;

				// Verify data visible within transaction
				const [found] = await tx
					.select()
					.from(integrationUsers)
					.where(eq(integrationUsers.id, user.id));
				expect(found).toBeDefined();
			});

			// Transaction committed, data should persist
			const [found] = await db
				.select()
				.from(integrationUsers)
				.where(eq(integrationUsers.id, userId!));
			expect(found).toBeDefined();
		});
	});
});
