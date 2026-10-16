import { MOTION } from '@app/ui/kit/Button';
import { FootRoomProvider, useFootRoom } from '@app/ui/kit/FootRoom';
import { Mark } from '@app/ui/kit/Mark';
import { RuledScroll } from '@app/ui/kit/RuledScroll';
import type { LucideIcon } from 'lucide-react';
import { ArrowLeft, ChevronLeft, Eye } from 'lucide-react';
import { useEffect, useRef } from 'react';
import type { CSSProperties, ReactNode, Ref } from 'react';
import { FOCUS } from '@app/ui/kit/focus';
import { focusLost, pageHeadIn, useKeyboardKept } from '@app/ui/kit/keyboard';
import type { NestedPage } from '@app/ui/kit/nested';
import { SCREEN_H, TUCKED, useTyping, useTypingScroll } from '@app/ui/kit/screen';
import { CONNECTED_LIFT } from '@app/ui/kit/viewing';

/**
 * The rail frame: where you are in the left margin, what you are inspecting in the right,
 * and no band across the top at all.
 *
 * The mark takes the head of the left rail, which is the whole of what a header would carry
 * once the sections leave it — so the 44px a band would occupy goes to the content. Nothing
 * takes its foot: the app has one shared password and no person to name there.
 *
 * The two rails are not one rail because they answer different questions. The right rail
 * inspects **what is in the column**, and the split exists precisely so that column stays
 * visible while you work on it. The left rail goes **somewhere else**, and picking from it
 * replaces the whole middle. Folded together they would pin one section's column beside
 * another section's panel.
 *
 * Both columns scroll on their own page-break rule — see `RuledScroll` — so nothing is ever
 * drawn in the space between them, and the panel's bar is not a native one in a system that
 * draws every other line itself.
 *
 * **Below `lg` both rails become one object at the foot: the slug.** A margin has no room
 * to be a margin at that width, and bands for the rails to fold into would spend 80px of an
 * 844px screen on two rows of destinations that are inert most of the day. The slug says
 * where you are and one true thing about it, and opens upward into everything the rails
 * hold. There is nothing across the top at all, and a panel keeps no back control of its
 * own — the slug is the way out, because the slug is what the thumb is already on. A frame
 * that declares a `home` splits that foot into two targets on an inner page: an arrow that
 * leaves, and the rest of the bar, which raises the register the way it always does.
 */

/**
 * One cell in either rail.
 *
 * `fact` is the one true thing that cell can report — a count, a range, a state. It is
 * printed in the raised menu and nowhere else, so a list of destinations is a readout of
 * the section rather than words that do nothing until tapped. The rails proper have no
 * room for it and do not draw it.
 */
export type RailEntry = { id: string; label: string; icon: LucideIcon; fact?: string };

/**
 * What the slug prints: where you are, and exactly one thing about it.
 *
 * Never two. When work is running the work *takes the fact's slot over* rather than being
 * added beside it — a foot carrying the horizon and the filters and the count is clutter
 * with a readout's manners.
 */
export type RailReading = { place: string; fact: string };

/**
 * 44px of cell and 4px of inset, on both sides.
 *
 * The bar marking the active cell sits on the **page edge** in both rails — the left
 * rail's on its left, the right rail's on its right — so the mark always means the same
 * thing and the two columns read as two margins rather than as a frame. At the literal
 * edge of the glass a 2px mark is not seen, which is what the inset is for.
 */
const RAIL_LEFT = 'w-12 pl-1';
const RAIL_RIGHT = 'w-12 pr-1';

/**
 * What the header used to give the top of the page. With no band across the top the mark,
 * the column's first line and the panel's title would all start on the window's own edge.
 *
 * **It is given to each column separately, and to the scrolling one it is given inside the
 * scroller.** Anything sticky in a column is held against that scroller's content box, so 8px
 * spent above it is 8px an object pinned to the column's top edge can never reach: it arrives with
 * a strip of ground over it and reads as hanging rather than as held.
 */
const TOP = 'pt-2';

/**
 * The slug's own box.
 *
 * `ui-edge` and not `ui-rule`: an object floating over the sheet has to be told apart from
 * the sheet, and `ui-rule` is 8% white — 1.23:1 against the ground, and the very hairline
 * the sheet rules its own rows with. A control drawn in the ink of the thing it floats
 * over is camouflage. `ui-edge` is 35% and 3.21:1, which is what the token is for.
 */
const SLUG = 'w-full max-w-[20rem] border border-ui-edge bg-ui-bg';

