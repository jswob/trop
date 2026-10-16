import { describe, expect, it } from 'vitest';

import {
	EDGE,
	GAP,
	MEASURE,
	PAGE,
	blockWidth,
	columnsFor,
	fitsTheGlass,
	searchBounds,
} from './spread';

/** Every width the frame actually hands a page, plus the two either side of each boundary. */
const ROOMS = [
	320, 390, 575, 576, 577, 756, 868, 1056, 1151, 1152, 1153, 1464, 1727, 1728, 1729, 2400,
];

describe('columnsFor', () => {
	it('never returns fewer than one column, however narrow the glass', () => {
		for (const room of [0, 1, 100, 320, 575]) expect(columnsFor(room)).toBe(1);
	});

	it('starts a column only on the width that has room for the whole of it', () => {
		expect(columnsFor(1151)).toBe(1);
		expect(columnsFor(1152)).toBe(2);
		expect(columnsFor(1727)).toBe(2);
		expect(columnsFor(1728)).toBe(3);
	});

	/**
	 * The spread's first rule, as an assertion: *columns split only when there is room for
	 * another column*. A block wider than the box it is in is a column laid out past the edge of the
	 * glass, which is the one direction a document must never move in.
	 */
	it('never sets the block wider than the room it was measured in', () => {
		for (const room of ROOMS) {
			if (room < blockWidth(1)) continue;
			expect(blockWidth(columnsFor(room))).toBeLessThanOrEqual(room);
		}
	});

	it('leaves the surplus in one piece rather than handing it to the columns', () => {
		expect(blockWidth(columnsFor(1500))).toBe(1152);
		expect(1500 - blockWidth(columnsFor(1500))).toBe(348);
	});
});

describe('blockWidth', () => {
	it('is the columns, their gutters and one margin on each side', () => {
		expect(blockWidth(1)).toBe(2 * EDGE + MEASURE);
		expect(blockWidth(2)).toBe(2 * EDGE + 2 * MEASURE + GAP);
		expect(blockWidth(3)).toBe(2 * EDGE + 3 * MEASURE + 2 * GAP);
	});
});

describe('searchBounds', () => {
	/**
	 * The invariant the whole search rests on. `whole` is the flow in one column, so every column of
	 * that height holds it — the search can always end, and it can always end somewhere safe.
	 */
	it('bounds the search above by a height every column can hold the whole flow at', () => {
		for (const length of [400, 1133, 1461, 3187]) {
			for (const columns of [1, 2, 3]) {
				const { ideal, whole } = searchBounds(length, columns, 820);

				expect(whole).toBe(length + PAGE);
				expect(ideal).toBeLessThanOrEqual(whole);
			}
		}
	});

	it('opens the search at the arithmetic answer, which is a floor and not the answer', () => {
		expect(searchBounds(3187, 2, 820).ideal).toBe(Math.ceil(3187 / 2) + PAGE);
	});

	it('collapses to one height in a single column, where the two ends are the same number', () => {
		const { ideal, whole } = searchBounds(1461, 1, 820);

		expect(ideal).toBe(whole);
	});

	it('never opens the search below the glass while the glass is the taller of the two', () => {
		expect(searchBounds(1600, 2, 1200).ideal).toBe(1200);
	});
});

describe('fitsTheGlass', () => {
	it('keeps the foot at the glass while the flow fits the columns there are', () => {
		expect(fitsTheGlass(700, 1, 820)).toBe(true);
		expect(fitsTheGlass(1400, 2, 820)).toBe(true);
	});

	it('gives the glass up once the flow runs past the columns there are', () => {
		expect(fitsTheGlass(1461, 1, 820)).toBe(false);
		expect(fitsTheGlass(3187, 2, 820)).toBe(false);
	});

	/** A glass shorter than the page's own margin holds nothing, and the arithmetic says so. */
	it('holds nothing in a glass no taller than the page margin', () => {
		expect(fitsTheGlass(10, 3, PAGE)).toBe(false);
	});
});

describe('an unbroken block', () => {
	it('never lets the search start on a column shorter than the block', () => {
		expect(searchBounds(3000, 4, 700).ideal).toBe(Math.ceil(3000 / 4) + PAGE);
		expect(searchBounds(3000, 4, 700, 1200).ideal).toBe(1200 + PAGE);
	});

	it('changes nothing where there is none, or where it is shorter than the ideal', () => {
		expect(searchBounds(3187, 2, 820, 0)).toEqual(searchBounds(3187, 2, 820));
		expect(searchBounds(3187, 2, 820, 400)).toEqual(searchBounds(3187, 2, 820));
	});

	it('is still bounded by the whole flow in one column', () => {
		const { ideal, whole } = searchBounds(1500, 3, 700, 1500);
		expect(ideal).toBe(whole);
	});

	it('does not fit the glass when the block is taller than a column there', () => {
		expect(fitsTheGlass(1400, 2, 820)).toBe(true);
		expect(fitsTheGlass(1400, 2, 820, 900)).toBe(false);
		expect(fitsTheGlass(1400, 2, 820, 700)).toBe(true);
	});
});
