import { Button } from '@app/ui/kit/Button';
import { createContext, useContext, useId } from 'react';

/**
 * **The foot of a thing: every control that finishes, leaves or destroys what is above it.**
 *
 * Reach for it wherever a block of content ends in a set of controls — a document, an open row, a
 * record being written. Do **not** reach for it for a single control standing in the flow, which is
 * a `Button` and needs no block around it.
 *
 * **A foot is a block, not a row.** It opens with the same `ui-rule` hairline a section band draws,
 * and every member of it is at least a box: a foot has no bare words — *a control button is a
 * button*. Four text buttons in a row under a box are four labels, and the one that destroys the
 * record is reached by the same press as the one that saves it.
 *
 * **Anatomy**
 *
 * ```
 * ──────────────────────────────────────────────  ← 1px ui-rule, the full measure
 *                                                    12px
 * Needs a title.                                  ← the reason, at most one, ui-red when the
 *                                                    record is wrong and ui-ink-3 when it is
 *                                                    merely nothing-to-do        12px
 * [ Save changes ] [ Discard changes ]            ← band one: the finisher first, then up to
 *                                                    BAND_CAP members, gap-2    12px
 * [ Delete note ]                                 ← band two: what destroys the record, alone
 * ```
 *
 * **Three laws, and they are arithmetic rather than layout.**
 *
 * - **`BAND_CAP` members on a band, and the destroying member is always alone below.** Never a
 *   wrap: the bands are `flex-nowrap` at every width, so `Delete` cannot be pushed down by a long
 *   word on a narrow screen and cannot come up beside `Save` on a wide one. It is where it is
 *   because of what it does.
 * - **Two rungs, and `quiet` is not one of them.** The plate for the member that finishes the
 *   errand, the hairline box for every other member, `TextButton` nowhere.
 * - **The reason stands in the block, once.** A control that is off owes a reason, and the
 *   reason belongs to the foot rather than to the button, so a blocked member takes `disabled` and
 *   `describedBy` and the paragraph does the printing. At most one member carries one.
 *
 * **The space above the block belongs to whatever it is the foot of**, and never to the block. A
 * fixed `pt-8` would be right for a document, where 32px is the rhythm every section already stands
 * on, and wrong inside an open row, where the same 32px puts the members 82px off the box they are
 * the controls for.
 */

/**
 * Members on a band before the next one starts a second.
 *
 * A number rather than a feeling, so the rule can be checked on screen: count the boxes on band
 * one. The destroying member never counts against it — it is below by its own rule.
 */
export const BAND_CAP = 3;

/** The two rungs a foot has. `quiet` is not one of them, anywhere. */
export type PlinthWeight = 'primary' | 'normal';

/** One member of a foot. */
export type PlinthMember = {
	id: string;
	word: string;
	/** Defaults to the hairline box. Exactly one member of a foot is the plate. */
	weight?: PlinthWeight;
	/** The word to print while this member's run is going — `Button`'s own `running`. */
	running?: string | null;
	/** Why it cannot be pressed. Hoisted out of the button and into the plinth's own paragraph. */
	reason?: string | null;
	/** `wrong` sets that paragraph in `ui-red`: the app replying, never a fact about the record. */
	tone?: 'wrong' | 'quiet';
	/**
	 * **Off, and the page already says why**: the id of the sentence that does. The member is off
	 * and described by it, and the foot prints no line of its own. `Select all` while everything is
	 * already picked is off because of a sentence standing directly over the foot, and a second copy
	 * of it under the buttons would be the one fact said twice.
	 */
	saidBy?: string;
	/** An accessible name, where the word is not the whole of it. */
	label?: string;
	/** The one member band two holds. At most one per foot. */
	destroys?: boolean;
	/**
	 * **The chrome carries this member while it is holding the errand, so it is drawn once.**
	 *
	 * It is a fact about the *condition*, not about the member: the commit is in the slug only
	 * while the record can take one and the discard only while something is held, so what the page
	 * gives up is exactly what the box has and never more. Two things have to be true before
	 * anything is given up — this flag, and a hold actually being up, which the shell publishes and
	 * `HoldingProvider` carries. Above `lg` there is no slug and nothing is given up at all.
	 */
	chrome?: boolean;
	/**
	 * **The one width this member is drawn at**, where it has no meaning at the other.
	 *
	 * `wide` is drawn from `lg`, where the sheet stands beside the page; `narrow` below it, where
	 * the page is all there is. A page's `Select all` and `Clear all` are `wide` — below `lg` they
	 * are the slug's cells while the phone is picking — and a press that sends the phone to the
	 * sheet is `narrow`, because above `lg` the sheet is already on screen. Unlike `chrome` this is a fact
	 * about the width alone, so it is CSS and never a hook: no condition is read in JavaScript, and
	 * `BAND_CAP` is counted per width.
	 */
	at?: 'wide' | 'narrow';
	onClick?: () => void;
};

/**
 * **Whether the chrome is holding this page's errand**, which is the other half of `chrome`.
 *
 * `kit/SplitShell` publishes it from the one `held` a place declares, so the slug's cells and the
 * page's members are read off a single fact rather than two conditions that agree today. It is
 * false by default, so a plinth drawn outside a frame — a specimen, the kit's stress page — gives
 * nothing up and draws every member it was handed.
 *
 * The context lives here, next to the only thing that reads it, the way `kit/viewing` owns the
 * viewing context the shell provides.
 */