/**
 * The room a column has to leave under its last row below `lg`, and the arithmetic of it.
 *
 * The slug floats over the column rather than displacing it, so without this the last row of a
 * sheet or a panel sits under the chrome — and a control the chrome sits on top of is a control
 * that cannot be reached. It belongs to the shell because the slug does, and it is given **once**:
 * a document that also leaves its own room ends a floor and a half below its last line.
 *
 * It is not a round number, because it is not a guess. `SLUG_INSET` under the bar, the bar's own
 * 42px — a 40px control in a 1px box — and one 32px page margin above it, which is the same margin
 * the head and foot of every column already spend. 20 + 42 + 32.
 *
 * **It is a variable for one case only.** A hold's row **replaces** the resting one rather than
 * adding to it, so the bar is 42px in every resting and held state. A nested page grows the box a
 * level row above the slug's own, and the floor grows by that row and its hairline — 135 in all.
 * The custom property is the channel — a column reads it, the shell writes it — and `SLUG_FLOOR`
 * carries its own fallback.
 *
 * **This is not the floor moving with the scroll**, which `kit/FootRoom` exists to avoid: that is
 * an object appearing and disappearing under the thumb, and
 * a document whose height changed as you scrolled it. A hold lasts as long as the errand does, and
 * a nested page as long as it is open.
 */
export const SLUG_FLOOR = 'max-lg:pb-[var(--ui-slug-floor,94px)]';

/** 20 under the bar, the bar's own 42, and one 32px page margin over it. */
const SLUG_SUM = 94;

/** A nested page's level row as the number the floor's sum is written in: `h-ui-touch`, and its hairline. */
const LEVEL_ROW = 41;

/** Whether an element is drawn at this width: the slug is `lg:hidden`, and a hidden one takes no focus. */
function drawn(element: HTMLElement | null): element is HTMLElement {
	return element !== null && element.getClientRects().length > 0;
}

/**
 * How far the slug floats off the glass, on each of the three edges it comes near.
 *
 * The gutter it is already inset by on the left and right. A bar held 16px off the foot and 20px
 * off the sides is not centred in anything; held off by its own gutter it reads as an object
 * floating in the page's margin, which is what it is.
 */
const SLUG_INSET = 'px-ui-gutter';

/**
 * The slug's own gutter off the foot, as a value rather than a class, because it is now the floor
 * of a sum: whatever a column has pinned to its foot, plus the gutter, keeps the bar off it.
 */
const SLUG_BOTTOM = 'var(--spacing-ui-gutter)';

/**
 * **Where the slug floats, and how it gets out of the keyboard's way.**
 *
 * Held `SLUG_BOTTOM` off the foot, plus whatever a column has pinned there (`kit/FootRoom`). While a
 * control that raises a keyboard has focus — on the sheet or on a page — the bar slides off the foot
 * by its own height and that offset, and fades, as the keyboard slides on: the foot is the
 * keyboard's then, and a field brought to the edge the slug floats over was under it. It comes back
 * when the field lets go, a hold's row and all.
 *
 * Its own component so that a focus change re-renders this box and nothing it holds.
 */
function SlugFloat({ room, children }: { room: number; children: ReactNode }) {
	const typing = useTyping();
	const lift = `${SLUG_BOTTOM} + ${String(room)}px`;
	return (
		<div
			style={{ bottom: `calc(${lift})`, translate: typing ? `0 calc(100% + ${lift})` : undefined }}
			className={`absolute inset-x-0 flex justify-center ${SLUG_INSET} transition-[bottom,translate,opacity,visibility] duration-[180ms,240ms,240ms,240ms] ease-out motion-reduce:transition-none ${typing ? TUCKED : ''}`}
		>
			{children}
		</div>
	);
}

/**
 * A destination's glyph.
 *
 * 18px at a 1.75 stroke: the system is drawn in hairlines, and lucide's default 2 at 24
 * puts a heavier line in the margin than anything else on the screen.
 */
export function RailIcon({ icon: Icon }: { icon: LucideIcon }) {
	return <Icon aria-hidden strokeWidth={1.75} className="h-[18px] w-[18px]" />;
}

function RailButton({
	entry,
	active,
	edge,
	onGo,
}: {
	entry: RailEntry;
	active: boolean;
	edge: 'left' | 'right';
	onGo?: (id: string) => void;
}) {
	return (
		<button
			type="button"
			onClick={() => onGo?.(entry.id)}
			title={entry.label}
			aria-label={entry.label}
			aria-current={active ? 'page' : undefined}
			className={`flex h-11 w-11 shrink-0 cursor-pointer items-center justify-center ${
				edge === 'left' ? 'border-l-2' : 'border-r-2'
			} ${FOCUS} ${
				active
					? 'border-ui-ink text-ui-ink'
					: 'border-transparent text-ui-ink-3 hover:bg-ui-row hover:text-ui-ink-2'
			}`}
		>
			<RailIcon icon={entry.icon} />
		</button>
	);
}

/**
 * **Where you are, in the slug's box**: the register's 2px ink bar on the row's left edge, or the
 * same 2px held transparent, so a row that is not marked still stands in the 44px label column.
 * Every row in the box takes its edge from here. It is one mapping, so the mark cannot mean two
 * things in one box.
 */
function rowMark(here: boolean): string {
	return here ? 'border-l-2 border-ui-ink' : 'border-l-2 border-transparent';
}

