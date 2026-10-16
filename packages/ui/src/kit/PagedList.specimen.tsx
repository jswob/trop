import type { Subject } from '@jswob/design-workbench';
import { specimenOf } from '@jswob/design-workbench';
import type { ListColumn, ListForce, ListMark, ListSort, ListWords } from '@app/ui/kit';
import { BY_NAME, NAME_ORDER, compareNames, PagedList, SCREEN_H, usePagedList } from '@app/ui/kit';

/**
 * The crew list `kit/PagedList`'s page draws, as data.
 *
 * The variant grid is **enumerated from a config** rather than hand-listed, so the page cannot
 * quietly stop showing a state the component still has. Every state the page and the frames draw
 * is `LIST_STATES`, and nothing is written in the page.
 *
 * **It is not the app**: a specimen is drawn on content that is not the app's. Here it is a film
 * unit's crew list, 212 people long, two figures per person and a page of their own behind each
 * name.
 *
 * Every record is made arithmetically rather than at random, so page 4 today is page 4 tomorrow.
 */

/** One person on the unit's crew list. */
export type CrewMember = {
	id: string;
	name: string;
	/** Days called over the shoot. */
	days: number;
	/** Hours of overtime, to the half hour. */
	overtime: number;
	/** On tomorrow's call sheet — the rows the `marked` state beds. */
	called: boolean;
};

const FIRST = [
	'Ada',
	'Aleksander',
	'Beatriz',
	'Bruno',
	'Chidi',
	'Dalia',
	'Dorota',
	'Emil',
	'Farah',
	'Grzegorz',
	'Hana',
	'Ines',
	'Jakub',
	'Kofi',
	'Lena',
	'Mateo',
	'Marta',
	'Nadia',
	'Oskar',
	'Priya',
	'Rafał',
	'Sade',
	'Tomasz',
	'Una',
	'Wiktoria',
	'Yusuf',
	'Zofia',
	'Ingrid',
];

const LAST = [
	'Abara',
	'Brennan',
	'Castillo',
	'Dąbrowski',
	'Eriksen',
	'Fischer',
	'Górecka',
	'Haddad',
	'Idrissi',
	'Jankowska',
	'Kowalczyk',
	'Lindqvist',
	'Mensah',
	'Novak',
	'Okonjo',
	'Pereira',
	'Quinn',
	'Ruiz',
	'Salles',
	'Tanaka',
	'Ueda',
	'Varga',
	'Wiśniewska',
	'Zieliński',
];

function member(index: number, name: string): CrewMember {
	return {
		id: `crew-${String(index)}`,
		name,
		days: 1 + ((index * 17) % 38),
		overtime: ((index * 29) % 46) / 2,
		called: (index * 11 + Math.floor(index / FIRST.length)) % 7 < 2,
	};
}

/** 212 people: eleven pages of twenty, the last one twelve long. Sorted by name, as a crew list is. */
export const CREW: readonly CrewMember[] = Array.from({ length: 212 }, (_unused, index) => {
	const first = FIRST[index % FIRST.length] ?? 'Ada';
	const last = LAST[(index * 7 + Math.floor(index / FIRST.length)) % LAST.length] ?? 'Abara';
	return member(index, `${first} ${last}`);
}).sort(compareNames);

/** Fourteen people: everything fits on one page. */
export const SHORT_CREW: readonly CrewMember[] = CREW.filter((_row, index) => index % 16 === 0);

/**
 * Long names at the head of a page. The list draws a name whole to **28** characters; these are
 * 28, 31 and 60, so one clears, one cuts and one is far past anything the slot was drawn for.
 */
const LONG_NAMES = [
	'Maximiliano Echeverría Solís',
	'Aleksandra Wiśniewska-Kowalczyk',
	'Konstantynopolitańczykiewiczówna-Brzęczyszczykiewicz Jadwiga',
];

export const LONG_CREW: readonly CrewMember[] = [
	...LONG_NAMES.map((name, index) => member(900 + index, name)),
	...CREW.slice(0, 17),
];

