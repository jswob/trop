import { Elysia, t } from 'elysia';
import type { DatabaseConnection } from '../../db/types';
import { mapError } from '../../utils/service-result';
import { createNote, deleteNote, findNote, listNotes, updateNote } from './service';

/** The longest title the API takes. */
export const NOTE_TITLE_LIMIT = 120;

const uuidParams = t.Object({ id: t.String({ format: 'uuid', error: 'Invalid UUID format' }) });

const title = t.String({
	minLength: 1,
	maxLength: NOTE_TITLE_LIMIT,
	error: `Title is required and must be at most ${String(NOTE_TITLE_LIMIT)} characters`,
});
const body = t.String({ error: 'Body must be a string' });

const noteBody = t.Object({ title, body: t.Optional(body) });

/** A save sends only what changed. */
const notePatchBody = t.Object({ title: t.Optional(title), body: t.Optional(body) });

/**
 * **The example resource's routes**: list, read, write, change and delete a note. Each answers a
 * failure with `mapError`, whose statuses are literals, so Eden keeps it off `data`.
 */
export const createNotesRoute = (db: DatabaseConnection) =>
	new Elysia({ prefix: '/api/notes' })
		.onTransform(({ body }) => {
			if (body && typeof body === 'object' && 'title' in body && typeof body.title === 'string') {
				body.title = body.title.trim();
			}
		})
		.get('/', () => listNotes(db))
		.get(
			'/:id',
			async ({ params }) => {
				const result = await findNote(db, params.id);
				if (!result.ok) return mapError(result.error);
				return result.data;
			},
			{ params: uuidParams }
		)
		.post(
			'/',
			async ({ body, status }) => {
				const result = await createNote(db, body);
				if (!result.ok) return mapError(result.error);
				return status(201, result.data);
			},
			{ body: noteBody }
		)
		.patch(
			'/:id',
			async ({ params, body }) => {
				const result = await updateNote(db, params.id, body);
				if (!result.ok) return mapError(result.error);
				return result.data;
			},
			{ params: uuidParams, body: notePatchBody }
		)
		.delete(
			'/:id',
			async ({ params }) => {
				const result = await deleteNote(db, params.id);
				if (!result.ok) return mapError(result.error);
				return result.data;
			},
			{ params: uuidParams }
		);
