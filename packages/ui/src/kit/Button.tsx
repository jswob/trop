import type { LucideIcon } from 'lucide-react';
import type { ReactNode, Ref } from 'react';
import { useId } from 'react';

import { CONTROL_H } from '@app/ui/kit/control';
import { FOCUS } from '@app/ui/kit/focus';

/**
 * The button set — emphasis is how much surface the button owns.
 *
 * Three rungs, and the drop from one to the next is a drop in area: a filled plate, a hairline
 * box, a bare word. Three shapes cross the rungs — a word, a word with a glyph, and a glyph
 * alone — and a shape is never a fourth rung: `IconButton` is one rung, for reasons written on
 * it.
 *
 * **There is no `danger` rung.** A red plate would be the one rung identified by a *hue* rather
 * than by an amount of surface — hue carrying emphasis, which belongs to the ink ramp. It is also
 * unbuildable: on a near-black ground no red is both light enough for a dark word and dark enough
 * for a light one, and the best chroma sits in the dead zone between. So destruction is drawn as
 * what it is — the errand of its own modal, and therefore that modal's `primary` — and red warns
 * in type beside it. The word and the modal carry the meaning anyway; hue is never the sole
 * carrier.
 *
 * **Which rung an action gets is the caller's decision, never the record's.** The same delete
 * is the errand's whole point in a modal and a footnote in a document, so weight travels with
 * the screen. What the kit fixes is what each rung is made of.
 *
 * Two things this file does not reopen. The primary is an **ink plate** — loud by value, because
 * a coloured action would compete with the one measure a screen exists to move. And **a pending
 * button draws no progress**: the run belongs to the foot's top edge, so a button that is working
 * goes inert and changes its word.
 */

/** The three rungs. `quiet` is the text button; `primary` is the one ink plate. */
export type ButtonWeight = 'primary' | 'normal' | 'quiet';

/**
 * Each rung at rest, with its pointer states on it.
 *
 * Written as **literal class strings** rather than composed at the call site: Tailwind v4
 * generates only what it has read in the source, so a `hover:` variant assembled at runtime
 * is a class that never exists, and a specimen drawn with it is inert.
 *
 * The plate **dims** under the pointer because `ui-ink` is the top of the ramp and down is the
 * only direction left — which is what a pressed key does anyway. The two lower rungs go the
 * other way: they have no fill to lose, so the pointer gives them one.
 *
 * **There is no coloured rung.** The primary action is loud by value and not by hue, because a
 * coloured action competes with the one measure a screen exists to move.
 */
const LIVE: Record<ButtonWeight, string> = {
	primary: 'bg-ui-ink font-medium text-ui-bg hover:bg-ui-ink-2 active:bg-ui-ink-3',
	normal: 'border border-ui-edge text-ui-ink-2 hover:bg-ui-row hover:text-ui-ink active:bg-ui-beat',
	quiet: 'text-ui-ink-2 hover:bg-ui-row hover:text-ui-ink active:bg-ui-beat',
};

/**
 * Off. The plate empties to the row ground and every rung drops to `ui-ink-3`.
 *
 * `ui-ink-3` on `ui-beat` measures **3.36:1**, under the system's 5.0 floor and deliberately so:
 * WCAG exempts an inactive control, and a disabled button that still reads at full strength is
 * a button that looks available. The normal rung's edge drops to `ui-rule` for the same reason —
 * it is the one place the rule that a control is never outlined in `ui-rule` (`controlBox`) does
 * not apply, because this one is not a control any more.
 */
const OFF: Record<ButtonWeight, string> = {
	primary: 'bg-ui-beat text-ui-ink-3',
	normal: 'border border-ui-rule text-ui-ink-3',
	quiet: 'text-ui-ink-3',
};

/**
 * Working. **Not the same as off** — a run is busy, not unavailable.
 *
 * The plate goes out to the row ground the way a disabled one does, and then the ink stays at
 * the top where a disabled control drops to `ui-ink-3`. That pair is the whole distinction, and
 * the word says the rest. The normal rung keeps its edge so the box does not change size while
 * it waits.
 */
const BUSY: Record<ButtonWeight, string> = {
	primary: 'bg-ui-beat text-ui-ink',
	normal: 'border border-ui-edge bg-ui-beat text-ui-ink',
	quiet: 'bg-ui-beat text-ui-ink',
};

/**
 * The geometry, which no rung varies: the control's own height, radius 0, 12px of side
 * padding, and 8px between a glyph and its word. `gap-2` with one child is nothing, so a word
 * button and a word-and-glyph button are the same box rather than two.
 */