const HOURS = (hours: number) => (hours === 0 ? '0' : hours.toFixed(1));

/** The two figures a crew list carries. Days stays at every width; overtime drops below `md`. */
export const CREW_COLUMNS: ListColumn<CrewMember>[] = [
	{
		id: 'days',
		head: 'Days',
		figure: row => String(row.days),
		value: row => row.days,
		narrow: true,
	},
	{
		id: 'overtime',
		head: 'Overtime',
		figure: row => HOURS(row.overtime),
		value: row => row.overtime,
		narrow: false,
	},
];

/**
 * Where a name goes: that person's own page. The specimen has no such page, so the link is a
 * fragment — pressing it in the workbench moves the address and nothing else, where a path would
 * land on the workbench's own not-found.
 */
export function crewHref(row: CrewMember): string {
	return `#person-${row.id}`;
}

export const LIST_TITLE = 'Crew';

/** Every word a crew list prints. The kit owns none of them. */
export const CREW_WORDS: ListWords = {
	name: 'Name',
	find: 'Find by name',
	clear: 'Clear search',
	previous: 'Previous page',
	next: 'Next page',
	empty: 'Nobody is on the crew list yet.',
	count: total => (total === 0 ? 'none' : `${String(total)} ${total === 1 ? 'person' : 'people'}`),
	match: (found, total) =>
		`${String(found)} of ${String(total)} ${found === 1 ? 'matches' : 'match'}`,
	noMatch: query => `No one matches “${query.trim()}”.`,
	range: (first, last) => (first === last ? String(first) : `${String(first)}–${String(last)}`),
	pageName: (page, pages) => `Page ${String(page)} of ${String(pages)}`,
};

/** The rows the `marked` state beds: everyone on tomorrow's call sheet. */
export const CALLED: ListMark<CrewMember> = { rows: row => row.called, word: 'called tomorrow' };

/** How many of the crew are called — the bed's count in words, which it is never without. */
const CALLED_COUNT = CREW.filter(row => row.called).length;

/** The crew list's words with the aside counting the bedded rows against the whole crew. */
const CALLED_WORDS: ListWords = {
	...CREW_WORDS,
	count: total => `${String(CALLED_COUNT)} of ${String(total)} called tomorrow`,
};

/**
 * Where one state opens: the records, the page on screen, what is searched, what it is sorted by,
 * `loading` — the list waiting on records it does not have yet — what is marked, with the words
 * that count it, and what is forced.
 */
export type ListSeed = {
	records: readonly CrewMember[];
	page: number;
	query: string;
	sort: ListSort | null;
	loading: boolean;
	mark: ListMark<CrewMember> | undefined;
	words: ListWords;
	force: ListForce;
};

const SEED: ListSeed = {
	records: CREW,
	page: 1,
	query: '',
	sort: NAME_ORDER,
	loading: false,
	mark: undefined,
	words: CREW_WORDS,
	force: {},
};

/** Twenty-five matches: a search that still needs a second page. */
const QUERY_MANY = 'ska';
/** A handful: a search that fits in one page. */
const QUERY_FEW = 'sade';
/** Nothing. */
const QUERY_NONE = 'zzq';

/** One state of the list: an id, what it is, and where it opens. */
export type ListState = { id: string; label: string; note: string; seed: ListSeed };

/**
 * Every state the list owes.
 *
 * `disabled` and `invalid` are not here and that is an answer rather than a gap: the list is never
 * switched off as a whole — a surface being read rather than worked takes its targets away, which
 * is a different drawing — and a search refuses nothing, so there is nothing for it to be invalid
 * about. A pager's own end, where there is no page to go to, is drawn on `first` and `last`.
 */
