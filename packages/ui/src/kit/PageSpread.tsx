import { IconButton } from '@app/ui/kit/Button';
import { RailIcon, SLUG_FLOOR } from '@app/ui/kit/RailShell';
import { RuledScroll } from '@app/ui/kit/RuledScroll';
import type { CSSProperties, ReactNode } from 'react';
import { useEffect, useLayoutEffect, useRef } from 'react';
import { FOCUS } from '@app/ui/kit/focus';
import type { NestedPage } from '@app/ui/kit/nested';
import {
	EDGE,
	GAP,
	PAGE,
	UNBROKEN,
	blockWidth,
	columnsFor,
	fitsTheGlass,
	searchBounds,
} from '@app/ui/kit/spread';

/**
 * **A page, set as a spread.**
 *
 * The content column of the frame is a document, and a document on a wide screen is a
 * *spread*: one flow, in as many columns of the measure as the glass will take, with one
 * margin spent on every edge of it. Three things are settled here and none of them is a
 * setting a caller passes:
 *
 * **A page says what it is.** Every page opens with its own name and one line of what it is
 * for, and that head is the **first block of the flow** rather than a band across the top of
 * the spread. It sits in the first column, it scrolls away with the opening line, and it is
 * set in the document's own type — which is what makes it part of the content rather than
 * more chrome, and why it is drawn identically however many columns the page turns out to
 * have. It is drawn before anything is known about the contents, so a page that is still
 * loading still says what it is.
 *
 * **A column is set at the measure.** Not at a share of the block: left to itself
 * `column-count` divides the width evenly and hands each column whatever is over, so two
 * columns at 1392px are 656px wide holding 512px of type, with the surplus cut into strips
 * and hidden *inside* the columns. The columns are set at `MEASURE` and the block capped at
 * exactly what they and their gutters come to, which leaves the surplus in one piece at the
 * right-hand edge, where it reads as the glass being wider than the page rather than as the
 * page being badly set.
 *
 * **Columns are a consequence of the text, not a setting of the page.** See `useSpread`.
 * Nothing here switches into two columns; the flow fills one column to the foot of the glass
 * and starts another only when there is something left over, so a page that never reaches
 * the foot is a one-column page with no rule drawn in it at all.
 *
 * **Where the flow breaks is the section's own answer, not the page's.** A policy applied to the
 * contents by tag name from outside, such as `[&_section]:break-inside-avoid`, makes every
 * `<section>` atomic because the page says so, and none can say otherwise. So `kit/PageSection` is
 * what a page is built of instead — the band, the rhythm below it, and a break policy the section
 * declares. The page knows how many columns there are and nothing at all about what is in them.
 *
 * Below `lg` none of that applies — there is never room for a second column — so the page is
 * `PageColumn`, which is the same head and the same margins in one column at the width of
 * the glass.
 */

/** What a page calls itself, and the one line under it. The errand, not the noun. */
export type PageHeading = {
	title: string;
	lede: string;
	/**
	 * The title is a record's name rather than an errand's, so it truncates like every other name
	 * instead of wrapping — a nested page's head. Undefined, it wraps.
	 */
	cut?: boolean;
};

/**
 * The measure, the margin and the arithmetic that falls out of them — `kit/spread`.
 *
 * `MEASURE` is not a new number: `views/page-shared` caps the document's body at 32rem and every
 * section inside it is drawn against that. The **column** is set at it too.
 *
 * `EDGE` is 32px because that is the gutter the scroller already carried. `GAP` is twice it,
 * so the rule — centred in its gap — stands exactly one edge from the column on either side.
 * Spent left of the first column, both sides of every rule, right of the last, and above and
 * below the lot of them, it is the single margin that makes the columns read as leaves of one
 * sheet rather than as panes. `PAGE` is what the head and the foot cost a column between them.
 *
 * The rules that are functions of those numbers — how many columns fit, what the block is set to,
 * and the two ends the height search is allowed to settle between — live in `kit/spread`, so that
 * they can be asserted without a browser. `kit/spread.test` is where *columns split only when there
 * is room* is written down as a check.
 */