/**
 * A cell in one of the slug's rows: a glyph pressed beside the row's own body. It is `ui-ink-2` and
 * rises a rung under the pointer, and it lays no ground, because the row around it is the ground.
 * The box it sits in is the caller's.
 */
const SLUG_CELL = `cursor-pointer text-ui-ink-2 ${MOTION} hover:text-ui-ink ${FOCUS}`;

/**
 * A slug cell's box: 40px square, the glyph centred. Every cell in the box is this one — a hold's,
 * a detour's and a resting page's own — so they cannot drift apart.
 */
const CELL_BOX = `flex h-ui-touch w-ui-touch shrink-0 items-center justify-center ${SLUG_CELL}`;

/**
 * One line of the raised menu: the glyph, where it goes, and what is true there.
 *
 * The active cell takes the same 2px ink bar the rails use, on the edge nearest the page,
 * so the mark means the same thing whether it is drawn in a margin or in a list.
 */
function MenuRow({
	entry,
	active,
	onGo,
}: {
	entry: RailEntry;
	active: boolean;
	onGo?: (id: string) => void;
}) {
	return (
		<button
			type="button"
			onClick={() => onGo?.(entry.id)}
			aria-current={active ? 'page' : undefined}
			className={`flex h-ui-touch w-full shrink-0 cursor-pointer items-center gap-3 ${rowMark(active)} px-3 text-left ${FOCUS} ${
				active ? 'text-ui-ink' : 'text-ui-ink-2 hover:bg-ui-row'
			}`}
		>
			<RailIcon icon={entry.icon} />
			<span className="truncate text-ui-body">{entry.label}</span>
			<span className="ml-auto shrink-0 text-ui-body text-ui-ink-3">{entry.fact ?? ''}</span>
		</button>
	);
}

/**
 * The run's progress, drawn on the slug's own top edge.
 *
 * The same move the scroll makes on the page-break rule: a line the layout already owns
 * reports a measure instead of a second object being added to report it. Two pixels of ink
 * over the hairline, filling as the run goes.
 *
 * **It is drawn only while something is running.** A bar that is always there stops being
 * read; on a screen where nothing is moving it is decoration pretending to be an
 * instrument. Everything else the slug has to say is a count, and a count is a word.
 *
 * `aria-hidden` because the elapsed time is printed beside it — the mark is never the sole
 * carrier of anything.
 */
function SlugGauge({ share }: { share: number }) {
	return (
		<span
			aria-hidden
			style={{ width: `${share * 100}%` }}
			className="absolute -top-px left-0 h-0.5 bg-ui-ink"
		/>
	);
}

/* ------------------------------------------------------------------------- *
 * The status row
 * ------------------------------------------------------------------------- */

/**
 * One capability a hold offers a hand from the chrome: the glyph, what it is called, and what it
 * does.
 *
 * A draft carries **discard** and **save**, a long run carries **stop**, and every one of them
 * ends the hold where it stands. Whether it asks first is the caller's, and the rule the caller
 * keeps is one line: **a cell that loses work asks; a cell that keeps it never does.** The shell
 * only calls `onPress`.
 */
export type HeldCell = {
	id: string;
	icon: LucideIcon;
	label: string;
	onPress: () => void;
};

/**
 * **What the open destination is holding: something you would lose by leaving.**
 *
 * A draft and a long run are one shape — a page has something live, the ground is still worth
 * looking at, and leaving costs you. Nothing on this type names a kind of record: a `held` that
 * knew what one was would have to be widened for the next.
 *
 * **Given, it takes the slug over.** The row is not a band above the resting bar and not a row in
 * the register: it is what the box draws *instead of* the bar, for as long as the errand lasts.
 * Two consequences, and both are the point — the state you are in is on screen whichever half of
 * the frame you are looking at, and the register cannot be raised, so nothing leaves the page
 * until the hold is answered. The resting bar is never re-pointed or re-worded to report a hold,
 * and a held page's foot offers no way out.
 *
 * What the row **is**: the record's own bed, the 44px label column, one fact, the peek, a cell per
 * capability.
 *
 * Omitted, the slug draws its resting bar.
 */
