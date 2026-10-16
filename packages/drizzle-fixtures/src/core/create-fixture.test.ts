import { describe, it, expect } from 'vitest';
import { users, posts } from '../fixtures/test-schema';
import type { DrizzleDatabase } from '../types';
import { ValidationError } from '../utils/errors';
import { createFixture } from './create-fixture';

// Mock database for testing
const mockDb = {} as DrizzleDatabase;

describe('createFixture()', () => {
	it('returns a function', () => {
		const userFixture = createFixture({
			table: users,
			fields: {
				id: 1,
				name: 'Test User',
			},
		});

		expect(typeof userFixture).toBe('function');
	});

	it('returned function returns builder when called with db mock', () => {
		const userFixture = createFixture({
			table: users,
			fields: {
				id: 1,
				name: 'Test User',
			},
		});

		const builder = userFixture(mockDb);

		expect(builder).toBeDefined();
		expect(typeof builder.build).toBe('function');
		expect(typeof builder.buildList).toBe('function');
		expect(typeof builder.trait).toBe('function');
		expect(typeof builder.dryRun).toBe('function');
		expect(typeof builder.explain).toBe('function');
		expect(typeof builder.create).toBe('function');
		expect(typeof builder.createList).toBe('function');
	});

	it('works with Drizzle table object', () => {
		const userFixture = createFixture({
			table: users,
			fields: {
				id: 1,
				name: 'Test',
				email: 'test@example.com',
			},
		});

		expect(() => userFixture(mockDb)).not.toThrow();
	});

	it('works with different tables', () => {
		const postFixture = createFixture({
			table: posts,
			fields: {
				id: 1,
				title: 'Test Post',
			},
		});

		const builder = postFixture(mockDb);
		expect(builder.explain().tableName).toBe('posts');
	});

	it('throws ValidationError if table is null', () => {
		expect(() =>
			createFixture({
				table: null as never,
				fields: {},
			})
		).toThrow(ValidationError);
	});

	it('throws ValidationError if table is undefined', () => {
		expect(() =>
			createFixture({
				table: undefined as never,
				fields: {},
			})
		).toThrow(ValidationError);
	});

	it('throws ValidationError if table is invalid object', () => {
		expect(() =>
			createFixture({
				table: { notATable: true } as never,
				fields: {},
			})
		).toThrow(ValidationError);
	});

	it('throws ValidationError if db is null', () => {
		const userFixture = createFixture({
			table: users,
			fields: {
				id: 1,
				name: 'Test',
			},
		});

		expect(() => userFixture(null as never)).toThrow(ValidationError);
		expect(() => userFixture(null as never)).toThrow('Database connection is required');
	});

	it('throws ValidationError if db is undefined', () => {
		const userFixture = createFixture({
			table: users,
			fields: {
				id: 1,
				name: 'Test',
			},
		});

		expect(() => userFixture(undefined as never)).toThrow(ValidationError);
	});

	it('builder can build objects', async () => {
		const userFixture = createFixture({
			table: users,
			fields: {
				id: ({ sequence }) => sequence,
				name: ({ sequence }) => `User ${sequence}`,
				email: 'test@example.com',
			},
		});

		const builder = userFixture(mockDb);
		const user = await builder.build();

		expect(user.id).toBe(1);
		expect(user.name).toBe('User 1');
		expect(user.email).toBe('test@example.com');
	});

	it('each fixture function call creates independent builder', async () => {
		const userFixture = createFixture({
			table: users,
			fields: {
				id: ({ sequence }) => sequence,
				name: ({ sequence }) => `User ${sequence}`,
			},
		});

		const builder1 = userFixture(mockDb);
		const builder2 = userFixture(mockDb);

		const user1 = await builder1.build();
		const user2 = await builder2.build();

		// Each builder has its own sequence counter
		expect(user1.id).toBe(1);
		expect(user2.id).toBe(1);
	});

	it('same builder maintains sequence across builds', async () => {
		const userFixture = createFixture({
			table: users,
			fields: {
				id: ({ sequence }) => sequence,
			},
		});

		const builder = userFixture(mockDb);

		const user1 = await builder.build();
		const user2 = await builder.build();
		const user3 = await builder.build();

		expect(user1.id).toBe(1);
		expect(user2.id).toBe(2);
		expect(user3.id).toBe(3);
	});

	it('passes hooks to builder', () => {
		const beforeMake = () => {};
		const afterMake = () => {};

		const userFixture = createFixture({
			table: users,
			fields: {
				id: 1,
				name: 'Test',
			},
			hooks: {
				beforeMake,
				afterMake,
			},
		});

		// Should not throw
		expect(() => userFixture(mockDb)).not.toThrow();
	});
});