/** Everything about the spread that is the same whatever `useSpread` measures. */
const SPREAD: CSSProperties = {
	paddingTop: EDGE,
	paddingBottom: EDGE,
	paddingLeft: EDGE,
	paddingRight: EDGE,
	columnGap: GAP,
	columnRule: '1px solid var(--color-ui-rule)',

	/**
	 * **The fill order.**
	 *
	 * The default is `balance`, which divides the flow evenly between the columns however short
	 * it is — so a page with four rows in it is cut down the middle and set as two two-row
	 * columns with a rule between them and an empty glass underneath. `auto` fills the first
	 * column to the foot and starts the second only when there is something left over, so the
	 * page works like one column until it would overflow. Chrome paints no rule past
	 * the content, so a page that never reaches the foot has no seam in it at all.
	 */
	columnFill: 'auto',
};

/**
 * **The same margin at the head and the foot, on both halves of the screen.**
 *
 * The spread already spent `EDGE` across: left of the first column, either side of every rule,
 * right of the last. It spends it twice more, above the first line and below the last, which is
 * what makes it the page's one margin rather than its one *horizontal* margin. Two things
 * follow from writing it as padding on the multicol block rather than as space around it:
 *
 * **The rule ends where the page does.** Chrome paints a column rule down the **content box**,
 * so the block's own `padding-bottom` is the one thing that stops the hairline short of the
 * glass — measured, 200px of padding moved the rule's foot up exactly 200px. A rule that runs
 * into the bottom edge reads as the page having been cut off mid-column rather than as having
 * ended.
 *
 * **The foot is only there when you have reached it.** Both margins are inside the scroller, so
 * they are the first and last things in the flow and neither is a fixed band: the head scrolls
 * away with the first line, and the foot is what you arrive at after the last.
 *
 * It has to be padding on a **block in the flow**, never on the scroller itself. Chrome leaves
 * a scroll container's own `padding-bottom` out of its scrollable overflow once the content is
 * taller than the box, so a foot written there is there until the moment you would see it and
 * then is not — measured, a scroller padded 32 top and bottom reported a `scrollHeight`
 * carrying the 32 above and nothing below. The head is safe either way; they are only
 * symmetrical in the source.
 *
 * **Whatever the page margin is, both halves of the screen spend it**, and a half that brings
 * its own has that one taken off rather than added to. `PAGE_EDGE` is therefore what the ledger
 * is given as well, so a ledger carries no vertical margin of its own.
 *
 * Below `lg` the head is the same and the foot is not, because down there the foot has the slug
 * standing in it: `PAGE_TOP` alone opens the page and `SLUG_FLOOR` closes it, which is the same
 * 32px with the bar and the gutter it floats in counted in underneath.
 */
export const PAGE_TOP = 'pt-8';

/**
 * The attribute on the boxes the frame draws the page through (`kit/SplitShell`'s one mount): its
 * slot, and the element the page is rendered into. Both are `display: contents`, so they set
 * nothing and are seen through — by the flow, and by `useSpread`, which measures the page's own
 * blocks inside them.
 */
export const PAGE_HOST = 'data-ui-page-host';

/** The flow's blocks: the element's children, read through the frame's page boxes. */
function blocksOf(element: Element): Element[] {
	return Array.from(element.children).flatMap(child =>
		child.hasAttribute(PAGE_HOST) ? blocksOf(child) : [child]
	);
}
export const PAGE_EDGE = `${PAGE_TOP} pb-8`;

