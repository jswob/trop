import { dbNow } from '@jswob/drizzle-utils';
import { desc, eq } from 'drizzle-orm';
import { notes } from '../../db/schema';
import type { DatabaseConnection } from '../../db/types';
import type { ServiceResult } from '../../utils/service-result';
import { fail, ok } from '../../utils/service-result';

type NoteRow = typeof notes.$inferSelect;

/** What a note is written from: its title, and its body when it has one. */
export type NoteInput = { title: string; body?: string };

/**
 * Every note, the one changed last first.
 * @param db - Database connection
 */
export function listNotes(db: DatabaseConnection): Promise<NoteRow[]> {
	return db.select().from(notes).orderBy(desc(notes.updatedAt), desc(notes.createdAt));
}

/**
 * One note.
 * @param db - Database connection
 * @param id - The note's id
 * @returns The note, or NOT_FOUND
 */
export async function findNote(
	db: DatabaseConnection,
	id: string
): Promise<ServiceResult<NoteRow>> {
	const [found] = await db.select().from(notes).where(eq(notes.id, id));
	return found === undefined ? fail('NOT_FOUND', 'Note not found') : ok(found);
}

/**
 * Write a new note.
 * @param db - Database connection
 * @param input - Its title and body
 */
export async function createNote(
	db: DatabaseConnection,
	input: NoteInput
): Promise<ServiceResult<NoteRow>> {
	const [created] = await db.insert(notes).values(input).returning();
	return created === undefined ? fail('INTERNAL', 'Failed to create note') : ok(created);
}

/**
 * Change a note: a field left out is not touched.
 * @param db - Database connection
 * @param id - The note's id
 * @param input - What changed
 * @returns The note as saved, or NOT_FOUND
 */
export async function updateNote(
	db: DatabaseConnection,
	id: string,
	input: Partial<NoteInput>
): Promise<ServiceResult<NoteRow>> {
	const [updated] = await db
		.update(notes)
		.set({ ...input, updatedAt: dbNow() })
		.where(eq(notes.id, id))
		.returning();
	return updated === undefined ? fail('NOT_FOUND', 'Note not found') : ok(updated);
}

/**
 * Delete a note.
 * @param db - Database connection
 * @param id - The note's id
 * @returns The note as it was, or NOT_FOUND
 */
export async function deleteNote(
	db: DatabaseConnection,
	id: string
): Promise<ServiceResult<NoteRow>> {
	const [deleted] = await db.delete(notes).where(eq(notes.id, id)).returning();
	return deleted === undefined ? fail('NOT_FOUND', 'Note not found') : ok(deleted);
}
