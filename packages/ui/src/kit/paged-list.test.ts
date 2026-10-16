import { describe, expect, it } from 'vitest';

import {
	BY_NAME,
	NAME_ORDER,
	PAGE_SIZE,
	compareNames,
	findRows,
	listFolds,
	listTracks,
	nextSort,
	pageCount,
	pageRows,
	pageSpan,
	sortRows,
} from './paged-list';
import type { ListColumn, ListSort } from './paged-list';

type Row = { id: string; name: string; days: number };

const DAYS: ListColumn<Row> = {
	id: 'days',
	head: 'Days',
	figure: row => String(row.days),
	value: row => row.days,
	narrow: true,
};

const rows = (count: number): Row[] =>
	Array.from({ length: count }, (_unused, index) => ({
		id: `r-${String(index)}`,
		name: `Name ${String(index).padStart(3, '0')}`,
		days: index % 3,
	}));

describe('findRows', () => {
	const records: Row[] = [
		{ id: 'a', name: 'Wiktoria Wiśniewska', days: 1 },
		{ id: 'b', name: 'Rafał Łukasik', days: 2 },
		{ id: 'c', name: 'Ada Abara', days: 3 },
	];

	it('keeps everything for an empty or blank search', () => {
		expect(findRows(records, '')).toEqual(records);
		expect(findRows(records, '   ')).toEqual(records);
	});

	it('matches anywhere in the name, ignoring case and diacritics', () => {
		expect(findRows(records, 'wisn').map(row => row.id)).toEqual(['a']);
		expect(findRows(records, 'LUKA').map(row => row.id)).toEqual(['b']);
		expect(findRows(records, 'bar').map(row => row.id)).toEqual(['c']);
	});

	it('keeps nothing when nothing matches', () => {
		expect(findRows(records, 'zzq')).toEqual([]);
	});

	describe('with more to find a record by than its name', () => {
		const tags: Record<string, string[]> = {
			a: ['Garden', 'Groceries'],
			b: ['Errands in town'],
			c: [],
		};
		const keys = (row: Row) => tags[row.id] ?? [];

		it('keeps a record whose other words match, folded the same way', () => {
			expect(findRows(records, 'garden', keys).map(row => row.id)).toEqual(['a']);
			expect(findRows(records, 'TOWN', keys).map(row => row.id)).toEqual(['b']);
		});

		it('still keeps a record by its name', () => {
			expect(findRows(records, 'abara', keys).map(row => row.id)).toEqual(['c']);
		});

		it('keeps a record once when its name and its words both match', () => {
			const both = (row: Row) => (row.id === 'c' ? ['Ada’s list'] : []);
			expect(findRows(records, 'ada', both).map(row => row.id)).toEqual(['c']);
		});
	});
});

describe('listTracks', () => {
	const WORDS: ListColumn<Row> = {
		id: 'words',
		head: 'Words',
		figure: row => String(row.days),
		value: row => row.days,
		narrow: true,
		words: row => (row.days === 0 ? null : 'Some words'),
		none: 'None',
	};

	it('gives the name the rest of the line and each figure its widest', () => {
		expect(listTracks([DAYS], 'wide')).toBe('minmax(0,1fr) max-content');
		expect(listTracks([DAYS, { ...DAYS, narrow: false }], 'narrow')).toBe(
			'minmax(0,1fr) max-content'
		);
	});

	it('from md, fixes the name at a whole name and gives a column of words the rest', () => {
		expect(listTracks([WORDS, DAYS], 'wide')).toBe('minmax(0,14rem) minmax(0,1fr) max-content');
	});

	it('below md, draws a column of words as its figure', () => {
		expect(listTracks([WORDS, DAYS], 'narrow')).toBe('minmax(0,1fr) max-content max-content');
	});
});

describe('paging', () => {
	it('is never fewer than one page', () => {
		expect(pageCount(0)).toBe(1);
		expect(pageCount(PAGE_SIZE)).toBe(1);
		expect(pageCount(PAGE_SIZE + 1)).toBe(2);
		expect(pageCount(212)).toBe(11);
	});

	it('draws the last page as short as its rows', () => {
		expect(pageRows(rows(212), 11)).toHaveLength(12);
	});

	it('holds a page asked for inside the list', () => {
		expect(pageRows(rows(212), 99)).toEqual(pageRows(rows(212), 11));
		expect(pageRows(rows(212), 0)).toEqual(pageRows(rows(212), 1));
	});

	it('spans a page 1-based and inclusive', () => {
		expect(pageSpan(1, 212)).toEqual({ first: 1, last: 20 });
		expect(pageSpan(2, 212)).toEqual({ first: 21, last: 40 });
		expect(pageSpan(11, 212)).toEqual({ first: 201, last: 212 });
		expect(pageSpan(3, 41)).toEqual({ first: 41, last: 41 });
	});
});

