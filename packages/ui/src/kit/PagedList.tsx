import { Button, IconButton, TextButton } from '@app/ui/kit/Button';
import { TextField } from '@app/ui/kit/Field';
import { Figure } from '@app/ui/kit/Figure';
import { PageSection } from '@app/ui/kit/PageSection';
import { PlaceholderRows, Waiting, WaitingNote, useWaitFailed } from '@app/ui/kit/Waiting';
import { ArrowDown, ArrowUp, ChevronLeft, ChevronRight } from 'lucide-react';
import {
	Fragment,
	useId,
	useLayoutEffect,
	useMemo,
	useRef,
	useState,
	useSyncExternalStore,
} from 'react';
import type { CSSProperties, KeyboardEvent } from 'react';
import { CONTROL_H } from '@app/ui/kit/control';
import { FORCED_GROUND, FORCED_PRESS, FORCED_RING } from '@app/ui/kit/focus';
import {
	BY_NAME,
	NAME_ORDER,
	PAGE_SIZE,
	findRows,
	listFolds,
	listTracks,
	nextSort,
	pageCount,
	pageRows,
	pageSpan,
	sortRows,
} from '@app/ui/kit/paged-list';
import type {
	ListColumn,
	ListMark,
	ListSort,
	ListWay,
	ListWords,
	PageSize,
	PagedRecord,
} from '@app/ui/kit/paged-list';
import { CONNECTED_LIFT } from '@app/ui/kit/viewing';

/**
 * A list of records of unknown length: found by name, sorted by any column, read twenty at a time.
 * Its page is `/kit/paged-list`.
 *
 * - **Search** is `TextField`, its label above the box at the full measure, 12px under the band and
 *   16px over the heads. The count is the band's aside and a search changes it to `25 of 212
 *   match`; nothing under the box repeats it.
 * - **The heads are the sort**: a row at the control height, 13px `TextButton`s at weight 500 in
 *   `ui-ink` — a rung and a weight over the rows, so a head never reads as data. A press sorts low
 *   to high, the next reverses it; the sorted head alone carries the arrow.
 * - **Rows** at the control height, the name a link to the record's own page, the figures in Geist
 *   Mono `ui-ink-2`. **Nothing opens in place.** A `ui-beat` hairline between rows 5|6, 10|11
 *   and 15|16, only where a real row follows.
 * - **One continuous column**: a page is as tall as its rows, and the pager follows the last
 *   of them by 8px — two squares 4px apart, then the rows on the page, `21–40`, never `of 212`.
 * - **Every text button in it takes its ground on both sides**, overhanging the measure by
 *   4px, which is the one place the kit overrules `LabelEdge`.
 * - **A row can be bedded** (opt-in `mark`): the `connected` ground behind the rows a caller
 *   names — the notes a project holds among every note, say. The ground is never alone: the row
 *   header carries the caller's word, and the aside counts them.
 *
 * Controlled: every figure it draws comes from `usePagedList`, and it holds only what any control
 * holds (where focus is). Every word is the caller's (`ListWords`).
 */

/* ------------------------------------------------------------------------- *
 * The behaviour
 * ------------------------------------------------------------------------- */

/**
 * Where the hook opens. Each part defaults to rest: nothing searched, page 1, names A to Z. A
 * `sort` of `null` opens sorted by nothing, the records as handed.
 */
export type PagedSeed = { page?: number; query?: string; sort?: ListSort | null };

/** What the list draws. Every figure is derived from the three things the hook holds. */
export type PagedState<R> = {
	/** Every record, before the search. */
	total: number;
	query: string;
	/** The page on screen, 1-based and always inside `1…pages`. */
	page: number;
	pages: number;
	/** `null` while nothing is sorted: the records as handed, and no head carries the arrow. */
	sort: ListSort | null;
	/** The records the search keeps, in the sort's order. */
	found: R[];
	/** The records on this page. */
	rows: R[];
	/** How many rows a page holds at the width on screen: `PAGE_SIZE`, or the caller's `size`. */
	size: number;
};

/** What a hand can do to the list. */
export type PagedHandlers = {
	onQuery: (next: string) => void;
	onClear: () => void;
	onTurn: (page: number) => void;
	/** A head pressed: `BY_NAME` or a column's id. */
	onSort: (by: string) => void;
};

export type PagedHand<R> = PagedState<R> & PagedHandlers;

