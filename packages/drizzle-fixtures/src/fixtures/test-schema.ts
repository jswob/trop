/**
 * Mock Drizzle tables for testing fixture functionality.
 *
 * These tables are used only for tests - they provide realistic schema
 * definitions without requiring a real database connection.
 */
import { pgTable, serial, varchar, integer, timestamp } from 'drizzle-orm/pg-core';

export const users = pgTable('users', {
	id: serial('id').primaryKey(),
	name: varchar('name', { length: 255 }).notNull(),
	email: varchar('email', { length: 255 }).notNull(),
	role: varchar('role', { length: 50 }).default('user'),
	createdAt: timestamp('created_at').defaultNow(),
});

export const posts = pgTable('posts', {
	id: serial('id').primaryKey(),
	title: varchar('title', { length: 255 }).notNull(),
	content: varchar('content', { length: 1000 }),
	userId: integer('user_id').references(() => users.id),
	status: varchar('status', { length: 50 }).default('draft'),
	createdAt: timestamp('created_at').defaultNow(),
});

export type User = typeof users.$inferSelect;
export type NewUser = typeof users.$inferInsert;
export type Post = typeof posts.$inferSelect;
export type NewPost = typeof posts.$inferInsert;