/**
 * **How many columns a page has, and how tall they are**, answered as arithmetic rather than as a
 * mode.
 *
 * Three numbers come out of one measurement of the box the page is scrolling in:
 *
 * 1. **How many columns fit.** As many whole columns of the measure as the width will take, at
 *    least one. The block is then set to exactly that width, so every column lands on the
 *    measure and every gutter on `EDGE`; the surplus is left over on the right in one piece
 *    rather than handed to the columns, which is the difference between a spread and two panes.
 * 2. **How long the flow is.** Read at a single column of the measure, with the height released.
 *    `scrollHeight` counts the padding as well as the content, and the page's head and foot
 *    margin is padding, so `PAGE` comes off it: what is wanted here is the length of the type,
 *    not the length of the sheet it is on. Everything downstream is arithmetic on the column's
 *    *content* box and has the margin added back at the end. This is the only way to know how
 *    much there is to place before deciding where to break it, and it costs one layout that
 *    never reaches the screen, because it happens inside `useLayoutEffect`.
 * 3. **What height to give the fragmentainer.** `column-fill: auto` has to be told where the
 *    foot is; it cannot infer one. While the flow fits in the columns available, the foot is
 *    **the glass** — fill column one to the bottom of the screen, start the second with what is
 *    left, and a page shorter than the glass is a one-column page. Once it does not fit, every
 *    column grows together to `flow / columns` and the page scrolls, which is the only honest
 *    thing left: the alternative is columns marching off the right-hand side into a horizontal
 *    scroll nobody asked for.
 *
 * `flow / columns` is a **floor, not the answer.** A section that will not fit whole at the foot
 * of a column moves down entire, so the real packing is always a little worse than the
 * arithmetic — measured on a 3187px flow in two columns, the ideal 1594 left 304px still
 * spilling. So the height is searched rather than computed.
 *
 * **The search is bounded by a height that cannot spill.** Growing the ideal by a fixed step and
 * giving up after a few is an unverified answer: if the last try still does not pack, that height
 * is written to the block anyway and the leftover goes sideways. The bound is arithmetic rather than
 * a guess — **the whole flow in one column** is a height every one of `columns` columns can hold,
 * because the flow measured `length` tall at exactly this measure, so it ends inside column one
 * and the rest stand empty. Bisecting between the ideal and that, `tall` is only ever moved onto a
 * height that has been *measured* not to spill, so the height finally written is one of those by
 * construction. Eight passes close a 3187px flow to within 0.4%; every pass is a forced layout,
 * and a page that fits the glass still costs two.
 *
 * 4. **When the contents change, all of it is wrong again.** `column-count: n` with an explicit
 *    height is not a cap: content past `n × height` is laid into **overflow columns to the
 *    right**, outside the block's own width, where the scroller clips them. So a height measured
 *    for yesterday's flow does not merely leave the page short — it prints a column the page has
 *    no room for, which is the one direction a document must never move in. Opening a disclosure
 *    on a page at 1512 once measured **256px** of exactly that.
 *
 *    The `flow` token cannot carry this. It says which page is open, and a page changes size
 *    without changing which page it is. So the contents are watched as well as the box: a
 *    `ResizeObserver` on the block's own children, and a `MutationObserver` for children arriving
 *    and leaving. **Children, never the block** — the block's height is this hook's own output and
 *    observing your own output is how you write a loop, whereas a child's box is the same at every
 *    column count, because a column is the measure whatever else changes.
 *
 * All of it runs inside one `useLayoutEffect`, so none of the intermediate heights is painted.
 *
 * Re-measured once more when the face has loaded: web fonts land after first layout and a flow
 * measured in the fallback is a flow measured wrong.
 *
 * **The block has to be the scroller's own child**, which is why `PageSpread` draws its
 * `RuledScroll` rather than expecting to be put inside one: `parentElement` is the track, and a
 * wrapper between the two would have the hook measuring a box that does not scroll.
 *
 * `inset` is width the flow is set in by on its left — a nested page's step. It is room the
 * columns cannot have, so it comes off the width before they are counted and goes back onto the
 * block after they are set.
 */
