import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { Mock } from 'vitest';

import { users, posts } from '../fixtures/test-schema';
import type { DrizzleDatabase } from '../types';
import { composeFactory } from './compose-factory';
import { createFixture } from './create-fixture';

// Mock database that supports insert operations
function createMockDb(): DrizzleDatabase & {
	insert: Mock;
	_insert: Mock;
	_values: Mock;
	_returning: Mock;
} {
	const _returning = vi
		.fn()
		.mockResolvedValue([{ id: 1, name: 'Test', email: 'test@example.com' }]);
	const _values = vi.fn().mockReturnValue({ returning: _returning });
	const insert = vi.fn().mockReturnValue({ values: _values });

	return {
		insert,
		_insert: insert,
		_values,
		_returning,
	} as unknown as DrizzleDatabase & {
		insert: Mock;
		_insert: Mock;
		_values: Mock;
		_returning: Mock;
	};
}

describe('composeFactory()', () => {
	let mockDb: ReturnType<typeof createMockDb>;

	const userFixture = createFixture({
		table: users,
		fields: {
			id: ({ sequence }) => sequence,
			name: 'Test User',
			email: ({ sequence }) => `user${sequence}@example.com`,
		},
	});

	const postFixture = createFixture({
		table: posts,
		fields: {
			id: ({ sequence }) => sequence,
			title: 'Test Post',
			userId: 1,
		},
	});

	beforeEach(() => {
		mockDb = createMockDb();
	});

	describe('basic composition', () => {
		it('returns a function', () => {
			const factory = composeFactory({ user: userFixture });
			expect(typeof factory).toBe('function');
		});

		it('factory(db) returns bound factory with fixture accessors', () => {
			const factory = composeFactory({ user: userFixture, post: postFixture });
			const session = factory(mockDb);

			expect(session.user).toBeDefined();
			expect(session.post).toBeDefined();
			expect(typeof session.user.create).toBe('function');
			expect(typeof session.post.create).toBe('function');
		});

		it('each fixture access returns a fresh builder', () => {
			const factory = composeFactory({ user: userFixture });
			const session = factory(mockDb);

			const builder1 = session.user;
			const builder2 = session.user;

			// Each access creates a new builder (due to getter)
			expect(builder1).not.toBe(builder2);
		});
	});

	describe('database operations', () => {
		it('create() inserts into database', async () => {
			const factory = composeFactory({ user: userFixture });
			const session = factory(mockDb);

			await session.user.create();

			expect(mockDb._insert).toHaveBeenCalledWith(users);
		});

		it('multiple creates insert multiple records', async () => {
			const factory = composeFactory({ user: userFixture, post: postFixture });
			const session = factory(mockDb);

			await session.user.create();
			await session.user.create();
			await session.post.create();

			expect(mockDb._insert).toHaveBeenCalledTimes(3);
		});
	});

	describe('session binding', () => {
		it('each factory(db) call creates independent session', async () => {
			const factory = composeFactory({ user: userFixture });
			const sessionA = factory(mockDb);
			const sessionB = factory(mockDb);

			// Both sessions reference the same db
			await sessionA.user.create();
			await sessionB.user.create();

			// Both inserts should use the same mock db
			expect(mockDb._insert).toHaveBeenCalledTimes(2);
		});

		it('factory can be bound to different databases', () => {
			const factory = composeFactory({ user: userFixture });
			const mockDb2 = createMockDb();

			const sessionA = factory(mockDb);
			const sessionB = factory(mockDb2);

			// Both sessions exist independently
			expect(sessionA.user).toBeDefined();
			expect(sessionB.user).toBeDefined();
		});
	});

	describe('createList() operations', () => {
		it('createList(3) creates 3 entities', async () => {
			const factory = composeFactory({ user: userFixture });
			const session = factory(mockDb);

			let insertCount = 0;
			mockDb._returning = vi.fn().mockImplementation(() => {
				insertCount++;
				return Promise.resolve([
					{ id: insertCount, name: 'Test', email: `test${insertCount}@example.com` },
				]);
			});

			const users = await session.user.createList(3);

			expect(users).toHaveLength(3);
			expect(mockDb._insert).toHaveBeenCalledTimes(3);
		});
	});

	describe('with traits', () => {
		it('traits work correctly through composed factory', async () => {
			type UserTraits = { admin: never };

			const adminUserFixture = createFixture<typeof users, UserTraits>({
				table: users,
				fields: {
					id: ({ sequence }) => sequence,
					name: 'Test User',
					email: 'test@example.com',
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

			const factory = composeFactory({ user: adminUserFixture });
			const session = factory(mockDb);

			await session.user.trait('admin').create();

			expect(mockDb._values).toHaveBeenCalledWith(
				expect.objectContaining({
					role: 'admin',
				})
			);
		});
	});

	describe('edge cases', () => {
		it('empty fixtures map works', () => {
			const factory = composeFactory({});
			const session = factory(mockDb);

			expect(session).toBeDefined();
		});

		it('works with single fixture', async () => {
			const factory = composeFactory({ user: userFixture });
			const session = factory(mockDb);

			const user = await session.user.create();

			expect(user).toBeDefined();
			expect(mockDb._insert).toHaveBeenCalledTimes(1);
		});

		it('works with many fixtures', () => {
			const fixture1 = createFixture({
				table: users,
				fields: { id: 1, name: 'A', email: 'a@a.com' },
			});
			const fixture2 = createFixture({
				table: users,
				fields: { id: 2, name: 'B', email: 'b@b.com' },
			});
			const fixture3 = createFixture({
				table: users,
				fields: { id: 3, name: 'C', email: 'c@c.com' },
			});

			const factory = composeFactory({
				user1: fixture1,
				user2: fixture2,
				user3: fixture3,
			});
			const session = factory(mockDb);

			expect(session.user1).toBeDefined();
			expect(session.user2).toBeDefined();
			expect(session.user3).toBeDefined();
		});
	});

	describe('transaction pattern', () => {
		it('factory accepts transaction (tx) just like regular db', () => {
			// Transaction objects have the same interface as db
			const factory = composeFactory({ user: userFixture });
			const mockTx = createMockDb(); // Same interface as db

			const session = factory(mockTx);

			expect(session.user).toBeDefined();
		});

		it('session bound to tx uses tx for all operations', async () => {
			const factory = composeFactory({ user: userFixture });
			const mockTx = createMockDb();

			const session = factory(mockTx);
			await session.user.create();

			// Insert uses the tx, not some other connection
			expect(mockTx._insert).toHaveBeenCalledWith(users);
		});
	});
});
