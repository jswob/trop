import { eq } from 'drizzle-orm';
import { pgTable, serial, text } from 'drizzle-orm/pg-core';
import { drizzle } from 'drizzle-orm/postgres-js';
import postgres from 'postgres';
import { afterEach, describe, expect, test } from 'vitest';
import { createDbTargets } from './db-target';
import { withPreparedStatements } from './prepared';

/** The test database; `postgres`, which every server has, when `TEST_DB_NAME` is unset. */
const url = createDbTargets({ app: 'postgres', test: 'postgres', e2e: 'postgres' }).dbTarget(
	'test'
).url;

const notes = pgTable('notes', { id: serial('id').primaryKey(), name: text('name').notNull() });
const tags = pgTable('tags', { id: serial('id').primaryKey(), name: text('name').notNull() });
const schema = { notes, tags };

/**
 * One connection, so the statements it has prepared are the ones this test asked for. Its tables
 * are temporary: they belong to the connection and vanish with it.
 */
async function connect(): Promise<postgres.Sql> {
	const sql = postgres(url, { max: 1, ssl: false, onnotice: () => {} });
	await sql`create temporary table notes (id serial primary key, name text not null)`;
	await sql`create temporary table tags (id serial primary key, name text not null)`;
	return sql;
}

let client: postgres.Sql | undefined;
afterEach(async () => {
	await client?.end();
	client = undefined;
});

/**
 * The reads of the test's tables the connection holds prepared, as the server lists them.
 * postgres.js prepares statements of its own (catalog reads, this one among them, `commit`,
 * `savepoint`); those are left out.
 */
async function prepared(sql: postgres.Sql): Promise<string[]> {
	const rows = await sql<{ statement: string }[]>`select statement from pg_prepared_statements`;
	return rows
		.map(row => row.statement)
		.filter(statement => / from "[a-z_]+"/.test(statement) && !statement.includes('pg_'));
}

describe('withPreparedStatements', () => {
	test('a plain client sends what Drizzle asks unprepared', async () => {
		client = await connect();
		const db = drizzle(client, { schema });

		await db.select().from(notes).where(eq(notes.name, 'Nobody'));

		expect(await prepared(client)).toEqual([]);
	});

	test('prepares what Drizzle asks', async () => {
		client = await connect();
		const db = drizzle(withPreparedStatements(client), { schema });

		const rows = await db.select().from(notes).where(eq(notes.name, 'Nobody'));

		expect(rows).toEqual([]);
		expect(await prepared(client)).toEqual([expect.stringContaining('from "notes"')]);
	});

	test('prepares a statement once, however often it is asked', async () => {
		client = await connect();
		const db = drizzle(withPreparedStatements(client), { schema });

		for (const name of ['Ada', 'Grace', 'Edsger']) {
			await db.select().from(notes).where(eq(notes.name, name));
		}

		expect(await prepared(client)).toHaveLength(1);
	});

	test('prepares inside a transaction and inside one nested in it', async () => {
		client = await connect();
		const db = drizzle(withPreparedStatements(client), { schema });

		await db.transaction(async tx => {
			await tx.select().from(tags).where(eq(tags.name, 'Nobody'));
			await tx.transaction(async nested => {
				await nested.select().from(notes).where(eq(notes.name, 'Nothing'));
			});
		});

		expect((await prepared(client)).sort()).toEqual([
			expect.stringContaining('from "notes"'),
			expect.stringContaining('from "tags"'),
		]);
	});

	test('a transaction still rolls back', async () => {
		client = await connect();
		const db = drizzle(withPreparedStatements(client), { schema });

		await expect(
			db.transaction(async tx => {
				await tx.insert(notes).values({ name: 'Left behind by a rollback' });
				throw new Error('undo');
			})
		).rejects.toThrow('undo');

		const left = await db.select().from(notes).where(eq(notes.name, 'Left behind by a rollback'));
		expect(left).toEqual([]);
	});
});