function useSpread(flow: string, inset = 0) {
	const box = useRef<HTMLDivElement | null>(null);

	useLayoutEffect(() => {
		const el = box.current;
		const track = el?.parentElement;
		if (!el || !track) return;

		const fit = () => {
			const glass = track.clientHeight;
			const room = track.clientWidth - inset;
			if (glass === 0 || room <= 0) return;

			const columns = columnsFor(room);

			/** One column of the measure, height released: the setting the flow is measured in. */
			const single = () => {
				el.style.width = `${String(blockWidth(1) + inset)}px`;
				el.style.columnCount = '1';
				el.style.height = 'auto';
			};

			single();

			/** The type on its own, with the page's head and foot margin taken back off it. */
			const length = el.scrollHeight - PAGE;

			/**
			 * The tallest block that asked never to be divided (`UNBROKEN`), measured here because
			 * here nothing is: one column, height released. 0 where the page has none.
			 */
			const unbroken = Math.max(
				0,
				...Array.from(
					el.querySelectorAll(`[${UNBROKEN}]`),
					block => block.getBoundingClientRect().height
				)
			);

			el.style.width = `${String(blockWidth(columns) + inset)}px`;
			el.style.columnCount = String(columns);

			/** Whether that height leaves the flow running on into a column that is not there. */
			const spills = (height: number) => {
				el.style.height = `${String(height)}px`;
				return track.scrollWidth - track.clientWidth >= 1;
			};

			/**
			 * The foot is the glass for as long as the flow fits in the columns there are.
			 *
			 * What a column can hold is the glass **less the page's margin**, which is the whole of
			 * what the head and foot cost: set against the glass itself this test passes on a page
			 * that is a margin too long, `spills` catches it, and the search runs for a page that
			 * should not have needed one.
			 */
			if (fitsTheGlass(length, columns, glass, unbroken) && !spills(glass)) return;

			/**
			 * The two ends of the search, and the invariant that makes it safe — see `kit/spread`.
			 *
			 * `tall` starts at `whole` and is only ever moved onto a height `spills` has just returned
			 * false for, so whatever it holds at the end is a height the page was measured to fit in.
			 */
			const { ideal, whole } = searchBounds(length, columns, glass, unbroken);

			if (!spills(ideal)) return;

			let short = ideal;
			let tall = whole;

			/**
			 * The one case the invariant cannot be proved into: content that refuses to pack at any
			 * height, which is a forced break or a box taller than the fragmentainer. A page is never
			 * allowed to answer that with a column standing off the side of the glass, so it stops
			 * being a spread — one column of the measure, height released, growing downwards.
			 */
			if (spills(tall)) {
				single();
				return;
			}

			for (let step = 0; step < 8 && tall - short > 8; step += 1) {
				const mid = Math.ceil((short + tall) / 2);
				if (mid >= tall) break;
				if (spills(mid)) short = mid;
				else tall = mid;
			}

			el.style.height = `${String(tall)}px`;
		};

		/**
		 * **What a fit is a function of**: the glass, and the size of every block in the flow. A block
		 * is set at the measure whatever the column count, so its box does not move when the fit does,
		 * and the fit's own output — the block's height — is not in it.
		 */
		const inputs = () =>
			[
				track.clientWidth,
				track.clientHeight,
				...blocksOf(el).map(block => (block as HTMLElement).offsetHeight),
			].join(' ');

		/**
		 * **Fit only when something it reads has changed.** Every observer reports once on the
		 * moment it starts watching, and a page arriving starts three — measured on a collapse, one
		 * fit and then two more of the same answer, 11ms of forced layout each, in the frames the rail
		 * is moving in.
		 */
		let fitted: string | null = null;
		const refit = () => {
			if (inputs() === fitted) return;
			fit();
			fitted = inputs();
		};

		/**
		 * The block's children, re-observed whenever the list of them changes.
		 *
		 * `disconnect` rather than unobserving one at a time: the list is a handful of sections, and
		 * a diff of it would be more code than the thing it saves.
		 */
		const contents = new ResizeObserver(refit);

		/**
		 * The page's blocks arrive inside the frame's page boxes (`PAGE_HOST`), and the page's one
		 * element moves between settings, so each box is watched for arrivals as well as the block.
		 */
		const arrivals = new MutationObserver(() => {
			watch();
			refit();
		});

		const watch = () => {
			contents.disconnect();
			for (const child of blocksOf(el)) contents.observe(child);
			arrivals.disconnect();
			arrivals.observe(el, { childList: true });
			for (const box of Array.from(el.querySelectorAll(`[${PAGE_HOST}]`)))
				arrivals.observe(box, { childList: true });
		};

		refit();
		watch();

		const glassBox = new ResizeObserver(refit);
		glassBox.observe(track);

		let live = true;
		void document.fonts.ready.then(() => {
			if (!live) return;
			fitted = null;
			refit();
		});

		return () => {
			live = false;
			glassBox.disconnect();
			contents.disconnect();
			arrivals.disconnect();
		};
	}, [flow, inset]);

	return box;
}

/**
 * The page's own first line: its name, and what it is for.
 *
 * **In the flow, not across it.** A head that spanned every column would be a band at the top
 * of the spread, which is a column head by another route — chrome over the page rather than the
 * page's own opening. This is the first block of the first column, it scrolls away like any
 * other, and it breaks nowhere. It is set at `ui-title` against the sections' `ui-lead`, so the
 * page and its sections are four points apart and in the same voice.
 *
 * **The title can hold the keyboard, and is never a stop in it**: `tabIndex` -1, so the frame
 * has somewhere to put focus when a press takes the control that held it off the page — a hold's
 * answer from the page's own foot, which replaces the foot.
 */
