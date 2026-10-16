/**
 * Integration test fixtures for drizzle-fixtures.
 *
 * Uses the integration schema with UUID PKs, enums, and FK relationships.
 */
import { faker } from '@faker-js/faker';

import { createFixture } from '../core/create-fixture';
import type { IntegrationUser, IntegrationPost } from './schema';
import { integrationUsers, integrationPosts } from './schema';

/**
 * User fixture traits.
 */
type UserTraits = {
	admin: never;
	moderator: never;
};

/**
 * User fixture for integration tests.
 *
 * Generates users with UUID IDs, unique emails using sequence,
 * and supports admin/moderator traits.
 */
export const userFixture = createFixture<typeof integrationUsers, UserTraits>({
	table: integrationUsers,

	fields: {
		name: () => faker.person.fullName(),
		email: ({ sequence }) => `user-${sequence}-${faker.string.alphanumeric(8)}@test.com`,
		role: () => 'user',
	},

	traits: {
		admin: {
			fields: {
				role: () => 'admin',
			},
		},
		moderator: {
			fields: {
				role: () => 'moderator',
			},
		},
	},
});

/**
 * Post fixture traits.
 */
type PostTraits = {
	published: never;
	draft: never;
};

/**
 * Post fixture for integration tests.
 *
 * Generates posts with FK to users, supports published/draft traits.
 */
export const postFixture = createFixture<typeof integrationPosts, PostTraits>({
	table: integrationPosts,

	fields: {
		title: () => faker.lorem.sentence({ min: 3, max: 8 }),
		content: () => faker.lorem.paragraphs(2),
		// userId: created via use() helper if not provided as override
		userId: async ({ use }) => {
			const user = await use(userFixture).create();
			return user.id;
		},
		status: () => 'draft',
	},

	traits: {
		published: {
			fields: {
				status: () => 'published',
			},
		},
		draft: {
			fields: {
				status: () => 'draft',
			},
		},
	},
});

// Export types for test assertions
export type { IntegrationUser, IntegrationPost };