export type HeldStatus = {
	/** What the held record is called. The shell prints it; it never looks it up. */
	name: string;
	/** The one fact about the hold, already in words — `3 unsaved`, `2m 14s`. */
	fact: string;
	/** How far through, 0–1, where the hold has a measure. Omitted, no gauge is drawn. */
	share?: number;
	/** Below `lg`: whether the ground is what is on screen rather than the page holding this. */
	peeking: boolean;
	/** Swap page ⇄ ground. Free, unguarded, and the whole of what the row's body does. */
	onPeek: () => void;
	/**
	 * What a hand can do about the hold from the chrome, in the order the cells sit — **on both
	 * faces of the row**. The slug carries every way to answer the hold whichever half is on
	 * screen, and the page gives those controls up below `lg`.
	 */
	cells: HeldCell[];
	/**
	 * **The page's own glyph** — the one its rail cell and register row wear. It is what the row's
	 * body draws while the ground is on screen, because that press goes back to the page. The
	 * shell has no default for it: a pencil standing in for a page would be one more glyph meaning
	 * *this destination*, and a caller that forgot would draw a page that is not there.
	 */
	icon: LucideIcon;
	/**
	 * **The hold's own run is going** — the record is being written, a long job is working.
	 *
	 * It is drawn in the vocabulary `kit/Button` already owns for exactly this and nothing is
	 * invented for the row: `BUSY` empties the plate to `ui-beat` and keeps the ink at the top of
	 * the ramp, so a running control is told from a live one by its **ground** and from a dead one
	 * by its **ink**. One level out, the row's ground is the record's bed, so the bed is what
	 * empties. The word is already in `fact` (`Saving`).
	 *
	 * Undefined by default. A run with a measure sets `share` as well and gets the gauge too.
	 */
	busy?: boolean;
	/**
	 * **How much of the ground the hold reads**: `'record'` reads only the record the page has open
	 * and leaves the rest of the sheet worked. Undefined, the whole ground is read.
	 */
	reads?: 'record';
};

/**
 * The row's ground at rest — the record's own bed — and while the record's run is going.
 *
 * `ui-beat` is `kit/Button`'s `BUSY` plate, spent here for the meaning it already has: the ground
 * empties and the ink stays at the top of the ramp, which is how this system says *working*.
 * Nothing is invented and no hue is added.
 *
 * **The lift is owed on both.** `CONNECTED_LIFT` re-sets `ui-ink-3` to `ui-ink-2`, and it is not the
 * bed's alone: `ui-ink-3` measures 3.36:1 on `ui-beat` against the 5.0 floor — `kit/Button`'s own
 * number for the ground it shares — and the fact slot is a quiet line on whichever ground is under
 * it.
 */
const HELD_BED = `bg-ui-connected ${CONNECTED_LIFT}`;
const HELD_BUSY = `bg-ui-beat ${CONNECTED_LIFT}`;

/**
 * **The hold's row: the record, wearing the record's own ground.**
 *
 * `ui-connected` already means one thing in this system and it is exactly this thing — the record
 * the page beside the ground has open, which the sheet beds on that ground — so the row that
 * reports that record is drawn on it. That ground is the whole of the difference:
 * the same 40px floor, the same 44px label column and the same glyph slot as `MenuRow`, nothing
 * re-worded, nothing taller, no hairline added.
 *
 * **The ink ramp comes with the ground.** `CONNECTED_LIFT` is the sheet's own override (`kit/viewing`),
 * imported rather than restated: `ui-ink-3` measures 3.70:1 on `#1e2f44` against the 5.0 floor, so every
 * quiet line on that ground owes a rung, and the fact slot is a quiet line.
 *
 * **The pointer composites instead of replacing.** `bg-ui-row` written *on* a banded element swaps
 * the hue out for grey at the moment somebody is asking about the banded thing. Here the ground is
 * the row and the hover is on the targets inside it, so 4% white lands over the bed rather than
 * instead of it.
 *
 * **Two targets, never one inside the other.** The row's own press is the peek, which is free, and
 * each capability is its own cell at the control floor — the same reason the slug beside it is two
 * targets in one box.
 *
 * **The glyph says where the press goes, and each glyph goes to one place only**: the eye to the
 * ground, the page's own glyph back to that page. The resting bar's arrow goes to the ground and to
 * nowhere else, which is why it is not here.
 *
 * **It is the slug's row, not a band over it** — see the box below. It keeps the 44px label column,
 * `border-l-2` included, so its name stands where every other row's name stands.
 *
 * **On a nested page the peek still peeks**, and under it the press goes back to the nested page
 * rather than the held one, so its name says so: `Back to Ada in Errands`. Where the hold *is* the
 * nested record (a note being changed inside Notes) the held name and the nested one are the same,
 * so the sentence names the page instead: `Back to Groceries in Notes`.
 */
function StatusRow({
	status,
	returnsTo,
	page,
}: {
	status: HeldStatus;
	returnsTo?: string;
	page?: string;
}) {
	const within = returnsTo === status.name ? (page ?? status.name) : status.name;
	const Glyph = status.peeking ? status.icon : Eye;

	return (
		<div
			aria-busy={status.busy === true || undefined}
			className={`relative flex h-ui-touch w-full items-center ${rowMark(false)} ${
				status.busy === true ? HELD_BUSY : HELD_BED
			}`}
		>
			{status.share === undefined ? null : <SlugGauge share={status.share} />}

			<button
				type="button"
				onClick={status.onPeek}
				aria-label={
					status.peeking
						? `Back to ${returnsTo === undefined ? status.name : `${returnsTo} in ${within}`}`
						: `Look at ${status.name}`
				}
				className={`flex h-full min-w-0 flex-1 cursor-pointer items-center gap-3 px-3 text-left ${MOTION} hover:bg-ui-row active:bg-ui-beat ${FOCUS}`}
			>
				<RailIcon icon={Glyph} />
				<span className="truncate text-ui-body text-ui-ink">{status.name}</span>
				<span className="ml-auto shrink-0 text-ui-body text-ui-ink-3">{status.fact}</span>
			</button>

			{status.cells.map(cell => (
				<button
					key={cell.id}
					type="button"
					onClick={cell.onPress}
					aria-label={cell.label}
					className={CELL_BOX}
				>
					<cell.icon aria-hidden className="h-4 w-4" />
				</button>
			))}
		</div>
	);
}

