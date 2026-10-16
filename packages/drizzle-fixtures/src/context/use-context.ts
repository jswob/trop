import type { InferSelectModel } from 'drizzle-orm';
import type { AnyPgTable } from 'drizzle-orm/pg-core';
import type { SequenceCounter } from '../core/sequence';
import type {
	DrizzleDatabase,
	FixtureFunction,
	UseHelper,
	TraitAugmentations,
	FixtureBuilder,
} from '../types';
import { CircularDependencyError } from '../utils/errors';

export interface UseContextOptions {
	db: DrizzleDatabase;
	sequence: SequenceCounter;
	resolutionStack?: string[];
}

export function createUseHelper(options: UseContextOptions): UseHelper {
	const { db, sequence, resolutionStack = [] } = options;

	return <
		TTable extends AnyPgTable,
		TTraits extends Record<string, unknown>,
		TAugmentations extends TraitAugmentations<TTraits>,
	>(
		fixture: FixtureFunction<TTable, TTraits, TAugmentations>
	): FixtureBuilder<InferSelectModel<TTable>, TTraits, TAugmentations, readonly []> => {
		const fixtureName = getFixtureName(fixture);

		if (resolutionStack.includes(fixtureName)) {
			throw new CircularDependencyError([...resolutionStack, fixtureName]);
		}

		const newStack = [...resolutionStack, fixtureName];

		return fixture(db, {
			sequence,
			resolutionStack: newStack,
		});
	};
}

function getFixtureName<
	TTable extends AnyPgTable,
	TTraits extends Record<string, unknown>,
	TAugmentations extends TraitAugmentations<TTraits>,
>(fixture: FixtureFunction<TTable, TTraits, TAugmentations>): string {
	return (fixture as { _tableName?: string })._tableName ?? 'anonymous';
}
