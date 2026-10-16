import { describe, it, expect, vi } from 'vitest';
import { createFixture } from '../core/create-fixture';
import { users } from '../fixtures/test-schema';
import type { DrizzleDatabase } from '../types';
import { HookExecutionError } from '../utils/errors';
import { executeHook } from './hook-executor';

const mockDb = {} as DrizzleDatabase;

describe('executeHook', () => {
	it('executes sync hook', async () => {
		const hook = vi.fn();
		await executeHook('testHook', hook, { mode: 'build' });

		expect(hook).toHaveBeenCalledWith({ mode: 'build' });
	});

	it('executes async hook', async () => {
		const hook = vi.fn().mockResolvedValue(undefined);
		await executeHook('testHook', hook, { data: 'test' });

		expect(hook).toHaveBeenCalledWith({ data: 'test' });
	});

	it('is no-op when hook is undefined', async () => {
		await expect(executeHook('testHook', undefined, {})).resolves.toBeUndefined();
	});

	it('wraps error in HookExecutionError', async () => {
		const hook = vi.fn().mockImplementation(() => {
			throw new Error('Hook failed');
		});

		await expect(executeHook('testHook', hook, {})).rejects.toThrow(HookExecutionError);
		await expect(executeHook('testHook', hook, {})).rejects.toThrow('Hook "testHook" failed');
	});

	it('wraps non-Error throws in HookExecutionError', async () => {
		const hook = vi.fn().mockImplementation(() => {
			// eslint-disable-next-line @typescript-eslint/only-throw-error -- Reason: Testing error handling - intentionally throwing string to verify non-Error exception wrapping
			throw 'string error';
		});

		await expect(executeHook('testHook', hook, {})).rejects.toThrow(HookExecutionError);
	});
});

