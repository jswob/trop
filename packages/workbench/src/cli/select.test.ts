import { describe, expect, it } from 'vitest';

import type { FrameEntry } from '../lib/frame-census';
import { selectFrames, selectOne } from './select';

const CENSUS: FrameEntry[] = [
	{
		id: 'timeline-sheet',
		kind: 'view',
		title: 'T',
		variant: 'populated',
		variantLabel: 'P',
		chrome: 'bare',
	},
	{
		id: 'timeline-sheet',
		kind: 'view',
		title: 'T',
		variant: 'empty',
		variantLabel: 'E',
		chrome: 'bare',
	},
	{
		id: 'order-create',
		kind: 'view',
		title: 'M',
		variant: 'blank',
		variantLabel: 'B',
		chrome: 'bare',
	},
	{
		id: 'timeline',
		kind: 'kit',
		title: 'B',
		variant: 'default',
		variantLabel: 'D',
		chrome: 'bare',
	},
];

describe('selectFrames', () => {
	it('takes every variant of a bare id', () => {
		expect(selectFrames(CENSUS, { frames: ['timeline-sheet'], spaces: [] })).toHaveLength(2);
	});

	it('takes one variant of a qualified id', () => {
		const chosen = selectFrames(CENSUS, { frames: ['timeline-sheet:empty'], spaces: [] });

		expect(chosen.map(entry => entry.variant)).toEqual(['empty']);
	});

	it('does not let a prefix of an id match it', () => {
		expect(
			selectFrames(CENSUS, { frames: ['timeline'], spaces: [] }).map(entry => entry.kind)
		).toEqual(['kit']);
	});

	it('filters by space when no frames are named, and takes everything when neither is', () => {
		expect(selectFrames(CENSUS, { frames: [], spaces: ['view'] })).toHaveLength(3);
		expect(selectFrames(CENSUS, { frames: [], spaces: [] })).toHaveLength(4);
	});
});

describe('selectOne', () => {
	it('returns the one frame a qualified token names', () => {
		expect(selectOne(CENSUS, 'order-create:blank').variant).toBe('blank');
	});

	it('takes an unqualified id only when it has exactly one state', () => {
		expect(selectOne(CENSUS, 'order-create').variant).toBe('blank');
		expect(() => selectOne(CENSUS, 'timeline-sheet')).toThrow(/has 2 states/);
	});

	/**
	 * Driving is not probing. A probe over four frames is four reports; a press applies to one
	 * screen, and "which of those did I press" is not a question the report can answer afterwards.
	 */
	it('names the states it does have rather than guessing', () => {
		expect(() => selectOne(CENSUS, 'timeline-sheet:menu')).toThrow(/populated, empty/);
	});

	it('suggests the near misses when the id is wrong', () => {
		expect(() => selectOne(CENSUS, 'timeline-shee')).toThrow(/did you mean/);
		expect(() => selectOne(CENSUS, 'nothing-like-it')).toThrow(/no frame with id/);
	});
});