const WORD = `flex ${CONTROL_H} items-center justify-center gap-2 px-3 text-ui-body`;

/**
 * **A word is centred on its type, not on its line box.**
 *
 * `items-center` centres the *line box*, and a line box is not symmetrical about the type in it:
 * Archivo's content box at 13px stands 11px over the baseline and 3px under it, where the visible
 * cap is 8.92 and a descender reaches 2.25. Measured in a 32px button, that put the cap 11.08px
 * from the top and the baseline 12.00px from the bottom — the word sitting **0.46px high**, on
 * every button in the app, in the same direction every time.
 *
 * `text-box` trims the box to the band the eye actually reads, cap to baseline, so centring it is
 * centring the type: 11.54 above and 11.54 below, at any size and any face. It is one declaration
 * rather than a nudge because a nudge would be this font at this size, and would be wrong at the
 * next one. A browser that does not know the property ignores it and draws what it drew before,
 * which is the old half-pixel and not a broken box.
 *
 * It goes on the **word, never on a line of prose**: trimming an inline label would pull its hover
 * ground off the line it sits in. `LABEL_INLINE` does not take it.
 */
const CAP = '[text-box:trim-both_cap_alphabetic]';

/**
 * The pointer's answer, and the one motion a button has.
 *
 * Motion here only ever answers something the user did, and a ground arriving under a pointer
 * is exactly that — the button is not greeting anybody, it is replying. What it may not do is put
 * itself between a finger and its own feedback, so the grammar is two-part and asymmetric:
 *
 * **Hover eases, press does not.** 120ms on the way in and on the way out; `active` drops the
 * duration to zero, so the pressed ground lands on the frame the mouse goes down and eases back
 * when it comes up. A press that fades in is a control reporting late.
 *
 * The properties are **named rather than `transition-colors`**, which in v4 also carries
 * `outline-color`: the focus ring's width appears at once and its colour would then crossfade out
 * of `currentColor`, which is a ring that flashes the wrong colour. A ring is a statement about
 * where you are and it is never animated.
 *
 * 120ms is under the ~150ms at which a colour change stops reading as immediate, and it is
 * deliberately not one of the frame's own durations (320 · 240 · 160): those move a boundary, and
 * nothing here moves.
 */
export const MOTION =
	'transition-[color,background-color,border-color] duration-[120ms] ease-out active:duration-0 motion-reduce:transition-none';

/**
 * A button.
 *
 * `running` is the only state the client cannot work out on its own — whoever owns the run
 * knows — and it arrives as the word to print while it happens, because the idle word and the
 * working word are not derivable from each other (`Save changes` / `Saving`).
 *
 * `blocked` is the other: a disabled control with no reason is a dead end, and the reason is a
 * fact about the data that no label can carry. It disables the button, prints the reason beside
 * it, and wires the two together with `aria-describedby` so the reason is not sighted-only.
 *
 * `describedBy` is `blocked` with the printing taken out, for the one caller that has somewhere
 * better to print — `kit/Plinth`, whose whole answer to *where the reason stands* is a paragraph
 * over the block. The three jobs `blocked` welds together come apart into the two the button owes
 * and the one the block has claimed, and the string stays in the DOM once. It is `disabled` that
 * turns the control off there, because a control with a reason somewhere else is still a control
 * that cannot be pressed.
 *
 * `href` makes it a link drawn as a button: a page's `Add a note` goes to a page, and a press that
 * goes somewhere is an `<a>`, so it opens in a new tab like any link and the frame reads it back
 * like any other. It takes no `running` or `blocked` — a link is never busy and never off; a
 * destination that cannot be reached is a button with a reason instead.
 */