export function PageHead({ head }: { head: PageHeading }) {
	return (
		<header className="break-inside-avoid pb-7">
			<h1
				tabIndex={-1}
				className={`text-ui-title font-medium text-ui-ink ${FOCUS} ${head.cut === true ? 'truncate' : ''}`}
			>
				{head.title}
			</h1>
			<p className="pt-1 text-ui-body text-ui-ink-2">{head.lede}</p>
		</header>
	);
}

/**
 * **The step a nested page is set in by**: a 32px marker and the 16px the text stands off it.
 */
const STEP = 48;

/**
 * **The thread ends on the flow's last line** — never in the foot margin, and never down an empty
 * glass under a page shorter than it, where a rule running on past the last line would be empty
 * space rather than structure.
 *
 * The step is the height of the flow's block, and the block is the glass whenever the flow is
 * shorter, so the end has to be read rather than inherited. The step stands inside the flow's own
 * block, so the thread's box is a grandchild of the step's and the step a child of the block: the
 * last line is the lowest edge among the block's other children, read wherever they change size and
 * whenever one arrives or leaves — the page's `hidden` copy measures nothing and is passed over. The
 * height is written on the thread directly, like `useSpread`'s own writes, and the thread stands
 * outside the flow, so nothing it measures moves when it is written.
 */
function useThreadEnd() {
	const thread = useRef<HTMLSpanElement | null>(null);

	useLayoutEffect(() => {
		const line = thread.current;
		const step = line?.parentElement?.parentElement;
		const flow = step?.parentElement;
		if (!line || !step || !flow) return;

		const read = () => {
			let end = Number.NEGATIVE_INFINITY;
			for (const child of Array.from(flow.children)) {
				if (child === step) continue;
				const box = child.getBoundingClientRect();
				if (box.height > 0) end = Math.max(end, box.bottom);
			}
			const top = line.getBoundingClientRect().top;
			line.style.height = `${String(Math.max(0, end - top))}px`;
		};

		const sizes = new ResizeObserver(read);
		const watch = () => {
			sizes.disconnect();
			sizes.observe(flow);
			for (const child of Array.from(flow.children)) if (child !== step) sizes.observe(child);
		};
		const arrivals = new MutationObserver(() => {
			watch();
			read();
		});

		read();
		watch();
		arrivals.observe(flow, { childList: true });

		let live = true;
		void document.fonts.ready.then(() => {
			if (live) read();
		});

		return () => {
			live = false;
			sizes.disconnect();
			arrivals.disconnect();
		};
	}, []);

	return thread;
}

/**
 * **A nested page's step, from `lg`**: the page's own glyph at the top of it, pinned, and the
 * thread under it.
 *
 * The whole flow is set one `STEP` in, head included, and this fills the step for the flow's whole
 * height. It opens on the page's one margin, so the button's resting place is the head's own top
 * line, and `top-8` — the same margin as an offset — holds it there while the page scrolls: the way
 * back never scrolls away, and the step is its sticky range, so it is held from the first line to
 * the last. It is `IconButton`, the quiet rung, named for where it goes. Its ground is `ui-bg` under
 * the button's own, so the pointer's `ui-row` still composites over an opaque square and the thread
 * never shows through it. The thread is one `ui-rule` hairline in the flow of the step, 8px under the
 * button's box and 16px in — the button's centre, taking the pixel right of it, because 1px cannot
 * centre on 32.
 *
 * **Opening the page puts the keyboard on it**, as a detour's row does: the name that
 * opened the nested page is on the page it replaced, which is no longer on screen, so without this
 * the keyboard is left on `<body>`. Below `lg` the step is not drawn and the call does nothing; the
 * slug's level row takes it there.
 */
function NestStep({ nested }: { nested: NestedPage }) {
	const thread = useThreadEnd();
	const back = useRef<HTMLButtonElement>(null);

	useEffect(() => {
		back.current?.focus({ preventScroll: true });
	}, [nested.id]);

	return (
		<div className={`flex h-full flex-col ${PAGE_TOP}`}>
			<div className="sticky top-8 z-10 self-start bg-ui-bg">
				<IconButton ref={back} label={`Back to ${nested.page}`} onClick={nested.onBack}>
					<RailIcon icon={nested.icon} />
				</IconButton>
			</div>
			<span ref={thread} aria-hidden className="mt-2 ml-4 block w-px shrink-0 bg-ui-rule" />
		</div>
	);
}