/**
 * The paged list's behaviour: a search, a page and a sort, and the rules that tie them.
 *
 * - A search never accepts anything; it only narrows. **Any change to it goes back to page 1.**
 * - **A new sort goes back to page 1 and keeps the search.** Page 4 of a list by name is nobody in
 *   particular once it is by a figure, so the page you were on means nothing in the new order.
 * - A head pressed sorts by it low to high; the sorted head pressed again reverses it, and a third
 *   time takes the sort off, the records back as handed (`nextSort`). Every press is a new
 *   sort, so letting go goes back to page 1 too.
 *
 * `columns` should be stable across renders — a module constant, or memoised — since the found
 * list is recomputed whenever it changes. So should `keys`: anything else a record is found by
 * besides its name, such as a note's tags.
 *
 * `size` pages the list shorter below `md`. The hook reads the width itself, so a page never
 * does; and it holds **the first row on screen** rather than the page number, so a page size that
 * changes under the reader lands on the page that still shows the row they were reading.
 */
export function usePagedList<R extends PagedRecord>(
	records: readonly R[],
	columns: readonly ListColumn<R>[],
	seed: PagedSeed = {},
	keys?: (record: R) => readonly string[],
	size?: PageSize
): PagedHand<R> {
	const wide = useWide();
	const per = size === undefined ? PAGE_SIZE : wide ? size.wide : size.narrow;
	const [query, setQuery] = useState(seed.query ?? '');
	const [top, setTop] = useState(((seed.page ?? 1) - 1) * per);
	const [sort, setSort] = useState<ListSort | null>(
		seed.sort === undefined ? NAME_ORDER : seed.sort
	);

	const found = useMemo(
		() => sortRows(findRows(records, query, keys), sort, columns),
		[records, query, sort, columns, keys]
	);
	const pages = pageCount(found.length, per);
	const page = Math.min(Math.max(1, Math.floor(top / per) + 1), pages);

	const onQuery = (next: string) => {
		setQuery(next);
		setTop(0);
	};

	return {
		total: records.length,
		query,
		page,
		pages,
		sort,
		found,
		rows: pageRows(found, page, per),
		size: per,
		onQuery,
		onClear: () => onQuery(''),
		onTurn: next => setTop((Math.min(Math.max(1, next), pages) - 1) * per),
		onSort: by => {
			setSort(current => nextSort(current, by));
			setTop(0);
		},
	};
}

/** `md`, where a list stops folding and pages at its wide size. */
const WIDE = '(min-width: 48rem)';

function subscribeWide(change: () => void): () => void {
	if (typeof window.matchMedia !== 'function') return () => undefined;
	const query = window.matchMedia(WIDE);
	query.addEventListener('change', change);
	return () => query.removeEventListener('change', change);
}

/** Whether the glass is `md` or wider. Wide where there is no media query to ask (a test). */
function useWide(): boolean {
	return useSyncExternalStore(subscribeWide, () =>
		typeof window.matchMedia === 'function' ? window.matchMedia(WIDE).matches : true
	);
}

/* ------------------------------------------------------------------------- *
 * Forced states — for a still, never for an app
 * ------------------------------------------------------------------------- */

/** A pointer or keyboard state, forced for a still: hover and focus are states to draw too. */
export type ListForce = {
	/** The search box. */
	search?: ForceState;
	/** The first figure head. */
	head?: ForceState;
	/** The first row's name. */
	row?: ForceState;
	/** The next arrow. */
	pager?: ForceState;
};

export type ForceState = 'hover' | 'focus' | 'active';

/** The kit's forced grounds and ring (`kit/focus`), with the ink a pointer raises the word to. */
const FORCED: Record<ForceState, string> = {
	hover: `${FORCED_GROUND} text-ui-ink`,
	active: `${FORCED_PRESS} text-ui-ink`,
	focus: FORCED_RING,
};

/**
 * The same, reaching into the two kit controls that take no `className`. Tailwind generates only
 * what it reads in the source, so they are written out.
 */
const FORCED_BUTTON: Record<ForceState, string> = {
	hover: '[&>button]:bg-ui-row [&>button]:text-ui-ink',
	active: '[&>button]:bg-ui-beat [&>button]:text-ui-ink',
	focus:
		'[&>button]:outline-2 [&>button]:outline-offset-2 [&>button]:outline-[var(--color-ui-ink)]',
};

const FORCED_INPUT: Record<ForceState, string> = {
	hover: '[&_input]:bg-ui-row [&_input]:text-ui-ink',
	active: '[&_input]:bg-ui-beat [&_input]:text-ui-ink',
	focus: '[&_input]:outline-2 [&_input]:outline-offset-2 [&_input]:outline-[var(--color-ui-ink)]',
};