export function Button({
	children,
	weight = 'normal',
	icon: Icon,
	running = null,
	blocked = null,
	describedBy,
	disabled = false,
	label,
	href,
	pressed,
	onClick,
}: {
	children: ReactNode;
	weight?: ButtonWeight;
	icon?: LucideIcon;
	/** The word to print while the run this button started is going. */
	running?: string | null;
	/** Why it cannot be pressed. Printed beside it, and disables it. */
	blocked?: string | null;
	/**
	 * The id of what says why, where something other than this button is printing it.
	 *
	 * Undefined default, and ignored where `blocked` is set — that one prints its own reason and
	 * points at it. Pass `disabled` with it: this wires the announcement and nothing else.
	 */
	describedBy?: string;
	disabled?: boolean;
	/** An accessible name, where the word in the button is not the whole of it. */
	label?: string;
	/** Where it goes, where it is a link rather than a press. */
	href?: string;
	/**
	 * **A button that stays in**: a `Pinned` toggle, say. It is the wire alone — `aria-pressed` —
	 * and the caller draws it in by the rung it hands over: `primary` while in, `normal` while out.
	 * A period's `No start` / `No end` is a text button that stays in — `TextButton`'s `pressed` with
	 * `held`.
	 * Undefined default: every other button is not a toggle and says nothing about being pressed.
	 */
	pressed?: boolean;
	onClick?: () => void;
}) {
	const reasonId = useId();
	const busy = running !== null;
	const off = disabled || blocked !== null;

	if (href !== undefined) {
		return (
			<a
				href={href}
				aria-label={label}
				onClick={onClick}
				className={`${WORD} ${MOTION} ${LIVE[weight]} ${FOCUS} cursor-pointer`}
			>
				{Icon === undefined ? null : <Icon aria-hidden className="h-4 w-4 shrink-0" />}
				<span className={CAP}>{children}</span>
			</a>
		);
	}

	const button = (
		<button
			type="button"
			disabled={busy || off}
			aria-busy={busy || undefined}
			aria-pressed={pressed}
			aria-label={label}
			aria-describedby={blocked === null ? describedBy : reasonId}
			onClick={onClick}
			className={`${WORD} ${MOTION} ${
				busy ? BUSY[weight] : off ? OFF[weight] : `${LIVE[weight]} ${FOCUS}`
			} ${busy || off ? 'cursor-default' : 'cursor-pointer'}`}
		>
			{Icon === undefined ? null : <Icon aria-hidden className="h-4 w-4 shrink-0" />}
			<span className={CAP}>{running ?? children}</span>
		</button>
	);

	if (blocked === null) return button;

	return (
		<span className="inline-flex items-center gap-3">
			{button}
			<span id={reasonId} className="text-ui-body text-ui-ink-3">
				{blocked}
			</span>
		</span>
	);
}

/**
 * The icon button's box, on its own so a thing that is **not** a button can take the same one.
 *
 * `kit/Arm`'s marker is the caller: armed, it is a `span` rather than a press, and it has to be the
 * same square at the same place or the row's columns move when it arms.
 */
export const ICON_BOX = `flex ${CONTROL_H} w-ui-touch shrink-0 items-center justify-center lg:w-ui-control`;

/**
 * An icon-only control, square at the control's own height.
 *
 * **It has one rung, and it is the quiet one.** The middle rung exists to keep a button from
 * being read as a field — an unfilled `Button` and a `Field` share a silhouette — and a 32px
 * square holding a glyph is never mistaken for somewhere to type. A bordered button on a dense
 * read surface is out besides, and that is where most of these live. So there is one icon button,
 * not a bordered one here and bare ones in `MonthPicker` and elsewhere.
 *
 * **Square at the control's own height** — 40px below `lg`, 32px above — in every state,
 * pressable or not, so a column of them keeps one measure.
 *
 * It is always rendered. Revealing a row's actions on `hover` was tried and rejected: tablet and
 * phone have no hover, so the action simply does not exist there.
 */
export function IconButton({
	children,
	label,
	disabled = false,
	describedBy,
	onClick,
	ref,
}: {
	children: ReactNode;
	label: string;
	disabled?: boolean;
	/**
	 * The id of the words that say why it cannot be pressed, printed by the caller beside it — a
	 * disabled control's reason, for the square that has no room to print one: the pager's arrows,
	 * say. Opt-in.
	 */
	describedBy?: string;
	onClick?: () => void;
	/**
	 * The button itself, for a caller that has to put the keyboard on it — a nested page's way
	 * back, when the name that opened it is no longer on screen. Opt-in.
	 */
	ref?: Ref<HTMLButtonElement>;
}) {
	return (
		<button
			ref={ref}
			type="button"
			disabled={disabled}
			aria-label={label}
			aria-describedby={describedBy}
			onClick={onClick}
			className={`${ICON_BOX} ${MOTION} ${
				disabled ? `${OFF.quiet} cursor-default` : `${LIVE.quiet} cursor-pointer ${FOCUS}`
			}`}
		>
			{children}
		</button>
	);
}

/**
 * A written value that is also the control that changes it — the app's text button.
 *
 * **A clickable label is drawn as the text it replaces**, with a pointer cursor and a hover
 * ground, and the target is the label rather than the row band. It is the quiet rung with the
 * padding taken back out — `-mx-1 px-1` cancel, so the word sits exactly where the plain word sat
 * and only the ground is wider than it.
 *
 * It comes in two heights, and the height is the one thing that differs between them —
 *
 * **`LABEL_ROW`** is a label standing on its own line. It takes the control floor, 40px below
 * `lg` and 32px above, because a line you can press is a control whatever it is called.
 *
 * **`LABEL_INLINE`** is a label set inside a line of prose. It takes the line's height, which
 * is under the floor — measured, a two-word label is 61×21 — and that is WCAG 2.5.8's own
 * exemption for a target in a sentence. Reach for it only where the words around it are not
 * pressable.
 *
 * Both brighten to `ui-ink` under the pointer rather than leaving each caller to remember.
 *
 * Both are **private**. See `TextButton` below for why.
 */