/**
 * The page above `lg`: one flow, in as many columns as it turns out to need.
 *
 * The scroller carries no padding of its own. Every horizontal measure on this page belongs to
 * the block `useSpread` sizes — a gutter written on the scroller as well would be an edge the
 * spread's arithmetic does not know about, and the first column would sit one gutter further in
 * than the rule stands from the second.
 *
 * **A nested page is set one step in**: the block keeps its margin and adds `STEP` to its
 * left edge, and `NestStep` stands in that room for the block's full height.
 */
export function PageSpread({
	head,
	flow,
	edge = 'left',
	className = 'flex-1',
	nested = null,
	children,
}: {
	head: PageHeading;
	/**
	 * Which page this is — a token that changes when the destination does.
	 *
	 * It re-measures the page synchronously, before the new one is painted, which is what keeps a
	 * page from arriving in the setting the last one was in. It is **not** how the spread learns
	 * that the contents changed: the block's own children are on a `ResizeObserver` for that,
	 * because a page changes size without changing which page it is, and a caller that has to
	 * remember to say so will forget. See `useSpread`.
	 */
	flow: string;
	/** Which side the scroll thumb rides — the rail's side, wherever the rail is. */
	edge?: 'left' | 'right';
	className?: string;
	/** The page open here is a nested one. `null`, the spread draws the page itself. */
	nested?: NestedPage | null;
	children: ReactNode;
}) {
	const inset = nested === null ? 0 : STEP;
	const box = useSpread(flow, inset);
	const style: CSSProperties =
		nested === null ? SPREAD : { ...SPREAD, position: 'relative', paddingLeft: EDGE + inset };

	return (
		<RuledScroll rule="none" edge={edge} className={className}>
			<div ref={box} style={style}>
				{nested === null ? null : (
					<div className="absolute inset-y-0" style={{ left: EDGE, width: STEP }}>
						<NestStep nested={nested} />
					</div>
				)}
				<PageHead head={head} />
				{children}
			</div>
		</RuledScroll>
	);
}

/**
 * The page below `lg`: the same head and the same opening margin, in one column.
 *
 * No spread arithmetic, because there is never room for a second column at this width — and no
 * foot margin either, because down there the foot has the slug standing in it. `SLUG_FLOOR` is
 * given once, here, by the thing that knows the slug is there.
 *
 * The gutter is pulled off the scroller and put back on the scrolling box, so the thumb rides
 * the edge of the glass while the type keeps its margin.
 *
 * **A nested page wears nothing here.** The slug says it, in its level row, and the floor it
 * leaves grows with that row through the same custom property.
 */
export function PageColumn({
	head,
	className = 'flex-1',
	children,
}: {
	head: PageHeading;
	className?: string;
	children: ReactNode;
}) {
	return (
		<RuledScroll
			rule="none"
			edge="right"
			className={`-mx-ui-gutter ${className}`}
			scrollerClassName="px-ui-gutter"
		>
			<div className={`${PAGE_TOP} ${SLUG_FLOOR}`}>
				<PageHead head={head} />
				{children}
			</div>
		</RuledScroll>
	);
}

/**
 * **A page set in the middle of its column, with no head drawn**: the one page that is not a
 * document. A long run in progress is a thing to watch rather than read, so it stands centred on
 * both axes and the head is kept for assistive technology only. When it outgrows the column it scrolls
 * from its top, never clipped at both ends. Below `lg` it keeps clear of the slug.
 */
export function CentredPage({
	head,
	className = 'flex-1',
	children,
}: {
	head: PageHeading;
	className?: string;
	children: ReactNode;
}) {
	return (
		<div className={`flex min-h-0 flex-col overflow-y-auto ${className}`}>
			<h1 tabIndex={-1} className="sr-only">
				{head.title}
			</h1>
			<div className={`m-auto w-full px-ui-gutter py-8 lg:px-8 ${SLUG_FLOOR}`}>{children}</div>
		</div>
	);
}
