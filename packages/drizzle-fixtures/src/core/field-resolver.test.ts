import { pgTable, serial, text, integer } from 'drizzle-orm/pg-core';
import { describe, expect, it, vi } from 'vitest';

import type { FieldResolverContext, UseHelper } from '../types';
import { FieldResolverError } from '../utils/errors';
import { resolveFields, mergeResolvers } from './field-resolver';
import { createSequenceCounter, SequenceCounter } from './sequence';
import { createPlaceholderUseHelper } from './use-placeholder';

// Test table schema - using _testTable to satisfy lint rule (value used as type)
const _testTable = pgTable('test_table', {
	id: serial('id').primaryKey(),
	name: text('name').notNull(),
	email: text('email').notNull(),
	age: integer('age'),
	status: text('status'),
});
type TestTable = typeof _testTable;

type TestModel = {
	id: number;
	name: string;
	email: string;
	age: number | null;
	status: string | null;
};

// Helper to create context
function createContext(sequence: number = 1, use?: UseHelper): FieldResolverContext<TestTable> {
	return {
		use: use ?? createPlaceholderUseHelper(),
		sequence,
	};
}

describe('resolveFields', () => {
	describe('Basic Resolution', () => {
		it('resolves direct values from base resolvers', async () => {
			const result = await resolveFields<TestModel, TestTable>({
				baseResolvers: {
					id: 1,
					name: 'Test User',
					email: 'test@example.com',
				},
				traitResolvers: [],
				overrides: {},
				context: createContext(),
			});

			expect(result).toEqual({
				id: 1,
				name: 'Test User',
				email: 'test@example.com',
			});
		});

		it('resolves function resolvers and returns values', async () => {
			const result = await resolveFields<TestModel, TestTable>({
				baseResolvers: {
					id: () => 42,
					name: ({ sequence }) => `User ${sequence}`,
					email: () => 'generated@example.com',
				},
				traitResolvers: [],
				overrides: {},
				context: createContext(5),
			});

			expect(result).toEqual({
				id: 42,
				name: 'User 5',
				email: 'generated@example.com',
			});
		});

		it('uses direct value resolver (not function) directly', async () => {
			const result = await resolveFields<TestModel, TestTable>({
				baseResolvers: {
					name: 'Static Name',
					age: 25,
				},
				traitResolvers: [],
				overrides: {},
				context: createContext(),
			});

			expect(result).toEqual({
				name: 'Static Name',
				age: 25,
			});
		});
	});

	describe('Override Priority', () => {
		it('direct override skips resolver entirely (verify resolver not called)', async () => {
			const resolverSpy = vi.fn(() => 'from-resolver');

			const result = await resolveFields<TestModel, TestTable>({
				baseResolvers: {
					name: resolverSpy,
					email: 'base@example.com',
				},
				traitResolvers: [],
				overrides: { name: 'Override Name' },
				context: createContext(),
			});

			expect(result.name).toBe('Override Name');
			expect(resolverSpy).not.toHaveBeenCalled();
		});

		it('explicit undefined override uses undefined', async () => {
			const result = await resolveFields<TestModel, TestTable>({
				baseResolvers: {
					name: 'Base Name',
					status: 'active',
				},
				traitResolvers: [],
				overrides: { status: undefined },
				context: createContext(),
			});

			expect(result).toEqual({
				name: 'Base Name',
				status: undefined,
			});
			expect(Object.hasOwn(result, 'status')).toBe(true);
		});

		it('override for field without resolver includes the override', async () => {
			const result = await resolveFields<TestModel, TestTable>({
				baseResolvers: {
					name: 'Base Name',
				},
				traitResolvers: [],
				overrides: { age: 30 },
				context: createContext(),
			});

			expect(result).toEqual({
				name: 'Base Name',
				age: 30,
			});
		});
	});

	describe('Trait Priority', () => {
		it('trait resolver overrides base resolver', async () => {
			const result = await resolveFields<TestModel, TestTable>({
				baseResolvers: {
					name: 'Base Name',
					email: 'base@example.com',
				},
				traitResolvers: [{ name: 'Trait Name' }],
				overrides: {},
				context: createContext(),
			});

			expect(result).toEqual({
				name: 'Trait Name',
				email: 'base@example.com',
			});
		});

		it('multiple traits: last trait wins', async () => {
			const result = await resolveFields<TestModel, TestTable>({
				baseResolvers: {
					name: 'Base Name',
					email: 'base@example.com',
					status: 'base-status',
				},
				traitResolvers: [
					{ name: 'First Trait', status: 'first-status' },
					{ name: 'Second Trait' },
					{ status: 'third-status' },
				],
				overrides: {},
				context: createContext(),
			});

			expect(result).toEqual({
				name: 'Second Trait',
				email: 'base@example.com',
				status: 'third-status',
			});
		});

		it('empty traits array works', async () => {
			const result = await resolveFields<TestModel, TestTable>({
				baseResolvers: {
					name: 'Base Name',
				},
				traitResolvers: [],
				overrides: {},
				context: createContext(),
			});

			expect(result).toEqual({ name: 'Base Name' });
		});
	});

	describe('Memoization', () => {
		it('resolver runs only once per field per build', async () => {
			let callCount = 0;
			const resolver = () => {
				callCount++;
				return `call-${callCount}`;
			};

			const result = await resolveFields<TestModel, TestTable>({
				baseResolvers: {
					name: resolver,
				},
				traitResolvers: [],
				overrides: {},
				context: createContext(),
			});

			expect(result.name).toBe('call-1');
			expect(callCount).toBe(1);
		});
	});

	describe('Async Support', () => {
		it('async resolver works correctly', async () => {
			const result = await resolveFields<TestModel, TestTable>({
				baseResolvers: {
					name: async () => {
						await new Promise(resolve => setTimeout(resolve, 10));
						return 'Async Name';
					},
				},
				traitResolvers: [],
				overrides: {},
				context: createContext(),
			});

			expect(result.name).toBe('Async Name');
		});

		it('mixed sync/async resolvers in same build', async () => {
			const result = await resolveFields<TestModel, TestTable>({
				baseResolvers: {
					name: 'Sync Name',
					email: () => Promise.resolve('async@example.com'),
					age: ({ sequence }) => sequence * 10,
					status: ({ sequence }) => Promise.resolve(`async-status-${sequence}`),
				},
				traitResolvers: [],
				overrides: {},
				context: createContext(3),
			});

			expect(result).toEqual({
				name: 'Sync Name',
				email: 'async@example.com',
				age: 30,
				status: 'async-status-3',
			});
		});
	});

	describe('Context', () => {
		it('resolver receives context with use and sequence', async () => {
			let capturedSequence: number | undefined;
			let capturedUseType: string | undefined;

			await resolveFields<TestModel, TestTable>({
				baseResolvers: {
					name: ctx => {
						capturedSequence = ctx.sequence;
						capturedUseType = typeof ctx.use;
						return 'Test';
					},
				},
				traitResolvers: [],
				overrides: {},
				context: createContext(42),
			});

			expect(capturedSequence).toBe(42);
			expect(capturedUseType).toBe('function');
		});

		it('UseHelper placeholder throws helpful error', () => {
			const use = createPlaceholderUseHelper();

			expect(() => use({} as never)).toThrow('[drizzle-fixtures] use() is not available');
		});
	});

	describe('Error Handling', () => {
		it('resolver error wrapped in FieldResolverError', async () => {
			await expect(
				resolveFields<TestModel, TestTable>({
					baseResolvers: {
						name: () => {
							throw new Error('Something went wrong');
						},
					},
					traitResolvers: [],
					overrides: {},
					context: createContext(),
				})
			).rejects.toThrow(FieldResolverError);
		});

		it('error message includes field name', async () => {
			await expect(
				resolveFields<TestModel, TestTable>({
					baseResolvers: {
						email: () => {
							throw new Error('Invalid email');
						},
					},
					traitResolvers: [],
					overrides: {},
					context: createContext(),
				})
			).rejects.toThrow('email');
		});

		it('non-Error throws are wrapped', async () => {
			await expect(
				resolveFields<TestModel, TestTable>({
					baseResolvers: {
						name: () => {
							// eslint-disable-next-line @typescript-eslint/only-throw-error -- Reason: Testing error handling - intentionally throwing string to verify non-Error exception wrapping
							throw 'string error';
						},
					},
					traitResolvers: [],
					overrides: {},
					context: createContext(),
				})
			).rejects.toThrow(FieldResolverError);
		});
	});

	describe('Edge Cases', () => {
		it('empty resolvers object returns empty result', async () => {
			const result = await resolveFields<TestModel, TestTable>({
				baseResolvers: {},
				traitResolvers: [],
				overrides: {},
				context: createContext(),
			});

			expect(result).toEqual({});
		});

		it('fields without resolver and no override are omitted', async () => {
			const result = await resolveFields<TestModel, TestTable>({
				baseResolvers: {
					name: 'Test',
				},
				traitResolvers: [],
				overrides: {},
				context: createContext(),
			});

			expect(result).toEqual({ name: 'Test' });
			expect(Object.hasOwn(result, 'email')).toBe(false);
			expect(Object.hasOwn(result, 'age')).toBe(false);
		});

		it('function as field value - document wrapping pattern', async () => {
			// To return a function as a value, wrap it in another function
			const myCallback = () => 'callback result';

			const result = await resolveFields<{ callback: () => string }, TestTable>({
				baseResolvers: {
					callback: () => myCallback, // Wrap to return function value
				},
				traitResolvers: [],
				overrides: {},
				context: createContext(),
			});

			expect(typeof result.callback).toBe('function');
			expect(result.callback()).toBe('callback result');
		});

		it('null values are handled correctly', async () => {
			const result = await resolveFields<TestModel, TestTable>({
				baseResolvers: {
					name: 'Test',
					age: null,
					status: () => null,
				},
				traitResolvers: [],
				overrides: {},
				context: createContext(),
			});

			expect(result).toEqual({
				name: 'Test',
				age: null,
				status: null,
			});
		});
	});

	describe('Sequence Integration', () => {
		it('sequence increments correctly across multiple builds', async () => {
			const counter = new SequenceCounter(1);

			const results: string[] = [];
			for (let i = 0; i < 3; i++) {
				const result = await resolveFields<TestModel, TestTable>({
					baseResolvers: {
						name: ({ sequence }) => `User ${sequence}`,
					},
					traitResolvers: [],
					overrides: {},
					context: createContext(counter.next()),
				});
				results.push(result.name);
			}

			expect(results).toEqual(['User 1', 'User 2', 'User 3']);
		});
	});
});