describe('sorting', () => {
	it('starts on names, A to Z', () => {
		expect(NAME_ORDER).toEqual({ by: BY_NAME, way: 'ascending' });
	});

	it('orders two records with one name by their ids', () => {
		expect(compareNames({ id: 'b', name: 'Ada' }, { id: 'a', name: 'ada' })).toBeGreaterThan(0);
	});

	it('breaks a tie on a figure by name, A to Z, whichever way the figure runs', () => {
		const sorted = sortRows(rows(6), { by: 'days', way: 'descending' }, [DAYS]);
		expect(sorted.map(row => `${String(row.days)} ${row.name}`)).toEqual([
			'2 Name 002',
			'2 Name 005',
			'1 Name 001',
			'1 Name 004',
			'0 Name 000',
			'0 Name 003',
		]);
	});

	it('sorts by name for a key no column has', () => {
		const sorted = sortRows(rows(3), { by: BY_NAME, way: 'descending' }, [DAYS]);
		expect(sorted.map(row => row.id)).toEqual(['r-2', 'r-1', 'r-0']);
	});

	it('reverses a head sorted low to high, and starts any other low to high', () => {
		expect(nextSort(NAME_ORDER, BY_NAME)).toEqual({ by: BY_NAME, way: 'descending' });
		expect(nextSort({ by: 'days', way: 'descending' }, BY_NAME)).toEqual(NAME_ORDER);
	});

	it('takes the sort off a head sorted high to low, and starts again from none', () => {
		expect(nextSort({ by: 'days', way: 'descending' }, 'days')).toBeNull();
		expect(nextSort(null, 'days')).toEqual({ by: 'days', way: 'ascending' });
	});

	it('goes low to high, high to low, then none, on one head pressed three times', () => {
		const presses = [1, 2, 3].reduce<(ListSort | null)[]>(
			(seen, _press) => [...seen, nextSort(seen.at(-1) ?? null, 'days')],
			[]
		);
		expect(presses).toEqual([
			{ by: 'days', way: 'ascending' },
			{ by: 'days', way: 'descending' },
			null,
		]);
	});

	it('leaves the records as they were handed where nothing is sorted', () => {
		const handed = [...rows(4)].reverse();
		expect(sortRows(handed, null, [DAYS]).map(row => row.id)).toEqual(handed.map(row => row.id));
		expect(sortRows(handed, null, [DAYS])).not.toBe(handed);
	});
});

describe('a drawn column', () => {
	const LINE: ListColumn<Row> = { ...DAYS, id: 'line', head: 'Progress', draw: () => null };

	it('takes the rest of the line after a fixed 14rem name from md, as a column of words does', () => {
		expect(listTracks([LINE, DAYS], 'wide')).toBe('minmax(0,14rem) minmax(0,1fr) max-content');
	});

	it('holds only its head below md, where the row folds and the drawing spans the measure', () => {
		expect(listTracks([LINE], 'narrow')).toBe('minmax(0,1fr) max-content');
	});

	it('folds the rows of a list that has one, and no other list', () => {
		expect(listFolds([LINE])).toBe(true);
		expect(listFolds([DAYS])).toBe(false);
	});
});

describe('a page size of its own', () => {
	it('pages sixty rows as six of ten', () => {
		expect(pageCount(60, 10)).toBe(6);
		expect(pageRows(rows(60), 6, 10).map(row => row.id)).toEqual(
			rows(60)
				.slice(50)
				.map(row => row.id)
		);
		expect(pageSpan(2, 60, 10)).toEqual({ first: 11, last: 20 });
	});

	it('is twenty wherever the caller says nothing', () => {
		expect(pageCount(60)).toBe(pageCount(60, PAGE_SIZE));
		expect(pageSpan(2, 60)).toEqual({ first: 21, last: 40 });
	});
});