function forced(force: ForceState | undefined): string {
	return force === undefined ? '' : FORCED[force];
}

/* ------------------------------------------------------------------------- *
 * The drawing
 * ------------------------------------------------------------------------- */

/**
 * One grid for the heads and every row, so a figure and its head cannot drift apart. The tracks
 * are custom properties set once on the list, because how many figure columns there are is the
 * caller's and a class cannot be written at runtime.
 *
 * The grid is the **table's**, and every row group and row is a subgrid spanning it. A row that
 * was its own grid could only agree with the next one on a fixed width, and a word with its arrow
 * overruns a fixed 4rem — measured, a 56px head cut to 42px the moment it was sorted. Shared, a
 * `max-content` track is the widest of the head and every figure on the page.
 */
const TABLE = 'grid gap-x-4 grid-cols-[var(--ui-list-narrow)] md:grid-cols-[var(--ui-list-wide)]';

/** A row group or a row: the table's own tracks, across all of them. */
const SPAN = 'col-span-full grid grid-cols-subgrid';

/**
 * A row that folds below `md`: the name at the control floor on its own line and the drawing
 * under it, 8px off the next row; from `md` one line at the control height like any other row.
 */
const FOLDED_ROW = 'pb-2 md:pb-0 md:h-ui-touch lg:h-ui-control';

/** A figure cell that is not drawn below `md`. */
function widthOf(column: { narrow: boolean }): string {
	return column.narrow ? '' : 'hidden md:block';
}

/** The arrow and its 4px gap, held beside a figure head that is not the sorted one. */
const ARROW_ROOM = 'w-[18px] shrink-0';

/**
 * A text button's cap in the name's track. With its ground on both sides the button's border box
 * is 8px wider than the word, and `max-w-full` would take those 8px out of a long name.
 */
const FULL_CELL = 'max-w-[calc(100%+0.5rem)]';

/**
 * A marked row's ground: the `connected` band, drawn as a ledger beds what its page has open — the
 * row's exact height, so marked neighbours run as one block, and 12px past the measure on both
 * sides, so it ends clear of the name's first letter and the last figure. The 4px a name's own
 * ground overhangs stands inside it. Behind the row's content, which is why
 * the row isolates: a negative layer with no context of its own sinks under the page's ground.
 */
const BED = 'absolute inset-y-0 -inset-x-3 -z-10 bg-ui-connected';

/** Which way the sorted head runs: up for low to high, down for high to low. */
function SortArrow({ way }: { way: ListWay }) {
	const Arrow = way === 'ascending' ? ArrowUp : ArrowDown;
	return <Arrow aria-hidden className="h-3.5 w-3.5 shrink-0" />;
}

/**
 * One head, and the control that sorts by it.
 *
 * `side` is the side the head stands on: the name's head opens the line, a figure's closes its
 * column. The arrow goes on the **other** side, so a figure head's word stays flush with the
 * figures under it whether or not it is the one sorted, and a name head's word stays on the
 * names' own x.
 *
 * **A figure head keeps the arrow's room when it has no arrow**, outside the button. Its column is
 * `max-content`, so an arrow arriving in it would widen the column and shift every column right of
 * it on the press; held, the sort lands and nothing moves. Held *outside*, so the ground under the
 * pointer is the word's and not the word's plus an empty square. The name's track is the rest
 * of the line and has room either way.
 */
function Head({
	by,
	word,
	side,
	sort,
	still,
	force,
	className = '',
	onSort,
}: {
	by: string;
	word: string;
	side: 'start' | 'end';
	sort: ListSort | null;
	still: boolean;
	force: ForceState | undefined;
	className?: string;
	onSort: (by: string) => void;
}) {
	const way = sort !== null && sort.by === by ? sort.way : null;
	const sorted = way !== null;
	const arrow = way === null ? null : <SortArrow way={way} />;

	return (
		<div
			role="columnheader"
			aria-sort={way ?? undefined}
			className={`min-w-0 items-center ${side === 'end' ? 'justify-end' : ''} ${className}`}
		>
			{side === 'end' && !sorted ? <span aria-hidden className={ARROW_ROOM} /> : null}
			<TextButton
				ink="ink"
				still={still}
				className={`font-medium ${side === 'start' ? `min-w-0 ${FULL_CELL}` : ''} ${forced(force)}`}
				onClick={() => onSort(by)}
			>
				<span className="flex min-w-0 items-center gap-1">
					{side === 'end' ? arrow : null}
					<span className="min-w-0 truncate">{word}</span>
					{side === 'start' ? arrow : null}
				</span>
			</TextButton>
		</div>
	);
}