const HoldingContext = createContext(false);

export const HoldingProvider = HoldingContext.Provider;

/**
 * A member the chrome is holding: hidden below `lg` and drawn above it.
 *
 * `lg:contents` and not a bare `lg:block`: the wrapper has to stop existing as far as the band's
 * flex is concerned wherever the member *is* drawn, or the member stops being the flex item
 * `kit/Button` was written to be.
 */
const IN_CHROME = 'max-lg:hidden lg:contents';

/** The same, on a band every one of whose members the chrome holds. The band keeps its `flex`. */
const BAND_IN_CHROME = 'max-lg:hidden';

/** A member drawn at one width only — `IN_CHROME`'s own wrapper, and its mirror. */
const AT: Record<'wide' | 'narrow', string> = {
	wide: IN_CHROME,
	narrow: 'lg:hidden max-lg:contents',
};

/** Whether a member is drawn at a width. */
function drawnAt(member: PlinthMember, width: 'wide' | 'narrow'): boolean {
	return member.at === undefined || member.at === width;
}

/**
 * Band one, filled up to `BAND_CAP` **at each width**: a member joins it while the band still has
 * room at every width the member is drawn at, so `Finish · Select all · Clear all` above `lg` and
 * `Finish · Pick on the sheet` below it are one band of four members and never two bands of three.
 */
function bandOne(members: PlinthMember[]): PlinthMember[] {
	const band: PlinthMember[] = [];
	for (const member of members) {
		const fits = (['wide', 'narrow'] as const).every(
			width => !drawnAt(member, width) || band.filter(one => drawnAt(one, width)).length < BAND_CAP
		);
		if (!fits) break;
		band.push(member);
	}
	return band;
}

function PlinthControl({ member, reasonId }: { member: PlinthMember; reasonId: string }) {
	const holding = useContext(HoldingContext);
	const reason = member.reason ?? null;

	const button = (
		<Button
			weight={member.weight ?? 'normal'}
			running={member.running ?? null}
			disabled={reason !== null || member.saidBy !== undefined}
			describedBy={reason === null ? member.saidBy : reasonId}
			label={member.label}
			onClick={member.onClick}
		>
			{member.word}
		</Button>
	);

	if (member.at !== undefined) return <span className={AT[member.at]}>{button}</span>;
	if (member.chrome !== true || !holding) return button;
	return <span className={IN_CHROME}>{button}</span>;
}

/**
 * One band of the block.
 *
 * `flex-nowrap` at every width — which is the default, and is written out because it is the claim.
 * Below `md` the members take a line each in the same reading order, which they get from the
 * column's own `stretch`: a member is never given a width, so nothing here can disagree with
 * `kit/Button` about what a button is. `md` and not `lg`, because this is the **measure's**
 * breakpoint and not the chrome's: a tablet has 512px of line where a phone has 350.
 */
function PlinthBand({ members, reasonId }: { members: PlinthMember[]; reasonId: string }) {
	const holding = useContext(HoldingContext);

	/**
	 * A band the chrome holds entirely is hidden as a band, not as three members: `space-y-3` gives
	 * its margin to the next child whether or not the one before it drew anything, so an emptied
	 * band would leave 12px of nothing between the reason and `Delete note`.
	 */
	const spent = holding && members.every(member => member.chrome === true);

	return (
		<div className={`flex flex-col flex-nowrap gap-2 md:flex-row ${spent ? BAND_IN_CHROME : ''}`}>
			{members.map(member => (
				<PlinthControl key={member.id} member={member} reasonId={reasonId} />
			))}
		</div>
	);
}

/**
 * The foot, as a block: a hairline, what the record owes, and the members in one or two bands.
 *
 * `break-inside-avoid` because the block is one statement and the column rule would otherwise be
 * free to put the hairline at the foot of one column and the members at the head of the next.
 *
 * **Nothing is reserved for the reason.** The paragraph is drawn or it is not, so a foot with
 * nothing owed closes up under its hairline.
 */
export function Plinth({ members }: { members: PlinthMember[] }) {
	const reasonId = useId();

	const destroying = members.filter(member => member.destroys === true);
	const rest = members.filter(member => member.destroys !== true);
	const above = bandOne(rest);
	const below = [...rest.slice(above.length), ...destroying];
	const reasoned = members.find(member => (member.reason ?? null) !== null);

	return (
		<div className="break-inside-avoid">
			<div className="space-y-3 border-t border-ui-rule pt-3">
				{reasoned === undefined ? null : (
					<p
						id={reasonId}
						className={`text-ui-body ${reasoned.tone === 'wrong' ? 'text-ui-red' : 'text-ui-ink-3'}`}
					>
						{reasoned.reason}
					</p>
				)}

				{above.length === 0 ? null : <PlinthBand members={above} reasonId={reasonId} />}
				{below.length === 0 ? null : <PlinthBand members={below} reasonId={reasonId} />}
			</div>
		</div>
	);
}
