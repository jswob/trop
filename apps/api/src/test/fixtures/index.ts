import { composeFactory } from '@jswob/drizzle-fixtures';
import { noteFixture } from './note.fixture';

const fixtures = {
	note: noteFixture,
};

export const factory = composeFactory(fixtures);
export type FixturesMap = typeof fixtures;
