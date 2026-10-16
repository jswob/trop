/**
 * Integration test schema for drizzle-fixtures.
 *
 * Uses UUID primary keys, pgEnum, and FK relationships to test
 * real database operations matching production patterns.
 */
import { pgTable, uuid, varchar, timestamp, pgEnum, text } from 'drizzle-orm/pg-core';

/**
 * Role enum for testing pgEnum support.
 */
export const roleEnum = pgEnum('integration_role', ['user', 'admin', 'moderator']);

/**
 * Integration users table with UUID PK and enum field.
 */
export const integrationUsers = pgTable('integration_users', {
	id: uuid('id').primaryKey().defaultRandom(),
	name: varchar('name', { length: 255 }).notNull(),
	email: varchar('email', { length: 255 }).notNull().unique(),
	role: roleEnum('role').notNull().default('user'),
	createdAt: timestamp('created_at').defaultNow().notNull(),
	updatedAt: timestamp('updated_at').defaultNow().notNull(),
});

/**
 * Integration posts table with FK to users.
 */
export const integrationPosts = pgTable('integration_posts', {
	id: uuid('id').primaryKey().defaultRandom(),
	title: varchar('title', { length: 255 }).notNull(),
	content: text('content'),
	userId: uuid('user_id')
		.notNull()
		.references(() => integrationUsers.id, { onDelete: 'cascade' }),
	status: varchar('status', { length: 50 }).default('draft'),
	createdAt: timestamp('created_at').defaultNow().notNull(),
	updatedAt: timestamp('updated_at').defaultNow().notNull(),
});

// Type exports for use in fixtures
export type IntegrationUser = typeof integrationUsers.$inferSelect;
export type NewIntegrationUser = typeof integrationUsers.$inferInsert;
export type IntegrationPost = typeof integrationPosts.$inferSelect;
export type NewIntegrationPost = typeof integrationPosts.$inferInsert;