/** The heads: the name's over the names, then one per figure, right-aligned over it. */
function HeadRow<R>({
	columns,
	words,
	sort,
	still,
	force,
	onSort,
}: {
	columns: readonly ListColumn<R>[];
	words: ListWords;
	sort: ListSort | null;
	still: boolean;
	force: ForceState | undefined;
	onSort: (by: string) => void;
}) {
	return (
		<div role="row" aria-rowindex={1} className={`${SPAN} items-center`}>
			<Head
				by={BY_NAME}
				word={words.name}
				side="start"
				sort={sort}
				still={still}
				force={undefined}
				className="flex"
				onSort={onSort}
			/>
			{columns.map((column, index) =>
				column.words === undefined && column.draw === undefined ? (
					<Head
						key={column.id}
						by={column.id}
						word={column.head}
						side="end"
						sort={sort}
						still={still}
						force={index === 0 ? force : undefined}
						className={column.narrow ? 'flex' : 'hidden md:flex'}
						onSort={onSort}
					/>
				) : (
					/* A column of words is a figure below `md` and a line of words from it, so its
					   head stands where each form stands: over the figures' right edge, and over the
					   words' left one. A drawn column is read the same way — right-aligned beside
					   the name on a folded row's first line, and over the drawing's start from `md`.
					   Two heads, one drawn at each width — `display: none` takes the other out of the
					   grid and out of the reader's table alike. */
					<Fragment key={column.id}>
						<Head
							by={column.id}
							word={column.head}
							side="end"
							sort={sort}
							still={still}
							force={index === 0 ? force : undefined}
							className="flex md:hidden"
							onSort={onSort}
						/>
						<Head
							by={column.id}
							word={column.head}
							side="start"
							sort={sort}
							still={still}
							force={index === 0 ? force : undefined}
							className="hidden md:flex"
							onSort={onSort}
						/>
					</Fragment>
				)
			)}
		</div>
	);
}

/**
 * One record: its name, which is the link to its own page, and its figures. Nothing opens under it.
 * With no `href` the name draws still (`TextButton.still`), exactly where the link stood.
 *
 * `beat` is the pulse: a 1px `ui-beat` hairline laid over the record's last pixel, so it costs the
 * page no height — a page is exactly its rows' control heights with or without it.
 */
function Row<R extends PagedRecord>({
	record,
	columns,
	href,
	index,
	beat,
	marked,
	folds,
	force,
}: {
	record: R;
	columns: readonly ListColumn<R>[];
	href: string | undefined;
	/** The row's place in the whole found list, the head being 1, for a reader paging through. */
	index: number;
	beat: boolean;
	/** The caller's word for a bedded row, or `null` for a row that is not. */
	marked: string | null;
	/** The list has a drawn column, so the row folds below `md`. */
	folds: boolean;
	force: ForceState | undefined;
}) {
	return (
		<div
			role="row"
			aria-rowindex={index}
			className={`relative ${SPAN} ${folds ? FOLDED_ROW : CONTROL_H} items-center ${marked === null ? '' : `isolate ${CONNECTED_LIFT}`}`}
		>
			{marked === null ? null : <span aria-hidden className={BED} />}
			<div role="rowheader" className={`flex min-w-0 ${folds ? 'col-span-full md:col-auto' : ''}`}>
				<TextButton
					href={href}
					still={href === undefined}
					className={`min-w-0 ${FULL_CELL} ${forced(force)}`}
				>
					<span className="flex min-w-0">
						<span className="min-w-0 truncate">{record.name}</span>
					</span>
				</TextButton>
				{marked === null ? null : <span className="sr-only">, {marked}</span>}
			</div>
			{columns.map(column =>
				column.draw !== undefined ? (
					<div key={column.id} role="cell" className="col-span-full min-w-0 md:col-auto">
						{column.draw(record)}
					</div>
				) : column.words === undefined ? (
					<div
						key={column.id}
						role="cell"
						className={`text-right text-ui-body ${column.quiet?.(record) === true ? 'text-ui-ink-3' : 'text-ui-ink-2'} ${widthOf(column)}`}
					>
						{column.plain === true ? (
							<span className="whitespace-nowrap">{column.figure(record)}</span>
						) : (
							<Figure>{column.figure(record)}</Figure>
						)}
					</div>
				) : (
					<WordsCell key={column.id} column={column} record={record} />
				)
			)}
			{beat ? <span aria-hidden className="absolute inset-x-0 bottom-0 h-px bg-ui-beat" /> : null}
		</div>
	);
}