export const LIST_STATES: ListState[] = [
	{
		id: 'first',
		label: 'First page',
		note: 'Page 1 of 11, names A to Z. The previous arrow is off.',
		seed: SEED,
	},
	{
		id: 'middle',
		label: 'A middle page',
		note: 'Page 4 of 11. Both ways are open.',
		seed: { ...SEED, page: 4 },
	},
	{
		id: 'last',
		label: 'Last page',
		note: 'Twelve rows, and the pager 8px under the last of them.',
		seed: { ...SEED, page: 11 },
	},
	{
		id: 'sort-figure',
		label: 'Sorted by a figure',
		note: 'Days pressed twice: most first, a tie in name order.',
		seed: { ...SEED, sort: { by: 'days', way: 'descending' } },
	},
	{
		id: 'sort-name',
		label: 'Names reversed',
		note: 'Name pressed on a list already by name: Z to A.',
		seed: { ...SEED, sort: { by: BY_NAME, way: 'descending' } },
	},
	{
		id: 'unsorted',
		label: 'Sorted by nothing',
		note: 'A third press lets go: records as handed, no arrow.',
		seed: { ...SEED, records: LONG_CREW, sort: null },
	},
	{
		id: 'sort-search',
		label: 'Sorted and searching',
		note: 'By overtime, least first. A sort keeps the search.',
		seed: { ...SEED, query: QUERY_MANY, sort: { by: 'overtime', way: 'ascending' } },
	},
	{
		id: 'search',
		label: 'Searching',
		note: 'A search that finds more than one page.',
		seed: { ...SEED, query: QUERY_MANY },
	},
	{
		id: 'search-few',
		label: 'Searching, one page',
		note: 'A search that fits on one page.',
		seed: { ...SEED, query: QUERY_FEW },
	},
	{
		id: 'no-match',
		label: 'Nothing found',
		note: 'Nobody found: the sentence, and the one way out of it.',
		seed: { ...SEED, query: QUERY_NONE },
	},
	{
		id: 'one-page',
		label: 'One page',
		note: 'Fourteen people: all fit, so both arrows are off.',
		seed: { ...SEED, records: SHORT_CREW },
	},
	{
		id: 'empty',
		label: 'Empty',
		note: 'Nobody yet: nothing to search and nothing to page.',
		seed: { ...SEED, records: [] },
	},
	{
		id: 'loading',
		label: 'Loading',
		note: 'The records are asked for and have not arrived.',
		seed: { ...SEED, loading: true },
	},
	{
		id: 'long',
		label: 'Long names',
		note: 'Names of 28, 31 and 60 characters: one whole, two cut.',
		seed: { ...SEED, records: LONG_CREW },
	},
	{
		id: 'marked',
		label: 'Marked rows',
		note: 'The people called tomorrow, on the connected ground.',
		seed: { ...SEED, mark: CALLED, words: CALLED_WORDS },
	},
	{
		id: 'marked-hover',
		label: 'Marked, under a pointer',
		note: 'Forced: a bedded name and a head under a pointer.',
		seed: {
			...SEED,
			mark: CALLED,
			words: CALLED_WORDS,
			force: { head: 'hover', row: 'hover' },
		},
	},
	{
		id: 'hover',
		label: 'Hover',
		note: 'Forced: a head, a name and the next arrow, hovered.',
		seed: { ...SEED, page: 4, force: { head: 'hover', row: 'hover', pager: 'hover' } },
	},
	{
		id: 'active',
		label: 'Pressed',
		note: 'Forced: a head, a name and the next arrow, pressed.',
		seed: { ...SEED, page: 4, force: { head: 'active', row: 'active', pager: 'active' } },
	},
	{
		id: 'focus-search',
		label: 'Focus on the search',
		note: 'Forced: the search box holding the keyboard.',
		seed: { ...SEED, query: QUERY_MANY, force: { search: 'focus' } },
	},
	{
		id: 'focus-head',
		label: 'Focus on a head',
		note: 'Forced: the Days head holding the keyboard.',
		seed: { ...SEED, force: { head: 'focus' } },
	},
	{
		id: 'focus-row',
		label: 'Focus on a name',
		note: 'Forced: the first name holding the keyboard.',
		seed: { ...SEED, force: { row: 'focus' } },
	},
	{
		id: 'focus-pager',
		label: 'Focus on paging',
		note: 'Forced: the next arrow holding the keyboard.',
		seed: { ...SEED, page: 4, force: { pager: 'focus' } },
	},
];

