/**
 * What `kit/PagedList` draws, and the arithmetic behind it — pure, so it is tested without a DOM.
 *
 * The specimen's records are its own, in `PagedList.specimen`: a kit module shipping its own records
 * would be a component with content in it.
 *
 * **Nothing here is a word.** Every sentence the list prints is the caller's, handed in as
 * `ListWords`: `212 notes` is one list's count and `14 projects` the next caller's. What is here is
 * what a sentence is made from — how many pages, which rows are on one, where a page starts and ends.
 */

import type { ReactNode } from 'react';

/** The least a record owes the list: something to key it by, and a name to find it by. */
export type PagedRecord = { id: string; name: string };

/**
 * One figure column. `narrow: false` is a column the list drops below `md`, where the name has to
 * keep the measure; `true` stays at every width. The figure is already in words — the list prints
 * it and never formats it — and `value` is the number it sorts by, because `12.5` and `9` sort
 * wrongly as words.
 */
export type ListColumn<R> = {
	id: string;
	head: string;
	figure: (record: R) => string;
	value: (record: R) => number;
	narrow: boolean;
	/**
	 * **A column of words** — a record's tags, say. From `md` the cell prints these in the
	 * name's own face, `ui-ink-2`, truncating where they run out, and the column takes whatever the
	 * line has left after a whole name; below `md`, where they cannot stand whole beside a whole name,
	 * the cell prints `figure` as any figure column does. It sorts by `value` at both widths, so the
	 * column is `narrow`: drawn everywhere, only its form changes. `null` prints `none` in `ui-ink-3`.
	 */
	words?: (record: R) => string | null;
	/** What a column of words prints for a record with none: `No tags`. */
	none?: string;
	/**
	 * **A drawn column** — a bar per row, say. The caller draws the cell, and the list
	 * only gives it room: from `md` the column takes the rest of the line after a name track fixed at
	 * `14rem`, as a column of words does; below `md` the row **folds** — the name alone on its own
	 * line, at the control floor, and the drawing under it across the whole measure — because a
	 * drawing squeezed in beside a name at 350px is a drawing nobody can read. The head keeps the
	 * first line at both widths. It sorts by `value`, so it is `narrow`: drawn everywhere, and `figure`
	 * is what a reader hears in its place.
	 */
	draw?: (record: R) => ReactNode;
	/**
	 * **A figure column whose values are words** — `Pinned` / `Not pinned`, say. Printed as a
	 * figure is, right-aligned under its head at every width, but in the names' own face: Geist Mono
	 * is for figures that align, and a word is not one. `quiet`
	 * sets a record's value a rung down, `ui-ink-3`, as a column of words sets its `none`.
	 */
	plain?: boolean;
	quiet?: (record: R) => boolean;
};

/**
 * The records a list beds, and the word a reader hears on each of them.
 *
 * The bed is the `connected` ground — a band behind a row related to the record the page has
 * open — and it is 1.36:1 off the ground, so it is **never the only thing that says so**: `word`
 * goes into each marked row's header for a reader, and the band's aside should count the marked
 * records in words (`ListWords.count` is the caller's, and so is that sentence).
 */
export type ListMark<R> = {
	rows: (record: R) => boolean;
	word: string;
};

/** Twenty rows: four beats of five. */
export const PAGE_SIZE = 20;

/**
 * **How many rows a page holds at each width**, opt-in. A list whose rows fold below `md` is
 * twice as tall there, so a caller may page it shorter on a phone — ten below `md` and twenty from
 * it keeps whatever stands under the list about a screen away. Unset, a
 * page is `PAGE_SIZE` at every width.
 */
export type PageSize = { narrow: number; wide: number };

/** Which way a column is sorted: low to high, or high to low. */
export type ListWay = 'ascending' | 'descending';

/** The sort key of the name's own column — no figure column may take it as its id. */
export const BY_NAME = 'name';

/**
 * What the list is sorted by: `BY_NAME` or a column's id, and which way. Where a list holds a sort
 * it is `ListSort | null`: `null` is a list sorted by nothing, its records as the caller handed
 * them.
 */
export type ListSort = { by: string; way: ListWay };

/** Where every list starts: names, A to Z. */
export const NAME_ORDER: ListSort = { by: BY_NAME, way: 'ascending' };

/**
 * Every word the list prints, handed in. The kit owns none of them.
 *
 * `range` is handed the numbers and never the page size: `21–40` is a sentence, and where page 2
 * starts is the list's.
 */
export type ListWords = {
	/** The head over the names. */
	name: string;
	/** The search box's label. */
	find: string;
	/** The one way out of a search that found nothing. */
	clear: string;
	previous: string;
	next: string;
	/** The whole of a list with no records at all. */
	empty: string;
	/** The band's aside at rest: `212 notes`. */
	count: (total: number) => string;
	/** The band's aside while a search stands: `25 of 212 match`. */
	match: (found: number, total: number) => string;
	/** Where a search found nothing, quoting it back. */
	noMatch: (query: string) => string;
	/** The rows on the page, 1-based and inclusive: `21–40`. Never the total. */
	range: (first: number, last: number) => string;
	/** The pager's group name, for a reader: `Page 2 of 11`. */
	pageName: (page: number, pages: number) => string;
};