/** One thing a hand can do from a detour's row — `Select all`, `Clear all`. */
export type DetourCell = {
	id: string;
	icon: LucideIcon;
	label: string;
	onPress: () => void;
};

/**
 * **The one thing a page at rest offers from the slug** — a page's `Add a note`, say. Below `lg`
 * the resting bar on that page grows one cell after its fact, so the page's own errand is under the
 * thumb at every scroll position; from `lg` there is no slug and the page draws it in its flow. A
 * resting face carries at most one, and only on a page — never on the ground, a nested page, a hold
 * or a detour, whose rows have their own cells.
 */
export type ErrandCell = {
	label: string;
	icon: LucideIcon;
} & (
	| {
			/**
			 * Where it goes. The cell is a link, as the page's plate is, so the two surfaces of one
			 * declaration are one kind of element: it opens in a new tab like any link, and a plain
			 * press goes through the frame's own door.
			 */
			href: string;
	  }
	| {
			/**
			 * What it does, where the errand goes nowhere — Account's `Sign out`. The cell is a
			 * button, as the page's plate is then, so the two surfaces are still one kind of element.
			 */
			onPress: () => void;
	  }
);

/**
 * **A detour: the page sent the phone to the ground to do one thing there, and is waiting.**
 *
 * Not a hold: nothing is lost by going back, so there is no bed and no guard. The page stays open
 * underneath and the ground is what is on screen, the way a peek puts it there. The row replaces
 * the resting bar whole — the resting bar or a row of its own, never a mixture: the page's glyph
 * and name as the way back, one fact, and a cell per capability, such as selecting or clearing
 * everything the detour is picking.
 */
export type DetourStatus = {
	/** The page the row goes back to. */
	name: string;
	/** That page's own glyph — the one its rail cell wears. See `HeldStatus.icon`. */
	icon: LucideIcon;
	/** The one fact, already in words — `9 of 12 picked`. */
	fact: string;
	onBack: () => void;
	cells: DetourCell[];
};

/**
 * The detour's row: the hold row's shape without its bed — the page's glyph, its name and one fact
 * as a single press back to the page, then the cells.
 *
 * **Not an arrow.** The slug's one arrow goes to exactly one place from anywhere — the ground — and
 * an inner page's bar here (arrow, divider, name) would make it mean two places. So on the ground
 * the way back to a page is that page's own glyph, as it is in a peek. `border-l-2` is the 44px
 * label column every row in the box stands in.
 */
function DetourRow({ detour }: { detour: DetourStatus }) {
	const back = useRef<HTMLButtonElement>(null);

	/**
	 * Focus lands on the way back. The control that opened the detour is on a page that is no
	 * longer on screen, so without this the keyboard is left on `<body>`.
	 */
	useEffect(() => {
		back.current?.focus();
	}, []);

	return (
		<div className={`relative flex h-ui-touch w-full items-center ${rowMark(false)}`}>
			<button
				ref={back}
				type="button"
				onClick={detour.onBack}
				aria-label={`Back to ${detour.name}`}
				className={`flex h-full min-w-0 flex-1 cursor-pointer items-center gap-3 px-3 text-left ${MOTION} hover:bg-ui-row active:bg-ui-beat ${FOCUS}`}
			>
				<RailIcon icon={detour.icon} />
				<span className="truncate text-ui-body text-ui-ink">{detour.name}</span>
				<span className="ml-auto shrink-0 text-ui-body text-ui-ink-3">{detour.fact}</span>
			</button>
			{detour.cells.map(cell => (
				<button
					key={cell.id}
					type="button"
					onClick={cell.onPress}
					aria-label={cell.label}
					className={CELL_BOX}
				>
					<cell.icon aria-hidden className="h-4 w-4" />
				</button>
			))}
		</div>
	);
}