describe('Lifecycle Hooks in FixtureBuilder', () => {
	describe('beforeMake hook', () => {
		it('is called before field resolution with correct mode', async () => {
			const callOrder: string[] = [];
			const beforeMake = vi.fn().mockImplementation(() => {
				callOrder.push('beforeMake');
			});

			const userFixture = createFixture({
				table: users,
				fields: {
					id: () => {
						callOrder.push('resolver');
						return 1;
					},
					name: 'Test',
					email: 'test@example.com',
				},
				hooks: {
					beforeMake,
				},
			});

			await userFixture(mockDb).build();

			expect(beforeMake).toHaveBeenCalledWith({ mode: 'build' });
			expect(callOrder).toEqual(['beforeMake', 'resolver']);
		});

		it('can be async', async () => {
			let beforeMakeCalled = false;
			const beforeMake = vi.fn().mockImplementation(async () => {
				await new Promise(r => setTimeout(r, 10));
				beforeMakeCalled = true;
			});

			const userFixture = createFixture({
				table: users,
				fields: {
					id: () => {
						expect(beforeMakeCalled).toBe(true);
						return 1;
					},
					name: 'Test',
					email: 'test@example.com',
				},
				hooks: {
					beforeMake,
				},
			});

			await userFixture(mockDb).build();

			expect(beforeMake).toHaveBeenCalled();
		});

		it('does not receive data (it is before resolution)', async () => {
			const beforeMake = vi.fn();

			const userFixture = createFixture({
				table: users,
				fields: {
					id: 1,
					name: 'Test',
					email: 'test@example.com',
				},
				hooks: {
					beforeMake,
				},
			});

			await userFixture(mockDb).build();

			expect(beforeMake).toHaveBeenCalledWith({ mode: 'build' });
			expect(beforeMake.mock.calls[0]?.[0]).not.toHaveProperty('data');
		});
	});

	describe('afterMake hook', () => {
		it('is called after field resolution', async () => {
			const callOrder: string[] = [];
			const afterMake = vi.fn().mockImplementation(() => {
				callOrder.push('afterMake');
			});

			const userFixture = createFixture({
				table: users,
				fields: {
					id: () => {
						callOrder.push('resolver');
						return 1;
					},
					name: 'Test',
					email: 'test@example.com',
				},
				hooks: {
					afterMake,
				},
			});

			await userFixture(mockDb).build();

			expect(callOrder).toEqual(['resolver', 'afterMake']);
		});

		it('receives resolved data and mode', async () => {
			const afterMake = vi.fn();

			const userFixture = createFixture({
				table: users,
				fields: {
					id: 1,
					name: 'Test User',
					email: 'test@example.com',
				},
				hooks: {
					afterMake,
				},
			});

			await userFixture(mockDb).build();

			expect(afterMake).toHaveBeenCalledWith(
				expect.objectContaining({
					// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment -- Reason: Vitest expect.objectContaining returns any type
					data: expect.objectContaining({
						id: 1,
						name: 'Test User',
						email: 'test@example.com',
					}),
					mode: 'build',
					// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment -- Reason: Vitest expect.any returns any type
					use: expect.any(Function),
				})
			);
		});

		it('receives frozen data (cannot mutate)', async () => {
			const afterMake = vi.fn().mockImplementation(({ data }) => {
				expect(Object.isFrozen(data)).toBe(true);
			});

			const userFixture = createFixture({
				table: users,
				fields: {
					id: 1,
					name: 'Test',
					email: 'test@example.com',
				},
				hooks: {
					afterMake,
				},
			});

			await userFixture(mockDb).build();

			expect(afterMake).toHaveBeenCalled();
		});

		it('can be async', async () => {
			const afterMake = vi.fn().mockImplementation(async () => {
				await new Promise(r => setTimeout(r, 10));
			});

			const userFixture = createFixture({
				table: users,
				fields: {
					id: 1,
					name: 'Test',
					email: 'test@example.com',
				},
				hooks: {
					afterMake,
				},
			});

			await userFixture(mockDb).build();

			expect(afterMake).toHaveBeenCalled();
		});
	});

	describe('hook execution order', () => {
		it('executes hooks in correct order: beforeMake -> resolve -> afterMake', async () => {
			const callOrder: string[] = [];

			const userFixture = createFixture({
				table: users,
				fields: {
					id: () => {
						callOrder.push('resolve');
						return 1;
					},
					name: 'Test',
					email: 'test@example.com',
				},
				hooks: {
					beforeMake: () => {
						callOrder.push('beforeMake');
					},
					afterMake: () => {
						callOrder.push('afterMake');
					},
				},
			});

			await userFixture(mockDb).build();

			expect(callOrder).toEqual(['beforeMake', 'resolve', 'afterMake']);
		});
	});

	describe('missing hooks', () => {
		it('works when no hooks are defined', async () => {
			const userFixture = createFixture({
				table: users,
				fields: {
					id: 1,
					name: 'Test',
					email: 'test@example.com',
				},
			});

			const result = await userFixture(mockDb).build();

			expect(result.id).toBe(1);
		});

		it('works when only beforeMake is defined', async () => {
			const beforeMake = vi.fn();

			const userFixture = createFixture({
				table: users,
				fields: {
					id: 1,
					name: 'Test',
					email: 'test@example.com',
				},
				hooks: {
					beforeMake,
				},
			});

			const result = await userFixture(mockDb).build();

			expect(result.id).toBe(1);
			expect(beforeMake).toHaveBeenCalled();
		});

		it('works when only afterMake is defined', async () => {
			const afterMake = vi.fn();

			const userFixture = createFixture({
				table: users,
				fields: {
					id: 1,
					name: 'Test',
					email: 'test@example.com',
				},
				hooks: {
					afterMake,
				},
			});

			const result = await userFixture(mockDb).build();

			expect(result.id).toBe(1);
			expect(afterMake).toHaveBeenCalled();
		});
	});

	describe('buildList hook execution', () => {
		it('calls hooks for each item in buildList', async () => {
			const beforeMake = vi.fn();
			const afterMake = vi.fn();

			const userFixture = createFixture({
				table: users,
				fields: {
					id: ({ sequence }) => sequence,
					name: 'Test',
					email: 'test@example.com',
				},
				hooks: {
					beforeMake,
					afterMake,
				},
			});

			await userFixture(mockDb).buildList(3);

			expect(beforeMake).toHaveBeenCalledTimes(3);
			expect(afterMake).toHaveBeenCalledTimes(3);
		});

		it('afterMake receives different data for each item', async () => {
			const afterMakeCalls: unknown[] = [];

			const userFixture = createFixture({
				table: users,
				fields: {
					id: ({ sequence }) => sequence,
					name: ({ sequence }) => `User ${sequence}`,
					email: 'test@example.com',
				},
				hooks: {
					afterMake: ({ data }) => {
						afterMakeCalls.push({ ...data });
					},
				},
			});

			await userFixture(mockDb).buildList(3);

			expect(afterMakeCalls).toHaveLength(3);
			expect(afterMakeCalls[0]).toMatchObject({ id: 1, name: 'User 1' });
			expect(afterMakeCalls[1]).toMatchObject({ id: 2, name: 'User 2' });
			expect(afterMakeCalls[2]).toMatchObject({ id: 3, name: 'User 3' });
		});
	});

	describe('error handling', () => {
		it('throws HookExecutionError when beforeMake fails', async () => {
			const userFixture = createFixture({
				table: users,
				fields: {
					id: 1,
					name: 'Test',
					email: 'test@example.com',
				},
				hooks: {
					beforeMake: () => {
						throw new Error('beforeMake failed');
					},
				},
			});

			await expect(userFixture(mockDb).build()).rejects.toThrow(HookExecutionError);
			await expect(userFixture(mockDb).build()).rejects.toThrow(
				'Hook "beforeMake" failed: beforeMake failed'
			);
		});

		it('throws HookExecutionError when afterMake fails', async () => {
			const userFixture = createFixture({
				table: users,
				fields: {
					id: 1,
					name: 'Test',
					email: 'test@example.com',
				},
				hooks: {
					afterMake: () => {
						throw new Error('afterMake failed');
					},
				},
			});

			await expect(userFixture(mockDb).build()).rejects.toThrow(HookExecutionError);
			await expect(userFixture(mockDb).build()).rejects.toThrow(
				'Hook "afterMake" failed: afterMake failed'
			);
		});

		it('HookExecutionError includes hook name', async () => {
			const userFixture = createFixture({
				table: users,
				fields: {
					id: 1,
					name: 'Test',
					email: 'test@example.com',
				},
				hooks: {
					beforeMake: () => {
						throw new Error('test');
					},
				},
			});

			try {
				await userFixture(mockDb).build();
			} catch (e) {
				expect(e).toBeInstanceOf(HookExecutionError);
				expect((e as HookExecutionError).hookName).toBe('beforeMake');
			}
		});
	});
});