/** Names in reading order, case and accents aside — the one order a name sorts by. */
const NAMES = new Intl.Collator(undefined, { sensitivity: 'base' });

/** Two names in reading order; two records with one name by their ids, so the order never wobbles. */
export function compareNames(a: PagedRecord, b: PagedRecord): number {
	return NAMES.compare(a.name, b.name) || (a.id < b.id ? -1 : a.id > b.id ? 1 : 0);
}

/** A string as a search sees it: case and diacritics gone, so `wisn` finds `Wiśniewska`. */
function folded(text: string): string {
	return text
		.normalize('NFD')
		.replace(/\p{M}/gu, '')
		.replace(/ł/g, 'l')
		.replace(/Ł/g, 'L')
		.toLowerCase();
}

/**
 * The records a search keeps: every one whose name contains what was typed, anywhere in it,
 * ignoring case and diacritics. An empty or blank search keeps everything. Order is the input's.
 *
 * `keys` is anything else a record is found by: a list may find a note by its tags as well as by
 * its title, so `home` keeps every note tagged Home. Folded the same way, and a record that matches
 * twice is kept once.
 */
export function findRows<R extends { name: string }>(
	rows: readonly R[],
	query: string,
	keys?: (row: R) => readonly string[]
): R[] {
	const needle = folded(query.trim());
	if (needle === '') return [...rows];
	const hit = (text: string) => folded(text).includes(needle);
	return rows.filter(row => hit(row.name) || (keys?.(row).some(hit) ?? false));
}

/**
 * The list's track list at one width: the name's, then one per column drawn there.
 *
 * The name takes the rest of the line and every figure its widest (`max-content`), which is what
 * lets a head and its figures share one column. **A column of words changes that from `md`**: it
 * takes the rest of the line instead, and the name is a fixed `14rem` — a whole name of 28
 * characters with its ground on both sides, the longest the slot holds — so the words start at one
 * x on every page, whatever names are on it. A track sized to the page's longest name moves the
 * column between two pages of one list (133px, measured), which is a size taken from the data.
 * Below `md` a column of words is a figure like any other.
 */
export function listTracks<R>(columns: readonly ListColumn<R>[], width: 'narrow' | 'wide'): string {
	const drawn = width === 'wide' ? columns : columns.filter(column => column.narrow);
	const fills = (column: ListColumn<R>) => column.words !== undefined || column.draw !== undefined;
	const worded = width === 'wide' && drawn.some(fills);
	return [
		worded ? 'minmax(0,14rem)' : 'minmax(0,1fr)',
		...drawn.map(column => (worded && fills(column) ? 'minmax(0,1fr)' : 'max-content')),
	].join(' ');
}

/** Whether a list folds its rows below `md`: it has a drawn column. */
export function listFolds<R>(columns: readonly ListColumn<R>[]): boolean {
	return columns.some(column => column.draw !== undefined);
}

/** How many pages `total` rows take. Never fewer than one — an empty list is one empty page. */
export function pageCount(total: number, size = PAGE_SIZE): number {
	return Math.max(1, Math.ceil(total / size));
}

/** A page asked for, held inside `1…pages`. */
function clampPage(page: number, total: number, size: number): number {
	return Math.min(Math.max(1, page), pageCount(total, size));
}

/** The rows on one page, 1-based. A page past the end is the last page. */
export function pageRows<R>(rows: readonly R[], page: number, size = PAGE_SIZE): R[] {
	const at = clampPage(page, rows.length, size);
	return rows.slice((at - 1) * size, at * size);
}

/**
 * Where one page starts and ends in the found list, 1-based and inclusive — what `ListWords.range`
 * is handed. Page 11 of 212 is `201…212`.
 */
export function pageSpan(
	page: number,
	total: number,
	size = PAGE_SIZE
): { first: number; last: number } {
	const at = clampPage(page, total, size);
	return { first: (at - 1) * size + 1, last: Math.min(at * size, total) };
}

/**
 * The records in the order a sort asks for. A figure sorts by its `value`; **a tie falls back to
 * the name, A to Z, whichever way the figure runs**, so reversing Words does not also reverse the
 * order of every note of eleven words. A sort key no column has is the name's. **No sort leaves the
 * records as they were handed** — the caller's own resting order.
 */
export function sortRows<R extends PagedRecord>(
	rows: readonly R[],
	sort: ListSort | null,
	columns: readonly ListColumn<R>[]
): R[] {
	if (sort === null) return [...rows];
	const column = columns.find(each => each.id === sort.by);
	const sign = sort.way === 'ascending' ? 1 : -1;

	return [...rows].sort((a, b) => {
		if (column === undefined) return sign * compareNames(a, b);
		return sign * (column.value(a) - column.value(b)) || compareNames(a, b);
	});
}

/**
 * The way a head's next press sorts: **low to high, then high to low, then nothing**. Any
 * head that is not the sorted one starts low to high; the sorted head reverses once and then lets
 * go, so a sort is never on for good once a head has been pressed.
 */
export function nextSort(current: ListSort | null, by: string): ListSort | null {
	if (current === null || current.by !== by) return { by, way: 'ascending' };
	return current.way === 'ascending' ? { by, way: 'descending' } : null;
}
