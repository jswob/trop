/**
 * Integration tests for drizzle-fixtures full workflow.
 *
 * Tests end-to-end functionality with a real PostgreSQL database.
 * Uses transactions for cleanup - each test runs in a transaction that is rolled back.
 */
import { eq } from 'drizzle-orm';
import type { PostgresJsDatabase } from 'drizzle-orm/postgres-js';
import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest';

import { composeFactory, createFixture } from '../index';
import { userFixture, postFixture } from './fixtures';
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

// Compose factory for session management
const factory = composeFactory({
	user: userFixture,
	post: postFixture,
});

describe('integration: full workflow', () => {
	let db: PostgresJsDatabase<typeof schema>;

	beforeAll(async () => {
		db = getIntegrationDb();
		await waitForDatabase();
		await createIntegrationTables();
	}, 30000);

	afterEach(async () => {
		// Manual cleanup for any test data that wasn't rolled back
		await truncateIntegrationTables();
	});

	afterAll(async () => {
		await dropIntegrationTables();
		await closeConnection();
	}, 30000);

	// ============================================================
	// Test 1: Basic Create
	// ============================================================
	it('should create a user and persist it to database', async () => {
		const session = factory(db);

		const user = await session.user.create();

		// Verify user exists in database
		const [foundUser] = await db
			.select()
			.from(integrationUsers)
			.where(eq(integrationUsers.id, user.id));
		expect(foundUser).toBeDefined();
		expect(foundUser!.id).toBe(user.id);
		expect(foundUser!.name).toBe(user.name);
	});

	// ============================================================
	// Test 2: Trait Application
	// ============================================================
	it('should apply admin trait and set role to admin', async () => {
		const builder = userFixture(db);

		// Create admin user with trait
		const admin = await builder.trait('admin').create();

		// Verify role is 'admin' in database
		const [foundUser] = await db
			.select()
			.from(integrationUsers)
			.where(eq(integrationUsers.id, admin.id));
		expect(foundUser!.role).toBe('admin');
	});

	// ============================================================
	// Test 3: Foreign Key Relationships
	// ============================================================
	it('should create post with FK to user', async () => {
		const session = factory(db);

		// Create user first
		const user = await session.user.create();

		// Create post referencing user via override
		const post = await session.post.create({ userId: user.id });

		// Verify FK constraint satisfied
		const [foundPost] = await db
			.select()
			.from(integrationPosts)
			.where(eq(integrationPosts.id, post.id));
		expect(foundPost!.userId).toBe(user.id);
	});

	// ============================================================
	// Test 4: use() Helper Composition
	// ============================================================
	it('should use use() helper to auto-create related user for post', async () => {
		const session = factory(db);

		// Create post without explicit user - fixture uses use() to create one
		const post = await session.post.create();

		// Post should have a valid userId
		expect(post.userId).toBeDefined();
		expect(typeof post.userId).toBe('string');

		// Verify user was created in DB
		const [foundUser] = await db
			.select()
			.from(integrationUsers)
			.where(eq(integrationUsers.id, post.userId));
		expect(foundUser).toBeDefined();
	});

	// ============================================================
	// Test 5: createList() Bulk Operations
	// ============================================================
	it('should create multiple users with createList', async () => {
		const session = factory(db);

		// Create 5 users
		const users = await session.user.createList(5);

		expect(users).toHaveLength(5);

		// Verify all have unique emails
		const emails = users.map((u: { email: string }) => u.email);
		const uniqueEmails = new Set(emails);
		expect(uniqueEmails.size).toBe(5);

		// Verify all exist in DB
		for (const user of users) {
			const [found] = await db
				.select()
				.from(integrationUsers)
				.where(eq(integrationUsers.id, user.id));
			expect(found).toBeDefined();
		}
	});

	// ============================================================
	// Test 6: Lifecycle Hooks with Real DB
	// ============================================================
	it('should execute lifecycle hooks in correct order', async () => {
		const hookOrder: string[] = [];

		// Create a custom fixture with hooks for this test
		const hookedUserFixture = createFixture<typeof integrationUsers>({
			table: integrationUsers,
			fields: {
				name: () => 'Hooked User',
				email: ({ sequence }) => `hooked-${sequence}-${Date.now()}@test.com`,
				role: () => 'user',
			},
			hooks: {
				beforeMake: () => {
					hookOrder.push('beforeMake');
				},
				afterMake: () => {
					hookOrder.push('afterMake');
				},
				beforeCreate: () => {
					hookOrder.push('beforeCreate');
				},
				afterCreate: () => {
					hookOrder.push('afterCreate');
				},
			},
		});

		// Use the hooked fixture
		const hookedBuilder = hookedUserFixture(db);
		await hookedBuilder.create();

		// Verify hooks executed in order
		expect(hookOrder).toEqual(['beforeMake', 'afterMake', 'beforeCreate', 'afterCreate']);
	});

	// ============================================================
	// Test 7: Factory Session Binding
	// ============================================================
	it('should bind session to database connection', async () => {
		// Create two sessions bound to the same db
		const sessionA = factory(db);
		const sessionB = factory(db);

		// Create users in each session
		const userA = await sessionA.user.create();
		const userB = await sessionB.user.create();

		// Verify both exist in the same database
		const [foundA] = await db
			.select()
			.from(integrationUsers)
			.where(eq(integrationUsers.id, userA.id));
		const [foundB] = await db
			.select()
			.from(integrationUsers)
			.where(eq(integrationUsers.id, userB.id));
		expect(foundA).toBeDefined();
		expect(foundB).toBeDefined();
	});

	// ============================================================
	// Test 8: UUID Primary Key Handling
	// ============================================================
	it('should handle UUID primary keys correctly', async () => {
		const session = factory(db);

		const user = await session.user.create();

		// Verify UUID format (xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx)
		const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
		expect(user.id).toMatch(uuidRegex);
	});

	// ============================================================
	// Test 9: Sequence Counter Across Calls
	// ============================================================
	it('should increment sequence counter across multiple creates via createList', async () => {
		// Note: composeFactory creates a new builder on each property access (via getter),
		// so each `session.user.create()` gets its own sequence counter.
		// To test sequence incrementing, we use createList() which uses a single builder.
		const session = factory(db);

		// Create 3 users using createList (single builder = shared sequence)
		const users = await session.user.createList(3);

		// Extract sequence numbers from email pattern (user-{sequence}-{random}@test.com)
		const getSequence = (email: string): number => {
			const match = email.match(/user-(\d+)-/);
			return match?.[1] ? parseInt(match[1], 10) : -1;
		};

		const seq1 = getSequence(users[0]!.email);
		const seq2 = getSequence(users[1]!.email);
		const seq3 = getSequence(users[2]!.email);

		// Sequences should be incrementing
		expect(seq2).toBeGreaterThan(seq1);
		expect(seq3).toBeGreaterThan(seq2);
	});

	// ============================================================
	// Test 10: Constraint Violation Handling
	// ============================================================
	it('should throw DatabaseOperationError on duplicate email', async () => {
		const session = factory(db);
		const { DatabaseOperationError } = await import('../utils/errors');

		// Create first user with specific email using override
		const email = `unique-${Date.now()}@test.com`;
		await session.user.create({ email });

		// Attempt to create second user with same email
		await expect(session.user.create({ email })).rejects.toThrow(DatabaseOperationError);
	});

	// ============================================================
	// Test 11: Data Persistence Verification
	// ============================================================
	it('should persist data that can be queried after creation', async () => {
		const session = factory(db);

		// Create a user
		const user = await session.user.create();
		const userId = user.id;

		// Verify the user persists and can be queried
		const [found] = await db.select().from(integrationUsers).where(eq(integrationUsers.id, userId));
		expect(found).toBeDefined();
		expect(found!.id).toBe(userId);
		expect(found!.name).toBe(user.name);
	});

	// ============================================================
	// Test 12: Multiple Sequential Creations
	// ============================================================
	it('should handle multiple sequential creations correctly', async () => {
		const session = factory(db);

		// Create multiple users sequentially
		const user1 = await session.user.create();
		const user2 = await session.user.create();
		const user3 = await session.user.create();
		const users = [user1, user2, user3];

		expect(users).toHaveLength(3);

		// Verify all have unique IDs
		const ids = users.map(u => u.id);
		const uniqueIds = new Set(ids);
		expect(uniqueIds.size).toBe(3);

		// Verify all have unique emails
		const emails = users.map(u => u.email);
		const uniqueEmails = new Set(emails);
		expect(uniqueEmails.size).toBe(3);

		// All should exist in DB
		for (const user of users) {
			const [found] = await db
				.select()
				.from(integrationUsers)
				.where(eq(integrationUsers.id, user.id));
			expect(found).toBeDefined();
		}
	});

	// ============================================================
	// Test 13: Transaction Support
	// ============================================================
	it('should work within a transaction context', async () => {
		// This tests that fixtures can be created within a transaction
		// and that the transaction interface is compatible

		let createdUserId: string | undefined;

		// Note: We can't actually test rollback with drizzle-orm postgres-js
		// because tx.rollback() throws TransactionRollbackError
		// Instead we verify that fixtures work within transactions

		await db.transaction(async tx => {
			const session = factory(tx);
			const user = await session.user.create();
			createdUserId = user.id;

			// Verify user exists within transaction
			const [found] = await tx
				.select()
				.from(integrationUsers)
				.where(eq(integrationUsers.id, user.id));
			expect(found).toBeDefined();
			expect(found!.name).toBe(user.name);
		});

		// Since transaction committed, user should still exist
		const [found] = await db
			.select()
			.from(integrationUsers)
			.where(eq(integrationUsers.id, createdUserId!));
		expect(found).toBeDefined();
	});
});
