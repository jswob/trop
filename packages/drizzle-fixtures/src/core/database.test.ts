import { describe, it, expect, vi } from 'vitest';
import type { Mock } from 'vitest';

import { users } from '../fixtures/test-schema';
import type { DrizzleDatabase, FixtureConfig } from '../types';
import { DatabaseOperationError } from '../utils/errors';
import { FixtureBuilder } from './fixture-builder';
import { SequenceCounter } from './sequence';

// Mock database for testing
function createMockDb(insertResult: unknown[] = []): DrizzleDatabase & {
	insert: Mock;
	_insert: Mock;
	_values: Mock;
	_returning: Mock;
} {
	const _returning = vi.fn().mockResolvedValue(insertResult);
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

describe('Database Integration', () => {
	describe('create()', () => {
		it('inserts record and returns data from .returning()', async () => {
			const insertedData = {
				id: 42,
				name: 'Test User',
				email: 'test@example.com',
				role: 'user',
				createdAt: new Date('2024-01-01'),
			};

			const mockDb = createMockDb([insertedData]);

			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: ({ sequence }) => sequence,
					name: 'Test User',
					email: 'test@example.com',
					role: 'user',
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const result = await builder.create();

			expect(mockDb._insert).toHaveBeenCalledWith(users);
			expect(mockDb._values).toHaveBeenCalledWith({
				id: 1,
				name: 'Test User',
				email: 'test@example.com',
				role: 'user',
			});
			expect(mockDb._returning).toHaveBeenCalled();
			expect(result).toEqual(insertedData);
		});

		it('calls beforeMake with mode: "create"', async () => {
			const beforeMakeSpy = vi.fn();
			const mockDb = createMockDb([{ id: 1, name: 'Test', email: 'test@example.com' }]);

			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test',
					email: 'test@example.com',
				},
				hooks: {
					beforeMake: beforeMakeSpy,
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			await builder.create();

			expect(beforeMakeSpy).toHaveBeenCalledWith({ mode: 'create' });
		});

		it('calls afterMake with mode: "create" and pre-INSERT data', async () => {
			const afterMakeSpy = vi.fn();
			const mockDb = createMockDb([{ id: 42, name: 'Test', email: 'test@example.com' }]);

			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: ({ sequence }) => sequence,
					name: 'Test',
					email: 'test@example.com',
				},
				hooks: {
					afterMake: afterMakeSpy,
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			await builder.create();

			expect(afterMakeSpy).toHaveBeenCalledWith(
				expect.objectContaining({
					mode: 'create',
					// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment -- Reason: Vitest expect.objectContaining returns any type
					data: expect.objectContaining({
						id: 1, // Pre-INSERT data (sequence value, not DB-returned 42)
						name: 'Test',
						email: 'test@example.com',
					}),
					// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment -- Reason: Vitest expect.any returns any type
					use: expect.any(Function),
				})
			);
		});

		it('calls beforeCreate before insert', async () => {
			const callOrder: string[] = [];
			const beforeCreateSpy = vi.fn().mockImplementation(() => {
				callOrder.push('beforeCreate');
			});
			const _returning = vi.fn().mockImplementation(() => {
				callOrder.push('insert');
				return Promise.resolve([{ id: 1, name: 'Test', email: 'test@example.com' }]);
			});
			const _values = vi.fn().mockReturnValue({ returning: _returning });
			const insert = vi.fn().mockReturnValue({ values: _values });

			const mockDb = { insert, _values, _returning } as unknown as DrizzleDatabase;

			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test',
					email: 'test@example.com',
				},
				hooks: {
					beforeCreate: beforeCreateSpy,
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			await builder.create();

			expect(callOrder).toEqual(['beforeCreate', 'insert']);
			expect(beforeCreateSpy).toHaveBeenCalledWith({
				// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment -- Reason: Vitest expect.objectContaining returns any type
				data: expect.objectContaining({
					id: 1,
					name: 'Test',
					email: 'test@example.com',
				}),
			});
		});

		it('calls afterCreate after insert with post-INSERT data', async () => {
			const callOrder: string[] = [];
			const afterCreateSpy = vi.fn().mockImplementation(() => {
				callOrder.push('afterCreate');
			});
			const insertedData = { id: 42, name: 'Test', email: 'test@example.com', role: 'admin' };
			const _returning = vi.fn().mockImplementation(() => {
				callOrder.push('insert');
				return Promise.resolve([insertedData]);
			});
			const _values = vi.fn().mockReturnValue({ returning: _returning });
			const insert = vi.fn().mockReturnValue({ values: _values });

			const mockDb = { insert, _values, _returning } as unknown as DrizzleDatabase;

			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test',
					email: 'test@example.com',
				},
				hooks: {
					afterCreate: afterCreateSpy,
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			await builder.create();

			expect(callOrder).toEqual(['insert', 'afterCreate']);
			expect(afterCreateSpy).toHaveBeenCalledWith({
				data: insertedData,
			});
		});

		it('trait afterMake receives post-INSERT data (not resolved data)', async () => {
			let receivedData: unknown;

			type UserTraits = { withCapture: never };
			type UserAugmentations = { withCapture: { captured: true } };

			const insertedData = { id: 42, name: 'Test', email: 'test@example.com' };
			const mockDb = createMockDb([insertedData]);

			const config: FixtureConfig<typeof users, UserTraits, UserAugmentations> = {
				table: users,
				fields: {
					id: ({ sequence }) => sequence, // Resolver returns 1
					name: 'Test',
					email: 'test@example.com',
				},
				traits: {
					withCapture: {
						afterMake: ({ data }) => {
							receivedData = data;
							return { captured: true };
						},
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			await builder.trait('withCapture').create();

			// Trait afterMake should receive DB-returned id (42), not resolver's sequence value (1)
			expect(receivedData).toHaveProperty('id', 42);
		});

		it('with trait applies trait fields before insert', async () => {
			type UserTraits = { admin: never };

			const insertedData = { id: 1, name: 'Test', email: 'test@example.com', role: 'admin' };
			const mockDb = createMockDb([insertedData]);

			const config: FixtureConfig<typeof users, UserTraits> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test',
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
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			await builder.trait('admin').create();

			expect(mockDb._values).toHaveBeenCalledWith(
				expect.objectContaining({
					role: 'admin',
				})
			);
		});

		it('applies overrides correctly', async () => {
			const insertedData = {
				id: 1,
				name: 'Override Name',
				email: 'custom@example.com',
				role: 'user',
			};
			const mockDb = createMockDb([insertedData]);

			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: 1,
					name: 'Default Name',
					email: 'default@example.com',
					role: 'user',
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			await builder.create({
				name: 'Override Name',
				email: 'custom@example.com',
			});

			expect(mockDb._values).toHaveBeenCalledWith(
				expect.objectContaining({
					name: 'Override Name',
					email: 'custom@example.com',
				})
			);
		});

		it('wraps database error in DatabaseOperationError', async () => {
			const dbError = new Error('Connection refused');
			const _returning = vi.fn().mockRejectedValue(dbError);
			const _values = vi.fn().mockReturnValue({ returning: _returning });
			const insert = vi.fn().mockReturnValue({ values: _values });

			const mockDb = { insert, _values, _returning } as unknown as DrizzleDatabase;

			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test',
					email: 'test@example.com',
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());

			await expect(builder.create()).rejects.toThrow(DatabaseOperationError);
			await expect(builder.create()).rejects.toThrow('Database insert on "users" failed');
		});
	});

	describe('createList()', () => {
		it('creates specified number of records', async () => {
			let insertCount = 0;
			const _returning = vi.fn().mockImplementation(() => {
				insertCount++;
				return Promise.resolve([
					{ id: insertCount, name: 'Test', email: `test${insertCount}@example.com` },
				]);
			});
			const _values = vi.fn().mockReturnValue({ returning: _returning });
			const insert = vi.fn().mockReturnValue({ values: _values });

			const mockDb = { insert, _values, _returning } as unknown as DrizzleDatabase;

			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: ({ sequence }) => sequence,
					name: 'Test',
					email: ({ sequence }) => `test${sequence}@example.com`,
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const results = await builder.createList(3);

			expect(results).toHaveLength(3);
			expect(insert).toHaveBeenCalledTimes(3);
		});

		it('each item has different sequence number', async () => {
			const insertedValues: unknown[] = [];
			const _returning = vi.fn().mockImplementation(() => {
				const lastValue = insertedValues[insertedValues.length - 1] as { id: number };
				return Promise.resolve([{ ...lastValue, email: `test${lastValue.id}@example.com` }]);
			});
			const _values = vi.fn().mockImplementation((data: unknown) => {
				insertedValues.push(data);
				return { returning: _returning };
			});
			const insert = vi.fn().mockReturnValue({ values: _values });

			const mockDb = { insert, _values, _returning } as unknown as DrizzleDatabase;

			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: ({ sequence }) => sequence,
					name: ({ sequence }) => `User ${sequence}`,
					email: ({ sequence }) => `test${sequence}@example.com`,
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			await builder.createList(3);

			expect(insertedValues).toHaveLength(3);
			expect(insertedValues[0]).toHaveProperty('id', 1);
			expect(insertedValues[1]).toHaveProperty('id', 2);
			expect(insertedValues[2]).toHaveProperty('id', 3);
			expect(insertedValues[0]).toHaveProperty('name', 'User 1');
			expect(insertedValues[1]).toHaveProperty('name', 'User 2');
			expect(insertedValues[2]).toHaveProperty('name', 'User 3');
		});

		it('returns empty array for count 0', async () => {
			const mockDb = createMockDb([]);

			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: 1,
					name: 'Test',
					email: 'test@example.com',
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const results = await builder.createList(0);

			expect(results).toEqual([]);
			expect(mockDb._insert).not.toHaveBeenCalled();
		});

		it('applies same overrides to all items', async () => {
			const insertedValues: unknown[] = [];
			const _returning = vi.fn().mockImplementation(() => {
				const lastValue = insertedValues[insertedValues.length - 1] as { id: number };
				return Promise.resolve([{ ...lastValue }]);
			});
			const _values = vi.fn().mockImplementation((data: unknown) => {
				insertedValues.push(data);
				return { returning: _returning };
			});
			const insert = vi.fn().mockReturnValue({ values: _values });

			const mockDb = { insert, _values, _returning } as unknown as DrizzleDatabase;

			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: ({ sequence }) => sequence,
					name: 'Default',
					email: 'default@example.com',
					role: 'user',
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			await builder.createList(2, { role: 'admin' });

			expect(insertedValues[0]).toHaveProperty('role', 'admin');
			expect(insertedValues[1]).toHaveProperty('role', 'admin');
		});

		it('calls hooks for each item', async () => {
			const beforeMakeCalls: number[] = [];
			let callCount = 0;

			const _returning = vi.fn().mockImplementation(() => {
				callCount++;
				return Promise.resolve([
					{ id: callCount, name: 'Test', email: `test${callCount}@example.com` },
				]);
			});
			const _values = vi.fn().mockReturnValue({ returning: _returning });
			const insert = vi.fn().mockReturnValue({ values: _values });

			const mockDb = { insert, _values, _returning } as unknown as DrizzleDatabase;

			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: ({ sequence }) => sequence,
					name: 'Test',
					email: 'test@example.com',
				},
				hooks: {
					beforeMake: () => {
						beforeMakeCalls.push(beforeMakeCalls.length + 1);
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			await builder.createList(3);

			expect(beforeMakeCalls).toEqual([1, 2, 3]);
		});
	});
});
