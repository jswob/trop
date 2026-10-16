import type { InferInsertModel, InferSelectModel } from 'drizzle-orm';
import { pgTable, text, varchar } from 'drizzle-orm/pg-core';
import { timestamps, uuidPrimaryKey } from './common';

/**
 * The example resource: a note, a title and a body. It stands in for the app's first table and
 * leaves in one commit (`docs/removing-notes.md` lists its files).
 */
export const notes = pgTable('notes', {
	id: uuidPrimaryKey,
	title: varchar('title', { length: 255 }).notNull(),
	body: text('body').default('').notNull(),
	...timestamps,
});

export type Note = InferSelectModel<typeof notes>;
export type NewNote = InferInsertModel<typeof notes>;
