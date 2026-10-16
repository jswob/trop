/**
 * **The spread's arithmetic, with no DOM in it.**
 *
 * `useSpread` in `kit/PageSpread` is three things at once: a measurement of the box, a search for a
 * height, and the browser writes that carry the answer. Only the first two are rules — *how many
 * columns fit*, and *what heights the search is allowed to end on* — and a rule that cannot be
 * asserted without a browser is a rule nobody checks. So they are here, as functions of numbers.
 *
 * The numbers themselves are `PageSpread`'s and are re-exported from it, not redeclared here.
 */

/**
 * The measure a column is set at, and the one margin the page spends on every edge.
 *
 * `MEASURE` is `views/page-shared`'s `FORM_MEASURE` as a number: the document's body is capped at
 * 32rem and the column is set at the same, so the page and the column it is on are one number.
 * `EDGE` is the gutter the scroller already carried, and `GAP` is twice it, so a rule centred in a
 * gutter stands exactly one edge from the column on either side.
 */
export const MEASURE = 512;
export const EDGE = 32;
export const GAP = EDGE * 2;

/** What the head and the foot margin cost a column between them. */
export const PAGE = EDGE * 2;

/**
 * **How many columns there is room for.** The whole of the rule *columns split only when there is
 * room for another column*.
 *
 * As many whole columns of the measure as the width will take, and never fewer than one. It is the
 * inverse of `blockWidth`: `blockWidth(columnsFor(room)) <= room` for every `room`, which is what
 * stops a column being laid out past the edge of the glass.
 */
export function columnsFor(room: number): number {
	return Math.max(1, Math.floor((room - 2 * EDGE + GAP) / (MEASURE + GAP)));
}

/**
 * What the block is set to for a given column count — the columns, their gutters and the margin,
 * and nothing else. The surplus past it stays in one piece at the right-hand edge, which is the
 * difference between a spread and two panes.
 */
export function blockWidth(columns: number): number {
	return 2 * EDGE + columns * MEASURE + (columns - 1) * GAP;
}

/**
 * **The two ends of the height search.**
 *
 * `ideal` is the arithmetic answer — the flow divided between the columns — and it is a *floor*: a
 * section that will not fit whole at the foot of a column moves down entire, so the real packing is
 * always a little worse.
 *
 * `whole` is the flow in a single column, and it is the reason the search is safe. Every one of the
 * columns can hold it, because the flow measured `length` tall at exactly this measure — so it ends
 * inside column one and the rest stand empty. A search that only ever settles on a height between
 * these two, and only after measuring that the height does not spill, cannot end on one that does.
 *
 * `ideal` is clamped to `whole` for the one-column case, where the two are the same number and the
 * glass may be taller than either.
 *
 * `unbroken` is the tallest `UNBROKEN` block's height, 0 where there is none: `ideal` never goes
 * below it plus the page's margin, so the search never tries a column that block cannot stand in.
 */
export function searchBounds(
	length: number,
	columns: number,
	glass: number,
	unbroken = 0
): { ideal: number; whole: number } {
	const whole = length + PAGE;
	const least = unbroken === 0 ? 0 : unbroken + PAGE;

	return {
		ideal: Math.min(Math.max(glass, Math.ceil(length / columns) + PAGE, least), whole),
		whole,
	};
}

/**
 * Whether the flow fits the columns at the height of the glass, in which case the foot **is** the
 * glass: fill column one to the bottom of the screen, start the second with what is left, and a
 * page shorter than the glass is a one-column page with no rule drawn in it at all.
 *
 * Arithmetic only — the caller still measures, because `break-inside: avoid` means a flow that
 * divides on paper can still spill.
 */
export function fitsTheGlass(
	length: number,
	columns: number,
	glass: number,
	unbroken = 0
): boolean {
	const held = glass - PAGE;

	return held > 0 && Math.ceil(length / held) <= columns && unbroken <= held;
}

/**
 * **The attribute a block carries to say the column rule may never fall inside it, however tall
 * it is**: a chart and the switch above it, say, that must only ever be in one column.
 *
 * `break-inside: avoid` alone cannot promise that. It moves a block to the next column, and a
 * block taller than a column is divided anyway, because the search ends on the shortest height
 * that does not *spill*, and a divided block spills nothing. So `useSpread` measures the tallest
 * block marked with this, while the flow is one column and nothing is divided, and hands it to
 * `fitsTheGlass` and `searchBounds` as `unbroken`: no column is ever shorter than it.
 *
 * **Opt-in, by attribute.** `PageSection`'s `whole` is not read, so a page that marks nothing packs
 * exactly as it would without it. A block that carries this still wants `break-inside-avoid` for the
 * move to the next column; this is only what stops the column being too short to take it.
 */
export const UNBROKEN = 'data-unbroken';