const LABEL = `-mx-1 cursor-pointer px-1 text-left hover:bg-ui-row hover:text-ui-ink active:bg-ui-beat ${MOTION} ${FOCUS}`;

const LABEL_ROW = `flex ${CONTROL_H} items-center ${LABEL}`;

const LABEL_INLINE = LABEL;

/**
 * The same two shapes with everything that says *operable* removed — see `TextButton`'s `still`.
 *
 * `-mx-1 px-1` stays, and that is the whole reason this is a pair of constants and not the absence
 * of one: the pair cancels, so the word stands on precisely the x it stood on as a button and a
 * sheet does not reflow when it stops being worked.
 */
const STILL_ROW = `flex ${CONTROL_H} items-center -mx-1 px-1 text-left`;

const STILL_INLINE = '-mx-1 px-1 text-left';

/**
 * Which ends of a text button sit on the edge of the measure.
 *
 * The overhang is `LABEL`'s: `-mx-1 px-1` cancel, so the word sits exactly where the plain
 * word sat and only the ground is wider than it. At the **end of a line** that rule collides with
 * the page's: the ground lands past the measure, which is 4px of document hanging over the column
 * and reads as the plate having been cut rather than the line having ended.
 *
 * Trimming the outer side settles both at once — the word still sits where a plain word sits, and
 * the ground stops where the page stops. The ground is then flush with the word on that side and
 * still 4px clear on the inside, which is what a printed page does at its own margin.
 *
 * Undefined default: a button that says nothing overhangs both ways exactly as it always has.
 *
 * **Never `mx-0 px-0`.** Tailwind orders utilities by property and then by value, so `px-0` and
 * `px-1` land in the same group and the larger value wins — measured, an `edge="both"` button kept
 * its 4px padding and its word stopped 4px short of the measure at both ends. The one-sided
 * utilities sort *after* the axis they narrow, so `pl-0` does beat `px-1`.
 */
export type LabelEdge = 'start' | 'end' | 'both';

const LABEL_EDGE: Record<LabelEdge, string> = {
	start: 'ml-0 pl-0',
	end: 'mr-0 pr-0',
	both: 'ml-0 pl-0 mr-0 pr-0',
};

/** How loud a written value is at rest. It brightens to `ui-ink` under the pointer either way. */
export type LabelInk = 'ink' | 'ink-2' | 'ink-3';

/** The type ramp, as far up it as a written value needs to go. */
const TEXT_SIZE: Record<'body' | 'lead' | 'title', string> = {
	body: 'text-ui-body',
	lead: 'text-ui-lead',
	title: 'text-ui-title',
};

/**
 * `held`'s drawing: the control's own pressed ground, standing. It dims to `ui-row` under the
 * pointer as the plate dims — `ui-beat` is as far up as a text button's ground goes.
 */
const HELD = 'bg-ui-beat text-ui-ink';

const LABEL_INK: Record<LabelInk, string> = {
	ink: 'text-ui-ink',
	'ink-2': 'text-ui-ink-2',
	'ink-3': 'text-ui-ink-3',
};

/**
 * The text button, as a component rather than a class string.
 *
 * A string cannot own a default. Shipped as `LABEL_ROW` and `LABEL_INLINE`, a clickable label
 * leaves every caller pasting one next to a hand-written `<button type="button">` with its own
 * `text-ui-body`, its own rung of the ink ramp and its own `aria-label`, re-deciding the same
 * three things with nothing to check that they agree — and the names on a read sheet, often the
 * only control on it, would be the one part of the app not built on the kit.
 *
 * **It is a third component, not another rung.** `Button` is a target drawn as a target — a box
 * with padding, centred contents and the control's height. This is a *written value that is also
 * the control that changes it*: the word sits exactly where the plain word sat, and only the
 * ground is wider than it. Running the two through one component would mean a `weight` that also
 * decides padding, alignment and whether the thing is a box, which is four decisions wearing one
 * name. `IconButton` is here for the same reason and on the same terms.
 *
 * The two heights survive as `inline`, because that is the one thing that genuinely differs: a
 * label on its own line takes the control floor, and one set inside a sentence takes the line's.
 * `ink` and `size` are the caller's, because emphasis is a decision about the screen and never a
 * fact on the record — the same note's title is `lead` ink on a wide sheet and `body` at `ink-2`
 * on a narrow one. `className` carries what is genuinely local: `truncate`, `w-full`,
 * `shrink-0`, and a container query that steps the pair together.
 */