/**
 * A column of words, one record's: its figure below `md`, its words from it. The words are the
 * names' own face and rung, so a line of tags reads as the row's second phrase rather than as
 * data; a record with none prints the column's `none` a rung down, and below `md` its figure the
 * same rung — which is how a record with none stands out without a colour.
 */
function WordsCell<R>({ column, record }: { column: ListColumn<R>; record: R }) {
	const words = column.words?.(record) ?? null;
	return (
		<div role="cell" className="min-w-0 text-right text-ui-body text-ui-ink-2 md:text-left">
			<span className={`md:hidden ${words === null ? 'text-ui-ink-3' : ''}`}>
				<Figure>{column.figure(record)}</Figure>
			</span>
			<span className={`hidden truncate md:block ${words === null ? 'text-ui-ink-3' : ''}`}>
				{words ?? column.none ?? ''}
			</span>
		</div>
	);
}

/**
 * The two arrows and the range. Left-aligned, 4px between the squares and 12px to the words.
 *
 * **An arrow that cannot be pressed points at the range**: `1–20` is why there is no
 * page before this one, and a reader who lands on the dead square hears it.
 *
 * **Focus stays on the arrow that was pressed.** It is the same element before and after the page
 * turns, so it keeps focus by itself — except where the press lands on an end and turns that arrow
 * `disabled`, which drops focus to the document. Then it goes to the other arrow, which the same
 * press has just made live.
 */
function Pager({
	page,
	pages,
	found,
	size,
	loading,
	failed,
	words,
	force,
	onTurn,
}: {
	page: number;
	pages: number;
	found: number;
	size: number;
	loading: boolean;
	/** The wait has failed; its note stands at the list's head instead. */
	failed: boolean;
	words: ListWords;
	force: ForceState | undefined;
	onTurn: (page: number) => void;
}) {
	const box = useRef<HTMLDivElement>(null);
	const pressed = useRef<'previous' | 'next' | null>(null);
	const rangeId = useId();

	useLayoutEffect(() => {
		const which = pressed.current;
		pressed.current = null;
		const [previous, next] = Array.from(box.current?.querySelectorAll('button') ?? []);
		if (which === null || previous === undefined || next === undefined) return;

		const [own, other] = which === 'next' ? [next, previous] : [previous, next];
		const lost = document.activeElement === own || document.activeElement === document.body;
		if (own.disabled && !other.disabled && lost) other.focus();
	}, [page]);

	const turn = (which: 'previous' | 'next') => {
		pressed.current = which;
		onTurn(which === 'next' ? page + 1 : page - 1);
	};

	const atFirst = page <= 1;
	const atLast = page >= pages;
	const { first, last } = pageSpan(page, found, size);

	return (
		<div
			ref={box}
			role="group"
			aria-label={loading ? undefined : words.pageName(page, pages)}
			className="mt-2 flex items-center gap-3"
		>
			<div className="flex items-center gap-1">
				<IconButton
					label={words.previous}
					disabled={loading || atFirst}
					describedBy={!loading && atFirst ? rangeId : undefined}
					onClick={() => turn('previous')}
				>
					<ChevronLeft aria-hidden className="h-4 w-4" />
				</IconButton>
				<span className={`flex ${force === undefined ? '' : FORCED_BUTTON[force]}`}>
					<IconButton
						label={words.next}
						disabled={loading || atLast}
						describedBy={!loading && atLast ? rangeId : undefined}
						onClick={() => turn('next')}
					>
						<ChevronRight aria-hidden className="h-4 w-4" />
					</IconButton>
				</span>
			</div>
			{loading ? (
				failed ? null : (
					<WaitingNote />
				)
			) : (
				<span id={rangeId} role="status" className="text-ui-body text-ui-ink-3">
					{words.range(first, last)}
				</span>
			)}
		</div>
	);
}

export type PagedListProps<R extends PagedRecord> = PagedHand<R> & {
	/** The band's title, and the table's accessible name. */
	title: string;
	words: ListWords;
	/** The figures after the name, in order. Stable across renders, as `usePagedList` asks. */
	columns: readonly ListColumn<R>[];
	/**
	 * Where a record's name goes: its own page. Left out, every name draws still — a list whose
	 * records have no page yet.
	 */
	href?: (record: R) => string;
	/** The records have been asked for and have not arrived. */
	loading?: boolean;
	/**
	 * The records to bed, and the word a reader hears on each. Opt-in: unset, no row is
	 * marked.
	 */
	mark?: ListMark<R>;
	/** Pointer and keyboard states, forced for a still. Never set by an app. */
	force?: ListForce;
};