describe('mergeResolvers', () => {
	it('returns correct sources map for base fields', () => {
		const { sources } = mergeResolvers<TestModel, TestTable>(
			{ name: 'Base', email: 'base@test.com' },
			[],
			[]
		);

		expect(sources.name).toBe('base');
		expect(sources.email).toBe('base');
	});

	it('returns correct sources map for trait overrides', () => {
		const { sources } = mergeResolvers<TestModel, TestTable>(
			{ name: 'Base', email: 'base@test.com' },
			[{ name: 'Trait Name' }],
			[]
		);

		expect(sources.name).toBe('trait');
		expect(sources.email).toBe('base');
	});

	it('returns correct sources for overrides', () => {
		const { sources } = mergeResolvers<TestModel, TestTable>(
			{ name: 'Base', email: 'base@test.com' },
			[{ name: 'Trait Name' }],
			['email']
		);

		expect(sources.name).toBe('trait');
		expect(sources.email).toBe('override');
	});

	it('merged contains correct resolvers', () => {
		const baseResolver = () => 'base-email@test.com';
		const traitResolver = () => 'Trait Name';

		const { merged } = mergeResolvers<TestModel, TestTable>(
			{ name: 'Base', email: baseResolver },
			[{ name: traitResolver }],
			[]
		);

		expect(merged.name).toBe(traitResolver);
		expect(merged.email).toBe(baseResolver);
	});

	it('later traits override earlier traits', () => {
		const { merged, sources } = mergeResolvers<TestModel, TestTable>(
			{ name: 'Base', status: 'base-status' },
			[{ name: 'First', status: 'first-status' }, { name: 'Second' }, { status: 'third-status' }],
			[]
		);

		expect(merged.name).toBe('Second');
		expect(merged.status).toBe('third-status');
		expect(sources.name).toBe('trait');
		expect(sources.status).toBe('trait');
	});
});