/**
 * `kit/PagedList` and `usePagedList` as frames.
 *
 * Every state is a frame, `kit-paged-list:<state>`, so each has an address a probe or a screenshot
 * can open at true pixel width. Below `md` the list drops a column, which only a frame at 390 can
 * show.
 */

/** One state of the crew list, seeded and then live. */
function Specimen({ seed }: { seed: ListSeed }) {
	const hand = usePagedList(seed.records, CREW_COLUMNS, {
		page: seed.page,
		query: seed.query,
		sort: seed.sort,
	});

	return (
		<PagedList
			title={LIST_TITLE}
			words={seed.words}
			columns={CREW_COLUMNS}
			href={crewHref}
			loading={seed.loading}
			mark={seed.mark}
			force={seed.force}
			{...hand}
		/>
	);
}

/**
 * The page a frame draws the list on: the ground, the page's own margins — 20px below `lg`, 32px
 * from it — and the 32rem measure.
 *
 * **It is its own scroller.** A bare frame stands at `h-dvh` and scrolls nothing, and twenty rows at
 * 40px are 800px of list in an 844px phone — so a stage that only grew would run the pager off the
 * foot of the glass, where no hand could reach it.
 */
function FrameStage({ seed }: { seed: ListSeed }) {
	return (
		<div className={`${SCREEN_H} overflow-y-auto bg-ui-bg px-ui-gutter py-8 lg:px-8`}>
			<div className="max-w-[32rem]">
				<Specimen seed={seed} />
			</div>
		</div>
	);
}

/** The list's own frames, one per state, registered so each one has a URL. */
export const pagedListFrames: Subject = {
	id: 'kit-paged-list',
	title: 'PagedList — specimens',
	description:
		'A list of unknown length: found by name, sorted by any column, read twenty at a time.',
	chrome: 'bare',
	variants: LIST_STATES.map(state => ({
		id: state.id,
		label: state.label,
		note: state.note,
		render: () => <FrameStage seed={state.seed} />,
	})),
};

export default specimenOf(pagedListFrames, {
	kit: 'paged-list',
	props: {
		title: 'The band’s title, and the table’s accessible name.',
		words: 'Every word it prints. The kit owns none of them.',
		columns: 'The figures after the name, in order. Stable across renders.',
		href: 'Where a record’s name goes: its own page. Left out, every name draws still.',
		loading: 'The records have been asked for and have not arrived.',
		mark: 'The records to bed on the connected ground, and the word a reader hears on each.',
		force: 'A pointer or keyboard state, forced for a still. Never set by an app.',
		total: 'Every record, before the search.',
		query: 'What is searched.',
		page: 'The page on screen, from 1 and never past the last.',
		pages: 'How many pages the found records make.',
		sort: 'The column sorted by and which way. Null is the records as handed.',
		found: 'The records the search keeps, in the sort’s order.',
		rows: 'The records on this page.',
		size: 'How many rows a page holds at the width on screen.',
		onQuery: 'What typing in the search does. Any change goes back to page 1.',
		onClear: 'What Escape and Clear search do.',
		onTurn: 'What an arrow does: the page to go to.',
		onSort: 'What a press on a head does. A new sort goes back to page 1.',
	},
	holds: [
		'It pages with two arrows and the range under the rows, twenty to a page.',
		'A page is as tall as its rows, and the pager follows the last row by 8px.',
		'The pager’s words are the rows on the page, 21–40, never of 212.',
		'A head is 13px at weight 500 in the first ink, over regular rows a rung down.',
		'Every text button in it takes its ground on both sides, 4px past the word.',
		'A name is a link to its record’s own page.',
	],
	refuses: [
		'Numbered page cells.',
		'A press that grows the page, Show 20 more.',
		'A row that opens in place.',
		'A rows area held at twenty rows’ height, which left 256px empty under the last page.',
		'The count said twice, 1–20 of 212.',
		'Figure columns at a fixed 4rem, which the sorted head’s arrow overran.',
	],
});
