import { eq } from 'drizzle-orm';
import type { Note } from '../../db/schema';
import { notes } from '../../db/schema';
import { describe, expect, test } from '../../test/test-context';
import { NOTE_TITLE_LIMIT, createNotesRoute } from './index';

const BASE_URL = 'http://localhost/api/notes';
const MISSING = '00000000-0000-0000-0000-000000000099';

function send(method: string, url: string, body?: unknown) {
	return new Request(url, {
		method,
		headers: { 'Content-Type': 'application/json' },
		...(body !== undefined && { body: JSON.stringify(body) }),
	});
}

describe('GET /api/notes', () => {
	test('lists every note, the one changed last first', async ({ tx, factory }) => {
		await factory.note.create({ title: 'Older', updatedAt: new Date('2026-01-01T00:00:00Z') });
		await factory.note.create({ title: 'Newer', updatedAt: new Date('2026-02-01T00:00:00Z') });

		const response = await createNotesRoute(tx).handle(new Request(BASE_URL));

		expect(response.status).toBe(200);
		const body = (await response.json()) as Note[];
		expect(body.map(note => note.title)).toEqual(['Newer', 'Older']);
	});

	test('answers an empty list when there are none', async ({ tx }) => {
		const response = await createNotesRoute(tx).handle(new Request(BASE_URL));

		expect(await response.json()).toEqual([]);
	});
});

describe('GET /api/notes/:id', () => {
	test('answers the note', async ({ tx, factory }) => {
		const note = await factory.note.create({ title: 'Groceries' });

		const response = await createNotesRoute(tx).handle(new Request(`${BASE_URL}/${note.id}`));

		expect(response.status).toBe(200);
		expect(await response.json()).toMatchObject({ id: note.id, title: 'Groceries' });
	});

	test('answers 404 for a note that does not exist', async ({ tx }) => {
		const response = await createNotesRoute(tx).handle(new Request(`${BASE_URL}/${MISSING}`));

		expect(response.status).toBe(404);
		expect(await response.json()).toEqual({ error: 'Note not found' });
	});

	test('refuses an id that is not a UUID', async ({ tx }) => {
		const response = await createNotesRoute(tx).handle(new Request(`${BASE_URL}/not-a-uuid`));

		expect(response.status).toBe(422);
	});
});

describe('POST /api/notes', () => {
	test('writes a note, trimming its title', async ({ tx }) => {
		const response = await createNotesRoute(tx).handle(
			send('POST', BASE_URL, { title: '  Groceries  ', body: 'Milk' })
		);

		expect(response.status).toBe(201);
		const created = (await response.json()) as Note;
		expect(created).toMatchObject({ title: 'Groceries', body: 'Milk' });
		const [saved] = await tx.select().from(notes).where(eq(notes.id, created.id));
		expect(saved?.title).toBe('Groceries');
	});

	test('gives a note written without a body an empty one', async ({ tx }) => {
		const response = await createNotesRoute(tx).handle(send('POST', BASE_URL, { title: 'Bare' }));

		expect(response.status).toBe(201);
		expect(((await response.json()) as Note).body).toBe('');
	});

	test('refuses a title that is empty once trimmed', async ({ tx }) => {
		const response = await createNotesRoute(tx).handle(send('POST', BASE_URL, { title: '   ' }));

		expect(response.status).toBe(422);
		expect(await tx.select().from(notes)).toHaveLength(0);
	});

	test('refuses a title past the limit', async ({ tx }) => {
		const response = await createNotesRoute(tx).handle(
			send('POST', BASE_URL, { title: 'x'.repeat(NOTE_TITLE_LIMIT + 1) })
		);

		expect(response.status).toBe(422);
	});
});

describe('PATCH /api/notes/:id', () => {
	test('changes only what it is sent, and moves the note to the top', async ({ tx, factory }) => {
		const note = await factory.note.create({
			title: 'Groceries',
			body: 'Milk',
			updatedAt: new Date('2026-01-01T00:00:00Z'),
		});

		const response = await createNotesRoute(tx).handle(
			send('PATCH', `${BASE_URL}/${note.id}`, { body: 'Milk, eggs' })
		);

		expect(response.status).toBe(200);
		const saved = (await response.json()) as Note;
		expect(saved).toMatchObject({ title: 'Groceries', body: 'Milk, eggs' });
		expect(new Date(saved.updatedAt).getTime()).toBeGreaterThan(note.updatedAt.getTime());
	});

	test('answers 404 for a note that does not exist', async ({ tx }) => {
		const response = await createNotesRoute(tx).handle(
			send('PATCH', `${BASE_URL}/${MISSING}`, { title: 'Gone' })
		);

		expect(response.status).toBe(404);
	});

	test('refuses an empty title', async ({ tx, factory }) => {
		const note = await factory.note.create();

		const response = await createNotesRoute(tx).handle(
			send('PATCH', `${BASE_URL}/${note.id}`, { title: '' })
		);

		expect(response.status).toBe(422);
	});
});

describe('DELETE /api/notes/:id', () => {
	test('deletes the note and answers it as it was', async ({ tx, factory }) => {
		const note = await factory.note.create({ title: 'Groceries' });

		const response = await createNotesRoute(tx).handle(send('DELETE', `${BASE_URL}/${note.id}`));

		expect(response.status).toBe(200);
		expect(await response.json()).toMatchObject({ id: note.id, title: 'Groceries' });
		expect(await tx.select().from(notes)).toHaveLength(0);
	});

	test('answers 404 for a note that does not exist', async ({ tx }) => {
		const response = await createNotesRoute(tx).handle(send('DELETE', `${BASE_URL}/${MISSING}`));

		expect(response.status).toBe(404);
	});
});