describe('createSequenceCounter', () => {
	it('returns incrementing numbers starting from 1 by default', () => {
		const getSeq = createSequenceCounter();
		expect(getSeq()).toBe(1);
		expect(getSeq()).toBe(2);
		expect(getSeq()).toBe(3);
	});

	it('starts from custom value when specified', () => {
		const getSeq = createSequenceCounter(10);
		expect(getSeq()).toBe(10);
		expect(getSeq()).toBe(11);
	});

	it('each call increments the counter', () => {
		const getSeq = createSequenceCounter(1);
		for (let i = 1; i <= 5; i++) {
			expect(getSeq()).toBe(i);
		}
	});

	it('independent counters do not affect each other', () => {
		const counter1 = createSequenceCounter(1);
		const counter2 = createSequenceCounter(100);

		expect(counter1()).toBe(1);
		expect(counter2()).toBe(100);
		expect(counter1()).toBe(2);
		expect(counter2()).toBe(101);
	});
});

describe('SequenceCounter', () => {
	it('starts at configured value', () => {
		const counter = new SequenceCounter(5);
		expect(counter.next()).toBe(5);
		expect(counter.next()).toBe(6);
	});

	it('current() returns current value without incrementing', () => {
		const counter = new SequenceCounter(1);
		expect(counter.current()).toBe(0); // Before first next()
		counter.next();
		expect(counter.current()).toBe(1);
		expect(counter.current()).toBe(1); // Still 1
	});

	it('reset() restarts the sequence', () => {
		const counter = new SequenceCounter(1);
		counter.next();
		counter.next();
		counter.reset(10);
		expect(counter.next()).toBe(10);
	});

	it('default start is 1', () => {
		const counter = new SequenceCounter();
		expect(counter.next()).toBe(1);
	});

	it('startAt() returns the configured start value', () => {
		const counter = new SequenceCounter(100);
		expect(counter.startAt()).toBe(100);
		counter.reset(50);
		expect(counter.startAt()).toBe(50);
	});
});

describe('createPlaceholderUseHelper', () => {
	it('throws informative error when called', () => {
		const use = createPlaceholderUseHelper();

		expect(() => use({} as never)).toThrow('[drizzle-fixtures] use() is not available yet');
	});
});