/**
 * The list, drawn from what `usePagedList` hands it. The ordinary caller is
 * `<PagedList title words columns href {...usePagedList(records, columns)} />`.
 *
 * It is a table to a reader — heads, `aria-sort` on the sorted one, and every row's place in the
 * whole found list — and a grid to the eye, one track list shared by the heads and the rows.
 */
export function PagedList<R extends PagedRecord>({
	title,
	words,
	columns,
	href,
	loading = false,
	mark,
	force = {},
	total,
	query,
	page,
	pages,
	sort,
	found,
	rows,
	size,
	onQuery,
	onClear,
	onTurn,
	onSort,
}: PagedListProps<R>) {
	const field = useRef<HTMLDivElement>(null);
	const failed = useWaitFailed();

	const searching = query.trim() !== '';
	const aside = loading
		? undefined
		: searching
			? words.match(found.length, total)
			: words.count(total);

	const tracks = {
		'--ui-list-narrow': listTracks(columns, 'narrow'),
		'--ui-list-wide': listTracks(columns, 'wide'),
	} as CSSProperties;

	const onKey = (event: KeyboardEvent) => {
		if (event.key !== 'Escape' || query === '') return;
		event.preventDefault();
		onClear();
	};

	const clearAndFind = () => {
		onClear();
		field.current?.querySelector('input')?.focus();
	};

	if (!loading && total === 0) {
		return (
			<div className="relative" style={tracks}>
				<PageSection title={title} aside={aside}>
					<p role="status" className="sr-only">
						{aside}
					</p>
					<p className="mt-3 text-ui-body text-ui-ink-3">{words.empty}</p>
				</PageSection>
			</div>
		);
	}

	const first = (page - 1) * size;
	const folds = listFolds(columns);

	return (
		<div className="relative" style={tracks}>
			<PageSection title={title} aside={aside}>
				<p role="status" className="sr-only">
					{aside}
				</p>

				<div
					ref={field}
					className={`mt-3 ${force.search === undefined ? '' : FORCED_INPUT[force.search]}`}
					onKeyDown={onKey}
				>
					<TextField
						label={words.find}
						value={loading ? '' : query}
						disabled={loading}
						onChange={onQuery}
					/>
				</div>

				<div
					role="table"
					aria-label={title}
					aria-rowcount={loading ? undefined : found.length + 1}
					className={`${TABLE} mt-4`}
				>
					<div role="rowgroup" className={SPAN}>
						<HeadRow
							columns={columns}
							words={words}
							sort={sort}
							still={loading}
							force={force.head}
							onSort={onSort}
						/>
					</div>
					{loading || found.length === 0 ? null : (
						<div role="rowgroup" className={SPAN}>
							{rows.map((record, index) => (
								<Row
									key={record.id}
									record={record}
									columns={columns}
									href={href?.(record)}
									index={first + index + 2}
									beat={(index + 1) % 5 === 0 && index + 1 < rows.length}
									marked={mark?.rows(record) === true ? mark.word : null}
									folds={folds}
									force={index === 0 ? force.row : undefined}
								/>
							))}
						</div>
					)}
				</div>

				{loading ? (
					<Waiting>
						{/*
						 * A failed wait says so at the list's head, not in the pager under twenty rows,
						 * where a tall list would put it below the fold.
						 */}
						{failed ? (
							<div className="flex items-center justify-end pt-2 pb-1">
								<WaitingNote />
							</div>
						) : null}
						<PlaceholderRows rows={size} />
					</Waiting>
				) : found.length === 0 ? (
					<div>
						<p className={`flex ${CONTROL_H} items-center text-ui-body text-ui-ink-2`}>
							<span className="min-w-0 break-words">{words.noMatch(query)}</span>
						</p>
						<div className="mt-2 flex">
							<Button onClick={clearAndFind}>{words.clear}</Button>
						</div>
					</div>
				) : null}

				{loading || found.length > 0 ? (
					<Pager
						page={page}
						pages={pages}
						found={found.length}
						size={size}
						loading={loading}
						failed={failed}
						words={words}
						force={force.pager}
						onTurn={onTurn}
					/>
				) : null}
			</PageSection>
		</div>
	);
}