/**
 * **Where you are, one level in**: a nested page's row, standing on the slug's own.
 *
 * It stands in the 44px label column every row in the box stands in: the box's own edge, then a
 * glyph slot 42px wide, then the record's name. The row itself is not a press. On the nested page
 * the edge is the register's active mark and the glyph slot is the page's own glyph, one slug cell
 * back to the page, lifted over the row under it so the hold's bed never paints over its ring.
 * With the sheet on screen — a peek — the nested page is still open, so the row still names the
 * record the sheet is bedding, but the mark goes (no cell in the foot is marked active for a
 * column that is not on screen) and so does the glyph, because the hold's row wears that glyph
 * then, as the way back to the nested page, and one glyph goes to one place.
 *
 * Over the resting bar the name stands 9px left of the bar's own, which is set in behind its arrow
 * and divider; that offset is the bar's, and the raised register shows it too.
 *
 * **It is the one mark while the register is raised.** The register's row for the page it was
 * opened from goes unmarked then: pressing that row goes back to the page, which is what the glyph
 * here does, and two marks in one box would say you were in two places.
 */
function LevelRow({
	nested,
	here,
	back,
}: {
	nested: NestedPage;
	here: boolean;
	back: Ref<HTMLButtonElement>;
}) {
	return (
		<div className="border-b border-ui-rule">
			<div
				aria-current={here ? 'page' : undefined}
				className={`flex h-ui-touch items-center ${rowMark(here)}`}
			>
				{here ? (
					<button
						ref={back}
						type="button"
						onClick={nested.onBack}
						aria-label={`Back to ${nested.page}`}
						className={`relative flex h-full shrink-0 items-center px-3 focus-visible:z-10 ${SLUG_CELL}`}
					>
						<RailIcon icon={nested.icon} />
					</button>
				) : (
					<span aria-hidden className="invisible flex shrink-0 px-3">
						<RailIcon icon={nested.icon} />
					</span>
				)}
				<span className="truncate pr-3 text-ui-body text-ui-ink">{nested.name}</span>
			</div>
		</div>
	);
}

/**
 * The ground the raised menu floats over, and the way back out of it.
 *
 * It is a control, not a wash: the register is dismissed by putting the thumb anywhere that
 * is not the register, which is the gesture every raised thing on a phone answers to. Only a
 * frame that can actually close it gets a button — without `onClose` this stays inert
 * dimming, because a focusable element that did nothing would be worse than none.
 */
function Scrim({ onClose }: { onClose?: () => void }) {
	if (onClose === undefined) return <div aria-hidden className="absolute inset-0 bg-ui-scrim" />;

	return (
		<button
			type="button"
			onClick={onClose}
			aria-label="Close the menu"
			className="absolute inset-0 cursor-default bg-ui-scrim"
		/>
	);
}

