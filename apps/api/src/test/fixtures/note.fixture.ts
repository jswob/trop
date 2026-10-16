import { faker } from '@faker-js/faker';
import { createFixture } from '@jswob/drizzle-fixtures';
import { notes } from '../../db/schema';

export const noteFixture = createFixture<typeof notes>({
	table: notes,
	fields: {
		title: () => faker.lorem.sentence({ min: 2, max: 5 }),
		body: () => faker.lorem.paragraph(),
	},
});