export function TextButton({
	children,
	ink = 'ink-2',
	size = 'body',
	inline = false,
	still = false,
	edge,
	label,
	expanded,
	controls,
	held = false,
	pressed,
	href,
	className = '',
	onClick,
}: {
	children: ReactNode;
	/** How loud at rest. Defaults to the secondary rung. */
	ink?: LabelInk;
	/**
	 * `lead` for a name that heads a group; `body` for everything else. `title` is opt-in, for a
	 * written value that outranks the page around it.
	 */
	size?: 'body' | 'lead' | 'title';
	/** Set inside a line of prose, taking the line's height rather than the control floor. */
	inline?: boolean;
	/**
	 * The written value with the control taken off it, drawn exactly where the control stood.
	 *
	 * A surface that is being *read* rather than worked needs this for every word on it, as
	 * `kit/Arm`'s armed marker is the same square drawn as a `span`. It keeps the box, the type and
	 * the rung, and drops the four things that say *operable*: the element is not a button, the
	 * pointer is the page's, the ground never arrives under it and it is out of the tab order.
	 *
	 * It is not `disabled`. A disabled control is one you could use and may not, and says so by
	 * losing its edge and a rung of ink; this word is not a control at all, and dimming every name
	 * on a sheet somebody is reading would be the opposite of the point.
	 */
	still?: boolean;
	/**
	 * Which ends of this button stand on the measure. Pass it wherever the button opens or closes
	 * a line of the document, so its ground stops where the page does. See `LabelEdge`.
	 */
	edge?: LabelEdge;
	/** An accessible name, where the word on screen is not the whole of it. */
	label?: string;
	/**
	 * Whether the thing this button opens is standing open.
	 *
	 * Opt-in and **undefined by default**, so a button that opens nothing announces nothing — a
	 * bare `aria-expanded="false"` on a word that has nothing to expand is a lie in the one channel
	 * that cannot be checked by looking. Pass it wherever the press discloses something, together
	 * with `controls`, and draw the state with `kit/Disclose` beside the word: `aria-expanded` is
	 * what the announcement rides on and the glyph is what the eye reads, and neither is allowed to
	 * be the only one.
	 */
	expanded?: boolean;
	/** The id of what `expanded` is about. */
	controls?: string;
	/**
	 * **A written value that is holding something**: the `ui-beat` ground under it at rest, the
	 * word at `ui-ink` whatever `ink` says. A period's end while it holds its month grid open, and
	 * the open word (`No start` / `No end`) while the end is open — the control's own pressed state,
	 * never the ink plate, because what a row opens is never louder than the page. Opt-in.
	 */
	held?: boolean;
	/**
	 * **A text button that stays in** — the wire alone, `aria-pressed`, as `Button`'s `pressed` is.
	 * Draw it in with `held`. Undefined default: a text button is not a toggle and says nothing.
	 */
	pressed?: boolean;
	/**
	 * Where the word goes: a name that is the way to its record's own page. Drawn as an `<a>` in
	 * exactly the button's box, so it opens in a new tab like any link and never navigates from a
	 * button's press. Opt-in: without it the word is a button.
	 */
	href?: string;
	/** Layout the caller owns — `truncate`, `w-full`, `shrink-0`, a container query. */
	className?: string;
	onClick?: () => void;
}) {
	const shape = still ? (inline ? STILL_INLINE : STILL_ROW) : inline ? LABEL_INLINE : LABEL_ROW;
	const box = `${shape} ${TEXT_SIZE[size]} ${held ? HELD : LABEL_INK[ink]} ${
		edge === undefined ? '' : LABEL_EDGE[edge]
	}`;

	const word = inline ? children : <span className={`${CAP} min-w-0`}>{children}</span>;

	if (still) return <span className={`${box} ${className}`}>{word}</span>;

	if (href !== undefined)
		return (
			<a href={href} aria-label={label} onClick={onClick} className={`${box} ${className}`}>
				{word}
			</a>
		);

	return (
		<button
			type="button"
			aria-label={label}
			aria-expanded={expanded}
			aria-controls={expanded === undefined ? undefined : controls}
			aria-pressed={pressed}
			onClick={onClick}
			className={`${box} ${className}`}
		>
			{word}
		</button>
	);
}