export function RailShell({
	sections,
	section,
	panels,
	panel,
	reading,
	progress,
	menu = false,
	phone = 'list',
	home,
	onGo,
	onSlug,
	onHome,
	column,
	columnClassName = '',
	overlay,
	held = null,
	detour = null,
	nested = null,
	errand = null,
	away = false,
	children,
}: {
	sections: RailEntry[];
	section: string;
	/** What the right rail offers. A section with nothing to inspect passes none and gets no rail. */
	panels: RailEntry[];
	panel: string;
	/** What the slug prints below `lg`. Every screen has to be able to say where it is. */
	reading: RailReading;
	/** How far through a run, 0–1. Given only while one is going; the slug's edge draws it. */
	progress?: number;
	/** Whether the slug's menu is raised. The workbench cannot tap, so the state is a prop. */
	menu?: boolean;
	/** Which of the two columns a phone shows. There is no room for both. */
	phone?: 'list' | 'panel';
	/**
	 * The one place every inner page leaves for, named — given only by a frame whose ground is
	 * a **place** rather than a rail.
	 *
	 * The split rail is that frame: the ground is the landing page and every destination is a
	 * whole page over it, so leaving a page is one fixed move rather than a step back through a
	 * history. Declaring it puts an arrow at the slug's left on an inner page and leaves the
	 * rest of the bar doing its own job. A frame whose ground is the section register passes
	 * nothing: there the mark heads the rail and the whole slug is the way back.
	 */
	home?: string;
	/** A destination was picked, in either rail or in the raised menu. */
	onGo?: (id: string) => void;
	/** The slug's body was tapped: raise or lower the register. */
	onSlug?: () => void;
	/** The back arrow was tapped. Always `home`, from any page and any depth. */
	onHome?: () => void;
	/** The left column's contents. The shell gives it the scroll and the rule. */
	column: ReactNode;
	/** What width the shell gives that column. */
	columnClassName?: string;
	/** A dialog over the whole frame. */
	overlay?: ReactNode;
	/**
	 * What the open destination is holding, and would lose — a draft, a run. `null` is the resting
	 * frame.
	 *
	 * Given, the slug draws this and nothing else: the resting bar, the back arrow and the way into
	 * the register are all gone for the length of the errand, and the frame's only ways off the page
	 * below `lg` are answering the hold or the rail above `lg`, where both halves are on screen
	 * anyway. Nothing navigates away until the changes are saved or put back.
	 */
	held?: HeldStatus | null;
	/** A page's errand on the ground, below `lg`. `null` is the resting frame. */
	detour?: DetourStatus | null;
	/**
	 * **The open page is a nested one.** `null` is the resting frame. Given, the slug's box grows a
	 * level row above its own row, whatever that row is, and the floor grows with it.
	 */
	nested?: NestedPage | null;
	/**
	 * **The page's own errand, as a cell on its resting bar.** Drawn only while the page itself
	 * is on screen at rest — not on a nested page, whose level row is where you are, and not while
	 * anything is held or detoured, whose rows replace the bar.
	 */
	errand?: ErrandCell | null;
	/** The frame is put away under a screen standing in its place; see `useKeyboardKept`. */
	away?: boolean;
	/** The panel's contents. */
	children: ReactNode;
}) {
	const onPanel = phone === 'panel';

	/* The phone's chrome owns the keyboard's scroll: see `kit/screen`. */
	useTypingScroll();

	/**
	 * What the column has pinned to its own foot, if anything. The slug rides above it and comes
	 * back down when it goes — see `kit/FootRoom` for why the column's floor does not move instead.
	 */
	const [footRoom, claimFootRoom] = useFootRoom();

	/**
	 * The level row stands directly on the slug's own in every face: over the hold's row, and
	 * between the raised register and the bar, so the box still reads from the thumb up. It is where
	 * you are only while the page is what is on screen.
	 */
	const levelBack = useRef<HTMLButtonElement>(null);
	const levelRow =
		nested === null ? null : <LevelRow nested={nested} here={onPanel} back={levelBack} />;

	/**
	 * **Opening a nested page puts the keyboard on its way back**, as a detour's row does:
	 * the name that opened it is on the page it replaced. Keyed on which record, so coming back from
	 * a peek — which draws the glyph again — leaves the keyboard where the peek put it. From `lg` the
	 * slug is not drawn, the call does nothing, and the spread's step takes it.
	 */
	const nestedId = nested?.id ?? null;
	useEffect(() => {
		if (nestedId === null) return;
		levelBack.current?.focus({ preventScroll: true });
	}, [nestedId]);

	/** The resting bar's body: where the keyboard goes when the slug has nowhere else to put it. */
	const barBody = useRef<HTMLButtonElement>(null);

	/**
	 * Any press that takes its own control off the page lands the keyboard on the bar, and on
	 * the page's head while a hold's or a detour's row stands where the bar was.
	 */
	useKeyboardKept(() => (drawn(barBody.current) ? barBody.current : pageHeadIn(document)), away);

	/**
	 * **A hold answered from the slug hands the keyboard to the bar.** The cell that answered it
	 * went with the hold's row, which left the keyboard on `<body>`; the bar comes back in the same
	 * place, so that is where it lands. A frame later, so a confirm that asked has let go of it first,
	 * and only if nothing else has taken it.
	 */
	const holding = held !== null;
	const wasHolding = useRef(holding);
	useEffect(() => {
		const was = wasHolding.current;
		wasHolding.current = holding;
		if (!was || holding) return;
		const frame = requestAnimationFrame(() => {
			const bar = barBody.current;
			if (focusLost() && drawn(bar)) bar.focus({ preventScroll: true });
		});
		return () => {
			cancelAnimationFrame(frame);
		};
	}, [holding]);

	/**
	 * **Escape lowers a raised register** — the keyboard's way of tapping anywhere that is not
	 * it — and puts the keyboard back on the bar that raised it, since a row it was on has gone
	 * with the register. Only where the register can be lowered and the slug is drawn, and never over
	 * a press something inside the page already took.
	 */
	const lower = menu && held === null && detour === null ? onSlug : undefined;
	useEffect(() => {
		if (lower === undefined) return;
		const onKey = (event: KeyboardEvent) => {
			const bar = barBody.current;
			if (event.key !== 'Escape' || event.defaultPrevented || !drawn(bar)) return;
			event.preventDefault();
			lower();
			bar.focus();
		};
		window.addEventListener('keydown', onKey);
		return () => {
			window.removeEventListener('keydown', onKey);
		};
	}, [lower]);

	return (
		<FootRoomProvider value={claimFootRoom}>
			<div
				style={
					{
						'--ui-slug-floor': `${String(SLUG_SUM + (nested === null ? 0 : LEVEL_ROW))}px`,
					} as CSSProperties
				}
				className={`relative flex ${SCREEN_H} min-h-0 bg-ui-bg font-ui-sans text-ui-ink [color-scheme:var(--ui-scheme)]`}
			>
				<nav
					aria-label="Sections"
					className={`hidden ${RAIL_LEFT} shrink-0 flex-col ${TOP} lg:flex`}
				>
					<span className="flex h-11 w-11 shrink-0 items-center justify-center">
						<Mark />
					</span>
					{sections.map(entry => (
						<RailButton
							key={entry.id}
							entry={entry}
							active={entry.id === section}
							edge="left"
							onGo={onGo}
						/>
					))}
				</nav>

				{/* `min-w-0`: a flex item's automatic minimum is its min-content, so without it one
			    unbreakable line in a panel widens the whole page past the glass. */}
				<div className="flex min-h-0 min-w-0 flex-1 flex-col">
					{/* `isolate`: the page halves are one stacking context, so a z-index inside a page
					    — a pinned month head, a column's sticky heads — ranks only against the page
					    and never climbs over the slug or its scrim, which are positioned with no
					    z-index of their own and paint in document order. */}
					<div className="isolate flex min-h-0 flex-1 lg:pl-ui-gutter">
						{/* `TOP` inside the scroller rather than over it: see `kit/SplitShell`'s
						    `Ledger`. A column's sticky objects are held against its scroller's
						    content box, so chrome above that box is room they cannot reach. */}
						<RuledScroll
							className={`${onPanel ? 'hidden' : 'flex max-lg:flex-1'} lg:flex ${columnClassName}`}
						>
							<div className={TOP}>{column}</div>
						</RuledScroll>

						{/* The panel's rule runs the full height, head included, so the two columns are
					    ruled the same way; the thumb rides it but reports only the body, which is
					    the only part that scrolls. */}
						<div
							className={`${onPanel ? 'flex' : 'hidden'} min-h-0 min-w-0 flex-1 flex-col px-ui-gutter ${TOP} lg:flex lg:border-r lg:border-ui-rule lg:pr-0 lg:pl-8`}
						>
							{children}
						</div>

						{panels.length > 0 ? (
							<nav
								aria-label="Panels"
								className={`hidden ${RAIL_RIGHT} shrink-0 flex-col ${TOP} lg:flex`}
							>
								{panels.map(entry => (
									<RailButton
										key={entry.id}
										entry={entry}
										active={entry.id === panel}
										edge="right"
										onGo={onGo}
									/>
								))}
							</nav>
						) : null}
					</div>

					<div className="lg:hidden">
						{menu && held === null && detour === null ? <Scrim onClose={onSlug} /> : null}

						<SlugFloat room={footRoom}>
							<div className={SLUG}>
								{held !== null ? (
									<>
										{levelRow}
										<StatusRow status={held} returnsTo={nested?.name} page={nested?.page} />
									</>
								) : detour !== null ? (
									<>
										{levelRow}
										<DetourRow detour={detour} />
									</>
								) : (
									<>
										{menu ? (
											<>
												<nav
													aria-label="Sections"
													className="flex flex-col border-b border-ui-rule"
												>
													{sections.map(entry => (
														<MenuRow
															key={entry.id}
															entry={entry}
															active={entry.id === section && nested === null}
															onGo={onGo}
														/>
													))}
												</nav>

												{panels.length > 0 ? (
													<nav
														aria-label="Panels"
														className="flex flex-col border-b border-ui-rule"
													>
														{panels.map(entry => (
															<MenuRow
																key={entry.id}
																entry={entry}
																active={onPanel && entry.id === panel}
																onGo={onGo}
															/>
														))}
													</nav>
												) : null}
											</>
										) : null}

										{levelRow}

										<div className="relative flex h-ui-touch w-full items-center">
											{progress === undefined ? null : <SlugGauge share={progress} />}

											{onPanel && home !== undefined ? (
												<>
													<button
														type="button"
														onClick={onHome}
														aria-label={`Back to ${home}`}
														className={`flex h-ui-touch w-ui-touch shrink-0 cursor-pointer items-center justify-center text-ui-ink-2 hover:text-ui-ink ${FOCUS}`}
													>
														<ArrowLeft aria-hidden className="h-4 w-4" />
													</button>
													<span aria-hidden className="h-6 w-px shrink-0 bg-ui-rule" />
												</>
											) : null}

											<button
												ref={barBody}
												type="button"
												onClick={onSlug}
												aria-expanded={menu}
												aria-label={
													onPanel && home === undefined ? `Back to ${reading.place}` : undefined
												}
												className={`flex h-ui-touch min-w-0 flex-1 cursor-pointer items-center gap-3 px-3 ${FOCUS}`}
											>
												{onPanel ? (
													home === undefined ? (
														<ChevronLeft aria-hidden className="h-4 w-4 shrink-0 text-ui-ink-2" />
													) : null
												) : (
													<Mark />
												)}
												<span className="truncate text-ui-body text-ui-ink">{reading.place}</span>
												<span className="ml-auto shrink-0 text-ui-body text-ui-ink-3">
													{reading.fact}
												</span>
											</button>

											{onPanel && nested === null && errand !== null ? (
												'href' in errand ? (
													<a href={errand.href} aria-label={errand.label} className={CELL_BOX}>
														<errand.icon aria-hidden className="h-4 w-4" />
													</a>
												) : (
													<button
														type="button"
														aria-label={errand.label}
														onClick={errand.onPress}
														className={CELL_BOX}
													>
														<errand.icon aria-hidden className="h-4 w-4" />
													</button>
												)
											) : null}
										</div>
									</>
								)}
							</div>
						</SlugFloat>
					</div>
				</div>

				{overlay}
			</div>
		</FootRoomProvider>
	);
}
