import { Mark } from '@app/ui/kit/Mark';
import {
	CentredPage,
	PAGE_EDGE,
	PAGE_HOST,
	PAGE_TOP,
	PageColumn,
	PageSpread,
} from '@app/ui/kit/PageSpread';
import type { PageHeading } from '@app/ui/kit/PageSpread';
import { HoldingProvider } from '@app/ui/kit/Plinth';
import { RailIcon, RailShell, SLUG_FLOOR } from '@app/ui/kit/RailShell';
import type {
	DetourStatus,
	ErrandCell,
	HeldStatus,
	RailEntry,
	RailReading,
} from '@app/ui/kit/RailShell';
import { RuledScroll } from '@app/ui/kit/RuledScroll';
import type { CSSProperties, MouseEvent, ReactNode, Ref, RefObject } from 'react';
import { useDeferredValue, useEffect, useLayoutEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import { FOCUS } from '@app/ui/kit/focus';
import { focusLost, useKeyboardKept } from '@app/ui/kit/keyboard';
import type { NestedPage } from '@app/ui/kit/nested';
import { SCREEN_H } from '@app/ui/kit/screen';
import { ViewingProvider } from '@app/ui/kit/viewing';

/**
 * **The frame.** The ground on the left, one destination on the right, and the rail that switches
 * between them standing on the split.
 *
 * **The ground is the app's home view, and the logo is its name.** On a wide screen the logo
 * collapses and restores it, and a destination's own cell closes it the same way — one cell, one
 * half, and the mark on it says whether that half is drawn. Below `lg` there is only ever one half,
 * so the ground becomes the landing page, every destination is a whole page over it, and the slug's
 * arrow is the fixed way back: not a history control, because there is only one place to go. The
 * ground is deliberately **not** a row in the raised register — it is not a destination, and a
 * phone must not be the one place that says it is.
 *
 * **The rail has three positions and they are the ground's, not the content's.** Which destination
 * is open never moves it. Collapsed, the ground is gone and the rail is at the left edge; with
 * nothing picked, the content is gone and the rail is at the right edge; with both open it stands
 * at the ground's own width.
 *
 * **Both columns report their scroll onto the rail.** The ledger's thumb on the rail's left edge,
 * the content's on its right. Neither column draws a bar at the outside of the glass, so the
 * divider in the middle carries both readouts and the outer edges stay clean. The two tracks
 * start at the same height because neither column has a head above its scroller.
 *
 * **Neither column is titled, and the page still names itself.** There is no *column* head:
 * chrome above the scroller, printing the destination the rail has already marked. Nothing is
 * drawn above either scroller. A *page* head is a different thing, in the way a document's title is
 * not the window's — it is the first line of the flow and it scrolls out of sight with it. See
 * `kit/PageSpread`. The ground has none, because the logo above it is still its name.
 *
 * **Width the page does not spend is a control with no reason to press it.** Collapsing the
 * ground hands the destination 1392px; set to a 32rem measure, five sixths of that would be
 * margin, and the state the control arrives at would be the same page with more air around it.
 * The width is spent on another column — but never as a mode: the page is drawn the same way at
 * every width and how many columns come of it is what the arithmetic in `PageSpread` returns.
 *
 * **This shell is controlled.** It takes the state and the handlers rather than owning them, so a
 * caller can seed a frame at any position. `useSplit` is the settled behaviour that goes with it —
 * which cell does what, and the rule that the screen always keeps one half — and a caller that
 * wants the frame to behave the way the design says it behaves uses it rather than reimplementing
 * it.
 */

/**
 * A destination: a rail cell that also says, in one line, what lives behind it.
 *
 * `fact` is the one true thing it reports in the raised register and the slug; `blurb` is what it
 * is for, and is all an undrawn destination has to show. `group` is which run of the rail it
 * belongs to — a change of group draws a hairline between two cells and nothing else does.
 */
export type SplitDest = RailEntry & { blurb: string; group: string };

/**
 * The state the whole frame is in, at any width.
 *
 * `ledger` is the wide screen's question and `menu` is the narrow screen's; `dest` is the only one
 * both widths answer, which is why one seed can draw both.
 */
export type SplitWhere = { ledger: boolean; dest: string | null; menu: boolean };

/** `overflow-hidden` so a column's clip-margin bleed cannot become document overflow. */
const SCREEN = `relative flex ${SCREEN_H} min-h-0 overflow-hidden bg-ui-bg font-ui-sans text-ui-ink [color-scheme:var(--ui-scheme)]`;
const CELL = `relative flex h-11 w-11 shrink-0 cursor-pointer items-center justify-center ${FOCUS}`;

/**
 * **The frame moves in three beats, and no two of them overlap.**
 *
 * The first two: opening a destination slides the split across to an empty column, lands, and
 * only then does the page wipe in from the rail — 320ms, then 160ms.
 * Page-to-page, where the split has nowhere to go, is the wipe alone.
 *
 * The third has the same grammar. Collapsing the ground is a boundary move like any other, so
 * the page does not re-set itself while the rail is still travelling; the rail lands, the
 * one-column page stops being what is drawn, and the two-column setting wipes in over it from the
 * rail. 240ms, then 160ms. That is the whole of what "a page suddenly re-written into two columns"
 * means here: the wipe is the press coming down, and what is under it for those 160ms is the
 * setting being replaced.
 *
 * Expanding the ground is not the mirror of this. The page is being narrowed, two columns have
 * nowhere to go at 864px, and a column is laid out at the width of the state it is *entering* — so
 * the two-column setting is dropped at the start of the move, with no wipe. A wipe announces an
 * arrival; nothing is arriving when a page gives width back.
 *
 * Only the wide frame moves. Below `lg` there is no split, one half at a time, and never room for
 * a second column — so the slug frame cuts as it always has.
 */

/**
 * Slow-in slow-out — the pacing Heer & Robertson measured as best for keeping hold of one object
 * across a change. Used wherever a thing travels between two places it belongs.
 *
 * Symmetric means the second control point mirrors the first: `x2 = 1 - x1`. `(0.45, 0, 0.15, 1)`
 * reads as an ease-in-out and is not one — pinned at its own midpoint it has already covered 79% of
 * the distance, so the middle of the move is a dash between two dawdles.
 */
const TRACK = 'cubic-bezier(0.45, 0, 0.55, 1)';

/** Decelerating. For something arriving, which is a different event from something moving. */
const DECELERATE = 'cubic-bezier(0.05, 0.7, 0.1, 1)';

/** How long the body takes to arrive, once the boundary that summoned it has landed. */
const WIPE_MS = 160;

/** The two measured widths the split is built from. The rail is the third column, always 3rem. */
const RAIL_W = '3rem';

/**
 * The ledger column, as the sum it is: the page's gutter, the sheet's own measure, and the same
 * `2rem` the page on the other side of the rail spends on its left edge.
 *
 * A flat width leaves the sheet's last column short of the rail — a flat `33rem` left it **76px**
 * short — while the page on the other side starts **32px** from it: the two halves of one screen
 * keeping two different margins off the thing standing between them. As a sum, the margin is the
 * same number on both sides and the sheet has the width that would go spare.
 *
 * **The sheet's measure is `34rem`**, and the same rule is why it is one number here rather than
 * two: the column *is* the sum, so widening the sheet is widening the column. The cost is the
 * page's second column, which needs **1796px** of glass beside it; at 1440 and 1920 the column
 * count is what it would be at `29rem`.
 */
const LEDGER_EDGE = '2rem';
const SHEET_W = '34rem';
const LEDGER_W = `calc(var(--spacing-ui-gutter) + ${SHEET_W} + ${LEDGER_EDGE})`;

/** Which half of the screen a cell opens. The logo owns the ledger; a destination owns the content. */
type Side = 'left' | 'right';

/**
 * The mark: a 20×6 rule with a head on it, at the foot of the cell, pointing at the half the cell
 * opens.
 *
 * Every marked cell draws it at the same end, so the arrowhead — six pixels of triangle — carries
 * the whole distinction between the logo and a destination. Hard corners, 2px shaft, the same ink
 * and the same width as the calendar's picked-day rule it is derived from.
 *
 * The rule that comes with it: **an arrow names a direction on screen, never a next step.**
 */
function ArrowMark({ side }: { side: Side }) {
	return (
		<svg
			aria-hidden
			viewBox="0 0 20 6"
			className="absolute inset-x-0 bottom-0 mx-auto h-1.5 w-5 fill-ui-ink"
		>
			{side === 'left' ? (
				<>
					<rect x={6} y={2} width={14} height={2} />
					<polygon points="0,3 6,0 6,6" />
				</>
			) : (
				<>
					<rect x={0} y={2} width={14} height={2} />
					<polygon points="20,3 14,0 14,6" />
				</>
			)}
		</svg>
	);
}

/**
 * The floor the motion answers to, and it answers the simplest way: nothing moves.
 *
 * Nothing in this frame is carried by motion — every state is already readable from the mark, the
 * widths and `aria-current` — so honouring the preference costs the design nothing and is not a
 * lesser version of it.
 */
function useStill(): boolean {
	const query = '(prefers-reduced-motion: reduce)';
	const [still, setStill] = useState(() => window.matchMedia(query).matches);

	useEffect(() => {
		const media = window.matchMedia(query);
		const onChange = () => {
			setStill(media.matches);
		};

		media.addEventListener('change', onChange);
		return () => {
			media.removeEventListener('change', onChange);
		};
	}, []);

	return still;
}

/** From here up the wide frame is the one drawn: Tailwind's `lg`, the width every shell switches on. */
const WIDE_SCREEN = '(min-width: 64rem)';

/** Whether the wide frame is the one on screen, followed as the window crosses `lg`. */
function useWideScreen(): boolean {
	const [wide, setWide] = useState(() => window.matchMedia(WIDE_SCREEN).matches);

	useEffect(() => {
		const media = window.matchMedia(WIDE_SCREEN);
		const onChange = () => {
			setWide(media.matches);
		};

		media.addEventListener('change', onChange);
		return () => {
			media.removeEventListener('change', onChange);
		};
	}, []);

	return wide;
}

/**
 * **The page is mounted once, whichever setting draws it.**
 *
 * The frame draws the open page in up to three places: the wide screen's one-column setting, its
 * two-column setting (both at once for the length of a collapse, `useReset`), and the phone's shell,
 * which CSS hides from `lg`. Given the page as three sets of children, React mounts it two or three
 * times over — every page's state is held twice at rest, and the logo, which is free because it
 * loses nothing, remounts the page and loses all of it: a draft's changes, a run back to its start,
 * and a second stream open for every copy.
 *
 * So the page is rendered once, into one element (`host`), and the settings hold empty slots. The
 * slot of the setting on screen takes the element; a move between settings moves it, and React,
 * which renders into it through a portal, never sees the move. Which slot is on screen is read off
 * the same `lg` the CSS switches on — the drawing is still the CSS's, and only where the one live
 * page sits follows it.
 *
 * **The setting a collapse is taking away keeps a still copy.** The wipe needs the old one-column
 * page standing under its edge while the two-column one arrives (`useReset`), so the element the
 * slot gives up is copied into it first — inert, for the 400ms the old setting is on screen.
 *
 * **A move keeps every animation where it was.** A browser restarts an element's CSS animations
 * when it is taken out of the document and put back, so a chip mid-pulse would jump to its first
 * frame on every collapse. Their clocks are read before the move — during render, because a setting
 * React takes away takes the page's element out with it before any effect runs — and put back on
 * the page and on its copy after it.
 */
function usePageHost({
	slots,
	wideScreen,
	wide,
	leaving,
}: {
	slots: {
		one: RefObject<HTMLDivElement | null>;
		two: RefObject<HTMLDivElement | null>;
		phone: RefObject<HTMLDivElement | null>;
	};
	wideScreen: boolean;
	/** The two-column setting is the one drawn. */
	wide: boolean;
	/** The one-column setting is still on screen, being wiped away. */
	leaving: boolean;
}): HTMLDivElement {
	const [host] = useState(() => {
		const element = document.createElement('div');
		element.setAttribute(PAGE_HOST, '');
		element.style.display = 'contents';
		return element;
	});

	const slot = wideScreen ? (wide ? slots.two : slots.one) : slots.phone;

	/** Where the page last stood, and its animations' clocks read on the render that moves it. */
	const placed = useRef(slot);
	const clocks = useRef<Clocks | null>(null);
	// eslint-disable-next-line react-hooks/refs -- Reason: the clocks have to be read before the commit that moves the page stops its animations, so on the render, not in an effect
	if (slot !== placed.current && host.isConnected) clocks.current = clocksOf(host);

	useLayoutEffect(() => {
		const target = slot.current;
		if (target === null || host.parentElement === target) return;

		const from = host.parentElement;
		const active = document.activeElement;
		const focused = active instanceof HTMLElement && host.contains(active) ? active : null;
		const kept = clocks.current;
		clocks.current = null;
		placed.current = slot;

		if (from !== null && from === slots.one.current && wide && leaving) {
			const copy = host.cloneNode(true) as HTMLElement;
			copy.setAttribute('inert', '');
			copy.setAttribute('aria-hidden', 'true');
			from.replaceChildren(copy);
			if (kept !== null) setClocks(copy, kept, twinsOf(copy, host));
		}
		target.replaceChildren(host);
		if (kept !== null) setClocks(host, kept, element => element);
		focused?.focus({ preventScroll: true });
	});

	return host;
}

/** One CSS animation's clock: where it stands on the timeline, and whether it is held still. */
type Clock = { start: CSSNumberish | null; current: CSSNumberish | null; paused: boolean };

/** Each element's CSS animations' clocks, by pseudo-element and animation name. */
type Clocks = Map<Element, Map<string, Clock>>;

/** A CSS animation's element and name — what it is known by across a move. */
function cssAnimationOf(animation: Animation): { target: Element; key: string } | null {
	const name = (animation as Partial<CSSAnimation>).animationName;
	const effect = animation.effect as Partial<KeyframeEffect> | null;
	const target = effect?.target;
	if (typeof name !== 'string' || target === null || target === undefined) return null;
	return { target, key: `${effect?.pseudoElement ?? ''}|${name}` };
}

function clocksOf(root: Element): Clocks {
	const clocks: Clocks = new Map();
	if (typeof root.getAnimations !== 'function') return clocks;
	for (const animation of root.getAnimations({ subtree: true })) {
		const known = cssAnimationOf(animation);
		if (known === null) continue;
		const own = clocks.get(known.target) ?? new Map<string, Clock>();
		own.set(known.key, {
			start: animation.startTime,
			current: animation.currentTime,
			paused: animation.playState === 'paused',
		});
		clocks.set(known.target, own);
	}
	return clocks;
}

/**
 * Put each CSS animation under `root` back on the clock it had before the move. `origin` names the
 * element whose clock an element takes: itself, or for a copy, the element it was copied from.
 */
function setClocks(
	root: Element,
	clocks: Clocks,
	origin: (element: Element) => Element | undefined
) {
	if (clocks.size === 0 || typeof root.getAnimations !== 'function') return;
	for (const animation of root.getAnimations({ subtree: true })) {
		const known = cssAnimationOf(animation);
		const from = known === null ? undefined : origin(known.target);
		const clock =
			known === null || from === undefined ? undefined : clocks.get(from)?.get(known.key);
		if (clock === undefined) continue;
		if (clock.paused) {
			animation.pause();
			animation.currentTime = clock.current;
		} else if (animation.playState === 'running' && clock.start !== null) {
			animation.startTime = clock.start;
		}
	}
}

/** For each element of a copy, the element it was copied from. */
function twinsOf(copy: Element, original: Element): (element: Element) => Element | undefined {
	const copies = copy.querySelectorAll('*');
	const originals = original.querySelectorAll('*');
	const twins = new Map<Element, Element>();
	copies.forEach((element, index) => {
		const twin = originals[index];
		if (twin !== undefined) twins.set(element, twin);
	});
	return element => twins.get(element);
}

/** Where a setting draws the page: an empty box the page's one element is moved into. */
function PageSlot({ slot }: { slot: RefObject<HTMLDivElement | null> }) {
	return <div ref={slot} className="contents" {...{ [PAGE_HOST]: '' }} />;
}

/**
 * The second beat: the wipe the content arrives under.
 *
 * A one-shot entrance is an animation, not a transition. Written as a transition it has to be set
 * closed and then opened, and those are two style changes one tick apart — Chrome treats the
 * second as a reversal, shortens it by however far the first had already got, and the wipe starts
 * from wherever it happened to be. Measured: a 160ms rule arriving as 133ms and starting 83% open.
 * `element.animate()` has none of that, plays once, and is still a real `Animation`, so the
 * harness can pin it like anything else.
 *
 * The guard is "did this key change", not "is this the first run": StrictMode mounts twice, so a
 * once-flag is spent on the discarded mount and the real one performs anyway.
 *
 * **Whether the boundary also moved is read from the key, not passed in.** Arriving from `null`
 * means the content column was shut and is opening, which is the one change that slides the split;
 * going from one page to another leaves the split exactly where it was. So the hook knows, from
 * the two ends of its own key, which of the two cases it is in, and holds itself back by `holdMs`
 * in the first — that hold is the whole of what makes this two beats rather than one blur.
 * `fill: 'backwards'` is what keeps the column shut through the delay; without it the page is
 * drawn for the length of the boundary's move and then wiped in over itself.
 */
function useWipe(
	key: string | null,
	{ still, easing, holdMs }: { still: boolean; easing: string; holdMs: number }
) {
	const ref = useRef<HTMLDivElement | null>(null);
	const seen = useRef<string | null | undefined>(undefined);

	useLayoutEffect(() => {
		const was = seen.current;
		seen.current = key;

		if (was === undefined || was === key || key === null || still) return;

		ref.current?.animate([{ clipPath: 'inset(0 100% 0 0)' }, { clipPath: 'inset(0 0 0 0)' }], {
			duration: WIPE_MS,
			easing,
			delay: was === null ? holdMs : 0,
			fill: 'backwards',
		});
	}, [key, still, easing, holdMs]);

	return ref;
}

/**
 * The third beat: the page re-set in two columns, once the boundary that widened it has landed.
 *
 * Two layers rather than one, and that is the whole trick. The one-column page has to stay on
 * screen for the length of the boundary's move — it is what the rail is uncovering — so the
 * two-column setting cannot simply replace it at the start. It is drawn over the top, held shut by
 * `fill: 'backwards'` for exactly `holdMs`, and then wiped in from the rail.
 *
 * **The two clips are complements, and they have to be.** The new setting opens from the left and
 * the old one closes to the left on the same clock, so the wipe edge has the new page on one side
 * of it and the old page on the other, and never both. Wiping the new setting in *over* the old
 * one looks right until the moment it lands: the two flows agree down the left column as far as
 * the first column break and diverge after it, so the surplus of the old setting sits under the
 * new one the whole way and then vanishes in a single frame when the layer is dropped. Erasing it
 * under the edge is what makes this a press coming down rather than one page fading into another.
 *
 * The spent layer is dropped when the wipe finishes, not on a timer: `finished` is the animation's
 * own clock, so a frame pinned at `?t=` still has the old setting standing beside the wipe edge,
 * which is the frame that shows what the beat actually does.
 *
 * `setTimeout` would have been the obvious way to hold the swap back and it is the wrong one — in
 * a capture tab timers are clamped to a second while the animation timeline is merely frozen, so
 * the two would disagree about what 240ms means.
 *
 * Going the other way is a cut. `wide` falling drops the second column at once, before the
 * boundary starts, because the page is being narrowed into a width two columns cannot hold and a
 * column is laid out at the width of the state it is entering.
 */
function useReset(wide: boolean, { still, holdMs }: { still: boolean; holdMs: number }) {
	const set = useRef<HTMLDivElement | null>(null);
	const spent = useRef<HTMLDivElement | null>(null);
	const seen = useRef(wide);

	/** Whether the one-column layer has been let go. Seeded wide means it never existed. */
	const [oneGone, setOneGone] = useState(wide);

	useLayoutEffect(() => {
		const was = seen.current;
		seen.current = wide;

		if (was === wide) return;

		if (!wide) {
			// eslint-disable-next-line react-hooks/set-state-in-effect -- Reason: the change in `wide` is only known against the ref the effect keeps, and the layer has to come back before paint
			setOneGone(false);
			return;
		}

		if (still) {
			setOneGone(true);
			return;
		}

		const timing = { duration: WIPE_MS, easing: DECELERATE, delay: holdMs };

		/**
		 * Closing to the left as the other opens from it: at the edge, one page or the other.
		 *
		 * **`fill: 'both'`, and the `both` is the whole of it.** A fill of `backwards` holds the
		 * first keyframe through the delay and then hands the element back to its own style the
		 * instant the wipe finishes — which for this layer means no clip at all, so the setting that
		 * had just been wiped away reappeared whole and stayed until React got round to unmounting
		 * it. That is the flicker: not a stray frame but a page snapping back on and off again, a
		 * state update wide. The layer that is being erased has to stay erased.
		 */
		spent.current?.animate([{ clipPath: 'inset(0 0 0 0)' }, { clipPath: 'inset(0 0 0 100%)' }], {
			...timing,
			fill: 'both',
		});

		/** `backwards` is right here: what this one falls back to when it ends is the arrived page. */
		const play = set.current?.animate(
			[{ clipPath: 'inset(0 100% 0 0)' }, { clipPath: 'inset(0 0 0 0)' }],
			{ ...timing, fill: 'backwards' }
		);

		if (play === undefined) {
			setOneGone(true);
			return;
		}

		let live = true;

		void play.finished.then(
			() => {
				if (live) setOneGone(true);
			},
			() => {
				/* cancelled: the state it was arriving at is gone, and so is the layer's job */
			}
		);

		return () => {
			live = false;
		};
	}, [wide, still, holdMs]);

	return { set, spent, oneGone };
}

/**
 * The ground, open. No head — the sheet is the first thing in it — but the same page margin
 * above and below it as the page on the other side of the rail, which is what makes the two read
 * as two halves of one screen rather than as two columns that happen to be adjacent.
 */
/**
 * The ground's column.
 *
 * **The 8px above the sheet is part of the document, not chrome over it.** Anything sticky in a
 * column is held inside that column's scroller, and the box it is held against is the scroller's
 * *content* box — so 8px of padding above the scroller, or on the scroller, is 8px an object pinned
 * to `top: 0` can never reach: it arrives with a strip of ground over it and reads as hanging rather
 * than as held against the glass. As the first margin of the document it is the same 8px at rest and
 * the first thing that scrolls away, which is what it was for.
 */
function Ledger({ className = '', children }: { className?: string; children: ReactNode }) {
	return (
		<div className={`flex min-h-0 flex-col pl-ui-gutter ${className}`}>
			<RuledScroll rule="none" edge="right" className="flex-1" scrollerClassName="pr-8">
				<div className={`mt-2 ${PAGE_EDGE}`}>{children}</div>
			</RuledScroll>
		</div>
	);
}

/** The open destination, in the column the rail uncovered. */
function Content({
	head,
	flow,
	centred = false,
	className = 'min-w-0 flex-1',
	nested = null,
	children,
}: {
	head: PageHeading;
	flow: string;
	/** Set in the middle of the column with no head drawn. See `CentredPage`. */
	centred?: boolean;
	className?: string;
	/** The page open is a nested one: the spread sets it one step in. */
	nested?: NestedPage | null;
	children: ReactNode;
}) {
	if (centred)
		return (
			<CentredPage head={head} className={className}>
				{children}
			</CentredPage>
		);
	return (
		<div className={`flex min-h-0 flex-col pt-2 ${className}`}>
			<PageSpread head={head} flow={flow} edge="left" className="flex-1" nested={nested}>
				{children}
			</PageSpread>
		</div>
	);
}

/** The logo, doing a job: the ground's switch, with a destination's body and a destination's mark. */
function LogoSwitch({
	open,
	home,
	onGo,
	ref,
}: {
	open: boolean;
	/** What the ground is called — the switch's name says which half it collapses. */
	home: string;
	onGo: () => void;
	ref?: Ref<HTMLButtonElement>;
}) {
	return (
		<button
			ref={ref}
			type="button"
			onClick={onGo}
			aria-label={open ? `Collapse ${home}` : `Expand ${home}`}
			aria-pressed={open}
			className={`${CELL} ${open ? 'text-ui-ink' : 'text-ui-ink-3 hover:text-ui-ink-2'}`}
		>
			{open ? <ArrowMark side="left" /> : null}
			<Mark />
		</button>
	);
}

function RailCell({ dest, active, onGo }: { dest: SplitDest; active: boolean; onGo: () => void }) {
	return (
		<button
			type="button"
			onClick={onGo}
			title={dest.label}
			aria-label={dest.label}
			aria-current={active ? 'page' : undefined}
			className={`${CELL} ${
				active ? 'text-ui-ink' : 'text-ui-ink-3 hover:bg-ui-row hover:text-ui-ink-2'
			}`}
		>
			{active ? <ArrowMark side="right" /> : null}
			<RailIcon icon={dest.icon} />
		</button>
	);
}

/** `open` is the ground's state; `active` is the open destination, or none. */
function Rail({
	destinations,
	open,
	home,
	active,
	logo,
	onLogo,
	onGo,
}: {
	destinations: SplitDest[];
	open: boolean;
	home: string;
	active: string | null;
	/** The logo's own button, which holds the keyboard on the ground. */
	logo?: Ref<HTMLButtonElement>;
	onLogo: () => void;
	onGo: (id: string) => void;
}) {
	return (
		<nav
			aria-label="Destinations"
			className="relative flex w-12 shrink-0 flex-col items-center border-x border-ui-rule pt-2"
		>
			<LogoSwitch ref={logo} open={open} home={home} onGo={onLogo} />
			<span aria-hidden className="my-1 h-px w-6 bg-ui-rule" />

			<div className="flex w-full flex-col items-center">
				{destinations.map((dest, index) => (
					<div key={dest.id} className="contents">
						{index > 0 && dest.group !== destinations[index - 1]?.group ? (
							<span aria-hidden className="my-1 h-px w-6 bg-ui-rule" />
						) : null}
						<RailCell
							dest={dest}
							active={dest.id === active}
							onGo={() => {
								onGo(dest.id);
							}}
						/>
					</div>
				))}
			</div>
		</nav>
	);
}

/**
 * **A frame whose `where` lives somewhere else** — the app's, which keeps the open destination in
 * the address so a link, a reload and the browser's back all agree with the rail.
 *
 * Given, `useSplit` holds no state of its own: it reads `where` and reports each move through
 * `onWhere`, and the caller hands the next `where` back. Omitted, the hook holds it, as the
 * workbench does. The moves themselves — which press opens which half, the screen always keeping
 * one — are the same in both, because there is one copy of them.
 */
export type SplitControl = { where: SplitWhere; onWhere: (next: SplitWhere) => void };

/**
 * The settled behaviour of the frame's cells, so that a caller does not have to restate it.
 *
 * Two rules, and both of them come out of one idea — the screen always keeps at least one half:
 *
 * - **Collapsing the ground with nothing picked opens the destination at rest**, rather than
 *   leaving a rail alone on the glass. Which one is at rest is the caller's to name.
 * - **Picking the destination that is already open closes it**, which is the same move the logo
 *   makes on the other half. Closing the content restores the ground for the same reason.
 */
export function useSplit(
	seed: SplitWhere,
	{ resting, control }: { resting: string; control?: SplitControl }
) {
	const [own, setOwn] = useState<SplitWhere>(seed);
	const where = control?.where ?? own;

	/**
	 * Every move goes through here. Held, it is state; controlled, it is reported and the caller's
	 * `where` comes back as the next render's — the same move either way.
	 */
	function setWhere(move: (was: SplitWhere) => SplitWhere) {
		if (control === undefined) setOwn(move);
		else control.onWhere(move(control.where));
	}

	function toggleLedger() {
		setWhere(was =>
			was.ledger
				? { ledger: false, dest: was.dest ?? resting, menu: false }
				: { ...was, ledger: true, menu: false }
		);
	}

	function go(id: string) {
		setWhere(was =>
			was.dest === id
				? { ledger: true, dest: null, menu: false }
				: { ...was, dest: id, menu: false }
		);
	}

	/** The slug's body does one thing wherever it is tapped: raise or lower the register. */
	function slug() {
		setWhere(was => ({ ...was, menu: !was.menu }));
	}

	/**
	 * The arrow at the slug's left, which is the only other thing down there.
	 *
	 * It is not a history control. Every destination is one page over the ground, so leaving one
	 * is a fixed move to the ground rather than a step back through wherever you had been.
	 */
	function home() {
		setWhere(() => ({ ledger: true, dest: null, menu: false }));
	}

	return { where, go, toggleLedger, slug, home };
}

/** Where a guarded press was heading, when it is the ground rather than a destination. */
export const GROUND_ID = 'ground';

/**
 * A guarded press that goes nowhere: *stop the run*, *discard the changes* — any cell that loses
 * the work where it stands. The confirm is raised exactly
 * as for leaving; answering it ends the hold and moves nothing — except from a peek, where the
 * ground is what is on screen and so is where the hand stays.
 */
export const HERE_ID = 'here';

/**
 * Where a guarded press was heading, when it is an address the rail has no cell for — another
 * record, pressed on the sheet. What it is called travels with it (`awayName`).
 */
export const AWAY_ID = 'away';

/** A frame that is holding something, seeded: where it is, what it holds, and what it is mid-way through. */
export type HeldSeed<T> = SplitWhere & {
	held: T | null;
	/** Below `lg`: the ground is on screen although the page is still open and still held. */
	peeking?: boolean;
	/** A guarded press already made and not yet answered. `GROUND_ID`, `HERE_ID`, or a destination's id. */
	going?: string | null;
};

/** What `useHeld` hands back: the frame's whole state, and every press it answers. */
export type HeldHand<T> = {
	where: SplitWhere;
	held: T | null;
	peeking: boolean;
	going: string | null;
	go: (id: string) => void;
	home: () => void;
	toggleLedger: () => void;
	slug: () => void;
	peek: () => void;
	stay: () => void;
	discard: () => void;
	release: () => void;
	/** Ask before ending the hold where it stands (`HERE_ID`) — a run's `Stop`. */
	stop: () => void;
	/**
	 * **Leave for an address the rail has no cell for**: straight through with nothing held;
	 * while something is, the confirm asks first (`AWAY_ID`) and `then` runs only on its discard.
	 */
	away: (name: string, then: () => void) => void;
	/** What the `away` press waiting on the confirm was heading for. `null` without one. */
	awayName: string | null;
};

/**
 * **The frame's behaviour when a page is holding something you would lose.**
 *
 * It wraps `useSplit` rather than replacing it: which cell opens which half, and the rule that the
 * screen always keeps one, are settled and unchanged. What is added is the one question the frame
 * had never had to answer — whether a press is allowed to happen yet.
 *
 * **Free, and it matters that they are free:** the logo, and the peek. Collapsing the ledger is the
 * same errand at a different amount of glass, and the peek is the whole reason the ground is worth
 * looking at while you write. Neither can lose anything, so neither asks.
 *
 * **Guarded, all of them:** every destination press, including the one that closes the destination
 * you are on, including a row of the raised register, and including the arrow that goes home. They
 * are one thing — leaving the page — and the frame has four ways to say it.
 *
 * A guarded press **lowers the register on its way to the confirm**. A menu standing behind a modal
 * is two overlays about two different questions, and the register's job is over the moment the row
 * was pressed.
 *
 * The record itself is the caller's — the hook never reads a field of it. A hold is a draft here
 * and a long run next, and a hook that knew which would have to be widened for the other.
 */
export function useHeld<T>(
	seed: HeldSeed<T>,
	{ resting, control }: { resting: string; control?: SplitControl }
): HeldHand<T> {
	const split = useSplit(seed, { resting, control });

	const [held, setHeld] = useState<T | null>(seed.held);
	const [peeking, setPeeking] = useState(seed.peeking ?? false);
	const [going, setGoing] = useState<string | null>(seed.going ?? null);
	const [bound, setBound] = useState<{ name: string; then: () => void } | null>(null);

	/**
	 * **A live hold follows its record.** A draft is declared once and ends on a press; a run
	 * starts on a press, ticks every second and ends on its own, so the seed is followed whenever it
	 * changes. A caller with a hold that never changes passes the same object every render.
	 */
	const [seen, setSeen] = useState<T | null>(seed.held);
	if (seed.held !== seen) {
		setSeen(seed.held);
		setHeld(seed.held);
		if (seed.held === null) {
			setPeeking(false);
			setGoing(null);
			setBound(null);
		}
	}

	/** Put the register away, wherever a press has made it irrelevant. `slug` is a toggle. */
	function lower() {
		if (split.where.menu) split.slug();
	}

	function ask(target: string) {
		lower();
		setGoing(target);
	}

	return {
		where: split.where,
		held,
		peeking,
		going,

		go(id: string) {
			if (held === null) {
				split.go(id);
				return;
			}
			ask(id);
		},

		home() {
			if (held === null) {
				split.home();
				return;
			}
			ask(GROUND_ID);
		},

		toggleLedger: split.toggleLedger,
		slug: split.slug,

		peek() {
			lower();
			setPeeking(was => !was);
		},

		away(name: string, then: () => void) {
			if (held === null) {
				then();
				return;
			}
			lower();
			setBound({ name, then });
			setGoing(AWAY_ID);
		},

		awayName: going === AWAY_ID ? (bound?.name ?? null) : null,

		/** Answer the confirm: nothing moves. */
		stay() {
			setGoing(null);
			setBound(null);
		},

		/**
		 * A capability cell that keeps the work was pressed — a save: the hold ends where it stands,
		 * and nothing moves.
		 *
		 * **A cell that loses the work does not come here.** It goes through `stop` and the confirm,
		 * because the rule is one line: a cell that loses work asks, a cell that keeps it never does.
		 */
		release() {
			setHeld(null);
			setPeeking(false);
			setGoing(null);
			setBound(null);
		},

		stop() {
			lower();
			setGoing(HERE_ID);
		},

		/** Answer the confirm: the hold is gone and the frame arrives where the press said. */
		discard() {
			const target = going;
			const then = bound?.then;
			setHeld(null);
			setPeeking(false);
			setGoing(null);
			setBound(null);

			if (target === null) return;
			if (target === AWAY_ID) {
				then?.();
				return;
			}
			if (target === HERE_ID) {
				if (peeking) split.home();
				return;
			}
			if (target === GROUND_ID) split.home();
			else split.go(target);
		},
	};
}

/** `value` as it stood at the last commit — this render's own value until it has committed once. */
function useCommitted<T>(value: T): T {
	const committed = useRef(value);
	useEffect(() => {
		committed.current = value;
	});
	// eslint-disable-next-line react-hooks/refs -- Reason: the last commit's value is the point, and it is only written by an effect
	return committed.current;
}

type DrawnPage = {
	head: PageHeading;
	body: ReactNode;
	flow: string;
	centred: boolean;
	nested: NestedPage | null;
};

/**
 * `page`, or the last page drawn once there is none.
 *
 * Written during render, and idempotent for it: StrictMode's second pass writes the same value
 * the first did, because what it writes comes only from this render's own props.
 */
function useLastDrawn(page: DrawnPage | null): DrawnPage | null {
	const last = useRef<DrawnPage | null>(null);
	// eslint-disable-next-line react-hooks/refs -- Reason: the page has to stay drawn on the render that takes it away, which an effect is one render late for
	if (page !== null) last.current = page;
	// eslint-disable-next-line react-hooks/refs -- Reason: read on the render that takes the page away, as above
	return page ?? last.current;
}

export function SplitShell({
	destinations,
	where,
	reading,
	home,
	head = null,
	flow = '',
	centred = false,
	ledger,
	overlay,
	held = null,
	detour = null,
	nested = null,
	errand = null,
	away = false,
	onGo,
	onLink,
	onLogo,
	onSlug,
	onHome,
	children,
}: {
	destinations: SplitDest[];
	where: SplitWhere;
	/**
	 * The frame is put away under a screen that stands in its place. The caller hides it; the
	 * shell lands the keyboard when it comes back.
	 */
	away?: boolean;
	/** What the slug prints below `lg`. Every screen has to be able to say where it is. */
	reading: RailReading;
	/** What the ground is called — the slug's back arrow names it, and so does the register. */
	home: string;
	/** What the open page calls itself. `null` when nothing is open, which is the ground. */
	head?: PageHeading | null;
	/** A token that changes whenever the page's contents do. See `PageSpread`. */
	flow?: string;
	/** The open page is set centred with no head drawn, not as a document. See `CentredPage`. */
	centred?: boolean;
	/** The ground's contents. The shell gives it the scroll, the rule and the page margin. */
	ledger: ReactNode;
	/** A dialog over the whole frame. */
	overlay?: ReactNode;
	/**
	 * What the open destination is holding, and would lose. `null` is the resting frame.
	 *
	 * Three things follow from it and none of them is a second prop. The slug draws the hold's row
	 * *instead of* the resting bar, on either half of the frame, and the register cannot be raised
	 * until the hold is answered. Below `lg` a **peek** is the one position the split's rule cannot
	 * express, so `peeking` is what puts the ground on screen with the page still open. And the
	 * ledger is drawn in view mode for as long as anything is held: a page holding a draft is the
	 * errand you are in the middle of, and a sheet beside it that could still be worked would be a
	 * second errand running on the same record with no guard of its own.
	 */
	held?: HeldStatus | null;
	/**
	 * Below `lg`: the page has sent the phone to the ground for one errand and is waiting. The
	 * ground is on screen with the page still open, as in a peek, and the slug is the detour's row.
	 */
	detour?: DetourStatus | null;
	/**
	 * **The page open is a nested one** — a record the page opened in its own place. From `lg`
	 * the spread sets it one step in with the way back pinned at the top of the step; below `lg` the
	 * slug grows a level row with the way back in it. It arrives under the page's own wipe. `null`,
	 * the frame draws exactly what it drew before.
	 */
	nested?: NestedPage | null;
	/**
	 * **The page's own errand, as a cell on the slug's resting bar** — below `lg` only; from
	 * `lg` the page draws it in its flow. `null`, the bar is what it always was.
	 */
	errand?: ErrandCell | null;
	onGo: (id: string) => void;
	/**
	 * **A plain press on a link anywhere in the frame**, before the browser follows it. Return
	 * `true` to take it off the browser — the caller has made it a press of its own, which is how a
	 * link on a page that is holding something goes through the same confirm as a rail cell.
	 *
	 * Opt-in. Omitted, every link is the browser's and the frame behaves exactly as it did. A press
	 * that opens a new tab or window leaves nothing behind, so it is never offered.
	 */
	onLink?: (href: string) => boolean;
	onLogo: () => void;
	onSlug: () => void;
	onHome: () => void;
	/** The open page's body, under its head. */
	children?: ReactNode;
}) {
	const dest = destinations.find(entry => entry.id === where.dest) ?? null;
	const showLedger = where.ledger || dest === null;

	/**
	 * The ground, told whether it is being read or worked — one expression, drawn at both widths.
	 *
	 * It wraps the ledger rather than the whole frame so that the rule stays a fact about *this
	 * column*: what is held is held by the page, and a sheet drawn inside that page would have no
	 * reason to go quiet.
	 */
	const viewing = held === null ? false : held.reads === 'record' ? 'record' : true;

	/**
	 * **The sheet follows the frame; it never holds the frame up.**
	 *
	 * A page arriving or leaving changes what the sheet draws — a record marked, worked or read —
	 * and every one of those re-draws every record on it: measured on a full sheet, 190–300ms in dev
	 * and 60–120ms built. Rendered with the press, that is all spent before the first frame of the
	 * move, so the rail and the wipe start late and the split lurches when they do. Deferred, the
	 * press paints the frame at once and the sheet catches up in a render the browser can interrupt, a
	 * frame or two later.
	 */
	const shownLedger = useDeferredValue(ledger);
	const shownViewing = useDeferredValue(viewing);

	const ground = <ViewingProvider value={shownViewing}>{shownLedger}</ViewingProvider>;

	/**
	 * **The sheet is drawn once, in the shell on screen.** Both shells are always drawn and CSS
	 * picks, but a sheet is a thousand nodes and the other shell's copy was the same again,
	 * re-rendered on every change for a sheet nobody could see. Crossing `lg` draws it afresh in the
	 * other shell, as the other copy always was.
	 */
	const wideScreen = useWideScreen();

	/**
	 * **The same fact, given to the page: the chrome is carrying this errand's controls.**
	 *
	 * A place declares `held` once. The slug builds its cells from it, and a foot inside the page
	 * reads it to know what not to draw a second time — which is what makes *one control, one
	 * surface* a single declaration rather than two conditions that happen to agree. It wraps
	 * the whole frame rather than the page alone because the hold is a fact about the frame: the
	 * guard is on every way out at both widths, and only the *giving up* is below `lg`, which
	 * `kit/Plinth` spends a media query on.
	 */
	const holding = held !== null;

	/** The page's half of the wide screen, whose head takes the keyboard when a hold is answered. */
	const pageHalf = useRef<HTMLDivElement | null>(null);

	/**
	 * **A hold answered from the page's own foot hands the keyboard to the page's head.** The
	 * foot's controls go with the hold, which would leave the keyboard on `<body>`. The head is
	 * where the page, as it now stands, begins. A frame later, so a confirm
	 * that asked has let go of it first, and only if nothing else has taken it and the wide screen is
	 * the one drawn: below `lg` the slug answers holds, and `kit/RailShell` puts it on the bar. Of two
	 * copies of the page mid-move, the last is the one that stays.
	 */
	const pageHead = (): HTMLElement | null => {
		const half = pageHalf.current;
		if (half === null || half.getClientRects().length === 0) return null;
		return Array.from(half.querySelectorAll<HTMLElement>('h1')).at(-1) ?? null;
	};

	/** The rail's logo, which holds the keyboard on the ground: the wide ground has no head. */
	const logo = useRef<HTMLButtonElement>(null);

	/**
	 * Any press that takes its own control off the page lands the keyboard on the head. On the
	 * ground there is no page, and the column that closed still holds the last one's head out of
	 * sight, so the rail's logo takes it there.
	 */
	useKeyboardKept(() => {
		if (dest !== null) return pageHead();
		return logo.current !== null && logo.current.getClientRects().length > 0 ? logo.current : null;
	}, away);

	const wasHolding = useRef(holding);
	useEffect(() => {
		const was = wasHolding.current;
		wasHolding.current = holding;
		if (!was || holding) return;
		const frame = requestAnimationFrame(() => {
			if (focusLost()) pageHead()?.focus();
		});
		return () => {
			cancelAnimationFrame(frame);
		};
	}, [holding]);

	/** No destination open: the ground is the whole left side and there is no right side. */
	const bare = dest === null;

	const wasBare = useCommitted(bare);

	/**
	 * The boundary's two steps, picked rather than computed. Collapsing the ground moves the
	 * split 528px; opening or closing the content column moves it 1144px or more, and the same
	 * duration over twice the distance is a different speed, not a consistent one.
	 */
	const splitMs = bare || wasBare ? 320 : 240;

	/**
	 * The last page drawn, so the column that is closing still has something in it. A column that
	 * empties itself and then shrinks is two events; this is one.
	 */
	const drawn = useLastDrawn(
		head === null ? null : { head, body: children, flow, centred, nested }
	);

	const still = useStill();

	/** A nested page arrives under the page's own wipe: it is one more page in the column. */
	const wipeKey =
		where.dest === null || nested === null ? where.dest : `${where.dest}/${nested.id}`;
	const wipeRef = useWipe(wipeKey, { still, easing: DECELERATE, holdMs: splitMs });

	/**
	 * The page has the whole glass. `showLedger` is already false only when a destination is open
	 * — the screen always keeps one half — so this is the collapsed state and nothing else.
	 */
	const wide = !showLedger;
	const { set: resetSet, spent: resetSpent, oneGone } = useReset(wide, { still, holdMs: splitMs });

	const oneSlot = useRef<HTMLDivElement | null>(null);
	const twoSlot = useRef<HTMLDivElement | null>(null);
	const phoneSlot = useRef<HTMLDivElement | null>(null);
	const host = usePageHost({
		slots: { one: oneSlot, two: twoSlot, phone: phoneSlot },
		wideScreen,
		wide,
		leaving: !oneGone,
	});

	/**
	 * **A column is laid out at a width it is shown at, and clipped to the rest.**
	 *
	 * Each column animates between three numbers — the ledger at its own width, the whole glass,
	 * gone; the content gone, the remainder, the whole glass — and **gone is not a layout**. A
	 * column on its way out therefore keeps the width it was last shown at, and the clip box shrinks
	 * out from under it. The layout width is still always one the column is *shown* at, so the
	 * frame the move lands on is correct. What is ruled out is the other thing — drawing the content
	 * at its collapsed-ground width in the resting both-open state, which would leave 528px of every
	 * page and its whole right gutter off-screen while nothing is moving at all. Clipping is for the
	 * length of a transition, never for a state, and neither of these outlives its move.
	 *
	 * **This is what stops a column crushing as it leaves.** Laid out at its own box's width the
	 * ledger is re-set at 400px, then 200, then 40 — every frame a real layout with its lines
	 * rewrapped and its rows stacked on top of one another, a sheet visibly collapsing in on itself
	 * for the length of the move, and the scroller's content height changing under the scroll
	 * position on each of them. Held at its own width it slides out under the rail instead, and the
	 * scroll position cannot move because nothing reflows.
	 *
	 * The ledger still reflows between its own width and the whole glass, because those are two real
	 * settings of the sheet and it rests in both. That was measured before it was allowed: forcing
	 * it through 21 widths from 528 to 1672 with a synchronous full-document layout each time costs
	 * **0.06–0.11ms per relayout**, against a 16.7ms frame.
	 *
	 * **`clip` rather than `hidden`, with a bleed.** Both thumbs straddle the rule on purpose — an
	 * 11px span at `-right-[5px]`, so half the 2px bar is outside its own column — and a plain
	 * `overflow: hidden` shaves that half off for exactly the length of the move. `overflow: clip`
	 * with `overflow-clip-margin` cuts the page off and lets the thumb through. A column that has
	 * gone to nothing is clipped hard instead: at zero width the scroller still overflows
	 * vertically, so its thumb would otherwise be left standing on the rail reporting a column that
	 * is not there.
	 *
	 * **Each column is pinned to the rail's side of its own clip box**, which is why both are
	 * `absolute` rather than merely `h-full`. A column laid out normally starts at the far edge and
	 * is eaten from the rail's side, and the scroll thumb — which is drawn at the column's
	 * rail-facing edge — therefore stays where the rail *was*. Measured at 160ms of the 240ms
	 * collapse: the rail had reached x=117 and the thumb was still at x=522, 405px out over the
	 * other half. Both columns report their scroll onto the rail, so nothing they draw on it may
	 * come off it for the length of a move.
	 *
	 * The animated width sits on the clip box rather than on `Ledger` itself for a second reason as
	 * well: `Ledger` carries the page's left gutter, and under `box-sizing: border-box` a box
	 * cannot shrink below its own padding — animating it directly leaves a 20px stub of the ground
	 * behind, with the rail standing on it and a live thumb beside it.
	 */
	const full = `calc(100vw - ${RAIL_W})`;

	/** The content column's width while the ground is open — its other real setting. */
	const half = `calc(100vw - ${LEDGER_W} - ${RAIL_W})`;
	const contentWidth = showLedger ? half : full;

	/** What the ledger is laid out at: one of its two real settings, never zero. */
	const ledgerWidth = bare ? full : LEDGER_W;

	/** What each clip box is drawn at. Zero is a box, not a layout. */
	const splitWidths = {
		ledger: showLedger ? ledgerWidth : '0px',
		content: bare ? '0px' : contentWidth,
	};

	/**
	 * **The width the spent layer is laid out at.**
	 *
	 * A column on its way out keeps the width it last rested at, and this is that rule one level
	 * down, for a reason of its own. How many columns a page has is
	 * not written on the page — it is what `PageSpread` reads off the box it is in. Laid out at the
	 * arrived width, the leaving layer would measure the *new* width, come back with the *new*
	 * column count, and the wipe would be a two-column page being replaced by the same two-column
	 * page. Held at `half` it is still the one-column setting the collapse is taking away, which is
	 * the only thing there is to see under that edge.
	 *
	 * **Named rather than remembered.** A previous width kept in a ref written during render is not
	 * idempotent the way `last` above is: StrictMode's second render reads back the value the first
	 * has just overwritten, and the spent layer measures 1672px instead of 1144. It does not need
	 * remembering: `wide` means the ground is collapsed with a destination open, and the only state
	 * that reaches it is the one where both halves were drawn.
	 */
	const spentWidth = wide ? half : contentWidth;

	/** A column that still has a width lets its thumb through; one that has none does not. */
	const clipped = (shown: boolean): CSSProperties =>
		shown ? { overflow: 'clip', overflowClipMargin: '8px' } : { overflow: 'hidden' };

	/**
	 * The frame's one listener for links, on its root: a link is read wherever it is drawn, and the
	 * page is laid out by `PageSpread`, which watches its own children — a box added round the page to
	 * catch presses would be one more child that never resizes.
	 */
	const follow =
		onLink === undefined
			? undefined
			: (event: MouseEvent<HTMLDivElement>) => {
					if (
						event.button !== 0 ||
						event.metaKey ||
						event.ctrlKey ||
						event.shiftKey ||
						event.altKey
					)
						return;
					const link = event.target instanceof Element ? event.target.closest('a[href]') : null;
					const href = link?.getAttribute('href') ?? null;
					if (href !== null && onLink(href)) event.preventDefault();
				};

	const slide: CSSProperties = {
		transition: still ? 'none' : `width ${String(splitMs)}ms ${TRACK}`,
	};

	return (
		/**
		 * One positioned root over both branches, so an overlay is drawn once and lands the same
		 * way at either width — `kit/Modal` is `absolute inset-0`, and two overlays in two shells
		 * would be two dialogs whenever a media query changed its mind.
		 */
		<HoldingProvider value={holding}>
			<div className="relative" onClickCapture={follow}>
				<div className={`${SCREEN} hidden lg:flex`}>
					<div
						className="relative shrink-0"
						style={{ width: splitWidths.ledger, ...clipped(showLedger), ...slide }}
					>
						<div className="absolute inset-y-0 right-0" style={{ width: ledgerWidth, ...slide }}>
							<Ledger className="h-full w-full">{wideScreen ? ground : null}</Ledger>
						</div>
					</div>

					<Rail
						destinations={destinations}
						open={showLedger}
						home={home}
						active={dest?.id ?? null}
						logo={logo}
						onLogo={onLogo}
						onGo={onGo}
					/>

					<div
						ref={pageHalf}
						className="relative shrink-0"
						style={{ width: splitWidths.content, ...clipped(!bare), ...slide }}
					>
						{drawn === null ? null : (
							<div
								ref={wipeRef}
								className="absolute inset-y-0 left-0"
								style={{ width: contentWidth }}
							>
								{oneGone ? null : (
									<div
										ref={resetSpent}
										className="absolute inset-y-0 left-0 flex"
										style={{ width: spentWidth }}
									>
										<Content
											head={drawn.head}
											flow={drawn.flow}
											centred={drawn.centred}
											nested={drawn.nested}
											className="min-h-0 min-w-0 flex-1"
										>
											<PageSlot slot={oneSlot} />
										</Content>
									</div>
								)}

								{wide ? (
									<div ref={resetSet} className="absolute inset-0 flex">
										<Content
											head={drawn.head}
											flow={drawn.flow}
											centred={drawn.centred}
											nested={drawn.nested}
											className="min-h-0 min-w-0 flex-1"
										>
											<PageSlot slot={twoSlot} />
										</Content>
									</div>
								) : null}
							</div>
						)}
					</div>
				</div>

				<div className="lg:hidden">
					<RailShell
						sections={destinations}
						section={dest?.id ?? ''}
						panels={[]}
						panel=""
						menu={where.menu}
						phone={
							held?.peeking === true || detour !== null ? 'list' : dest === null ? 'list' : 'panel'
						}
						reading={reading}
						home={home}
						held={held}
						detour={detour}
						nested={nested}
						errand={errand}
						away={away}
						onGo={onGo}
						onSlug={onSlug}
						onHome={onHome}
						column={
							<div className={`${PAGE_TOP} ${SLUG_FLOOR} px-ui-gutter`}>
								{wideScreen ? null : ground}
							</div>
						}
					>
						{head === null ? null : centred ? (
							<CentredPage head={head}>
								<PageSlot slot={phoneSlot} />
							</CentredPage>
						) : (
							<PageColumn head={head}>
								<PageSlot slot={phoneSlot} />
							</PageColumn>
						)}
					</RailShell>
				</div>

				{drawn === null ? null : createPortal(drawn.body, host)}

				{overlay}
			</div>
		</HoldingProvider>
	);
}
