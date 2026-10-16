import type { LucideIcon } from 'lucide-react';
import type { ReactNode } from 'react';

import type { DetourCell, PageHeading, RailReading } from '@app/ui/kit';

/**
 * **What a page declares to the frame** — the half of the workbench's `views/types` the app needs
 * as well. A place is a workbench idea: variants, mirrors, a `Subject` to stage. What a place
 * *draws* is not, so it lives here beside the frame, with no import of anything workbench-only, and
 * the workbench's `views/types` re-exports all of it.
 */

/** What a place puts in the frame. Everything else on the screen belongs to the frame. */
export type ViewDraw = {
	/**
	 * The page in the content column.
	 *
	 * Omitted when the place is the ground and nothing is open over it. A place that *is* a
	 * destination always draws one — that is what makes it that place.
	 */
	page?: ReactNode;
	/**
	 * What the page calls itself, when the destination's own name is not the errand.
	 *
	 * Defaults to `headOf(destination)`. Overridden only where one destination is two errands: *Add a
	 * note* with nothing open, and the note's own title when one is.
	 */
	head?: PageHeading;
	/**
	 * Whether the page is a **stage** right now: set centred on both axes with no head drawn rather
	 * than as a document. A long run is the case, while it goes and after it ends without a result:
	 * a run is a thing to watch, not to read, and the same page idle is an ordinary page again.
	 *
	 * Live, because the page changes under it: an ended run holds nothing, so the hold cannot be
	 * what says so. Undefined, never centred.
	 */
	centred?: Live<boolean>;
	/**
	 * The ledger's contents. Omitted to get the resting sheet — the ground as it usually is.
	 *
	 * Where a state of the **ledger** is caused by a page, it belongs to that page: a draft drawn
	 * into the ground while a page edits it is that page's, and it is drawn here by that page's place.
	 */
	ledger?: ReactNode;
	/** A dialog over the whole frame. */
	overlay?: ReactNode;
	/**
	 * Whether the slug's register is raised — **the one exception to the rule that a state a press
	 * reaches is not listed**.
	 *
	 * The raised register is reached by pressing the slug, so by the rule it should not be listed at
	 * all. It is, because it is the phone's entire navigation and nothing else draws the app's own
	 * destinations with their own facts on them.
	 *
	 * The exception is **counted, not open**: a place lists it at most once, and only where the
	 * register draws something that place itself causes — the ground's destinations at rest. The
	 * register cannot be raised over a hold, so a held destination has no such state to list. A
	 * second entry is the rule being reopened, and the workbench's `views/frame.test` fails it.
	 */
	menu?: boolean;
	/**
	 * What the slug prints: where you are, and exactly one thing about it. Never two.
	 *
	 * Live where the one thing changes under the page — a run's fact is what is left to do while it
	 * is idle, and what became of the run once one has ended.
	 */
	reading: RailReading | Live<RailReading>;
	/** What each destination reports in the raised register, by id. Overrides its own `fact`. */
	facts?: Partial<Record<string, string>>;
	/**
	 * What this page is **holding**: something live that leaving would lose.
	 *
	 * Declaring it arms the whole guard at once — the slug given over to the hold's row, the
	 * guard on every way out above `lg`, the confirm that guard raises, and the ledger drawn to be
	 * read rather than worked. The place says what is held and what can be done about it; nothing
	 * about the chrome that reports it is the place's.
	 *
	 * Only a place that *is* a destination can hold anything. The ground holds nothing: it is what
	 * you would be leaving for.
	 */
	held?: Hold | HoldSource;
	/**
	 * An errand this page sends the phone to the ground for, and waits on: picking records off the
	 * ground for the page. The page asks for it with `useDetour()`; the frame puts the ground on
	 * screen below `lg` with the page still open, and the slug becomes the detour's row. Not a hold —
	 * nothing is lost by going back.
	 */
	detour?: Detour;
	/**
	 * **Records this page can open inside itself.** A nested page takes the page's place whole,
	 * entering and leaving it never asks and never touches a hold — unless the hold is the nested
	 * page's own (`Hold.nested`), when leaving it asks — and the way back always goes to the page it
	 * was opened from, exactly as it was left. The place declares which records and what each page
	 * holds; the frame draws everything around it (`kit/nested`'s `NestedPage`).
	 */
	nest?: Nest;
	/**
	 * **The record the page opens on, inside itself — the second counted exception.**
	 *
	 * A nested page is a press from its page, so by the same rule it should not be listed. One is,
	 * because it is a whole page rather than the position of a hand, and unlisted it is the one page
	 * the workbench never stages at any width. The exception is **counted, not open**: a place lists
	 * at most one, it names a record the place's `nest` can open, and the workbench's
	 * `views/frame.test` fails a second. Everything else a nest opens stays a press.
	 */
	nested?: string;
	/**
	 * **The page's own errand**: the one thing it offers at rest — a list's `Add a note`.
	 *
	 * One declaration, read by both surfaces: below `lg` the frame puts it on the slug's resting bar
	 * as a cell, under the thumb at every scroll position; from `lg` the page draws it in its flow
	 * as `views/frame`'s `ErrandPlate`, and the slug does not exist. Neither surface decides the width
	 * for itself, and the page never reads the breakpoint in JavaScript.
	 */
	errand?: PageErrand;
	/**
	 * **Signed out, and which condition sign-in stands in** — a state of the app, not of a hand.
	 *
	 * Account's `Sign out` reaches the empty one with a press, and it is listed anyway beside the
	 * ones no press reaches: the server still answering, a server nobody set a password on, and one
	 * that never answered. The frame draws sign-in whole in place of the shell (`views/sign-in`),
	 * and signing in lands on the sheet. A wrong password is a press from the empty one, so it has
	 * no entry.
	 */
	signedOut?: SignedOut;
	/**
	 * **A read that failed, and whose** — a condition of the app, which no press reaches.
	 *
	 * `ground`: the sheet's own reads failed, so the frame gives way to a centred screen saying so
	 * (`views/read-failed`). `page`: the page's own reads failed with the ground fine, so every wait
	 * on screen stops and says so (`kit/Waiting`'s `WaitFailed`). `Try again` clears it, as signing
	 * in clears `signedOut`. The app passes `Frame`'s `readFailed` instead.
	 */
	readFailed?: 'ground' | 'page';
};

/** Which condition sign-in opens in — see `ViewDraw.signedOut`. */
export type SignedOut = 'empty' | 'signing-in' | 'not-set-up' | 'unreachable';

/**
 * What a page offers at rest, as its slug cell and its plate: a link, so it opens in a new tab like
 * any other and the frame reads it back. A nested address opens that record inside the page.
 *
 * **Or the frame's one press, `sign-out`**: Account's errand goes nowhere a link can, so both
 * surfaces are buttons, and what the press does is the frame's — a page cannot sign anybody out.
 */
export type PageErrand = {
	label: string;
	icon: LucideIcon;
} & ({ href: string } | { press: 'sign-out' });

/**
 * Records a page can open inside itself: which, what each is called, and what its page holds.
 *
 * A link on the page written in `nestHref(dest, kind, id)` is how one is opened — the press is the
 * frame's, like every other link, and it is free. Nothing here opens one: a nested page is a press
 * from the page, and the one listed state that opens on one says so through `ViewDraw.nested`,
 * counted.
 */
export type Nest = {
	/** The address segment — `note` in `#notes/note/n-groceries`. */
	kind: string;
	/** What the record is called, or `null` for one this page cannot open. */
	name: (id: string) => string | null;
	/**
	 * The nested page's one line of what it is for. A function where the line depends on the
	 * record — adding a note is not changing one.
	 */
	lede: string | ((id: string) => string);
	/** The nested page's body. */
	page: (id: string) => ReactNode;
};

/** What a detour's row prints, read live from whatever store the page keeps. */
export type DetourRead = { fact: string; cells: DetourCell[] };

/** A detour, declared. `get` returns the same object until something it reads has changed. */
export type Detour = {
	subscribe: (listener: () => void) => () => void;
	get: () => DetourRead;
	/** Runs as the phone arrives on the ground. */
	onEnter?: () => void;
};

/**
 * One capability a hold offers from the chrome — `Discard` on a draft, `Stop` on a run.
 *
 * The press is not declared here: every capability in this design ends the hold where it stands,
 * and the frame is what owns that. A place that wired its own would be the one place able to draw
 * a cell that did something else. What the record does about it is `Hold.onEnd`, told which cell.
 */
export type HoldCell = {
	id: string;
	icon: LucideIcon;
	label: string;
	/**
	 * Whether this cell raises the confirm before it ends the hold, and **the rule is one line: a
	 * cell that loses work asks; a cell that keeps it never does.** A draft's discard and a run's
	 * stop ask; a save does not. It is per cell because a hold carries both kinds, and there is no
	 * hold-wide switch: a hold that set one on a save would ask before keeping work.
	 */
	asks?: boolean;
};

/** What a place is holding, and the positions of the frame that only a hold can reach. */
export type Hold = {
	/** What the held record is called — read off the record, never typed twice. */
	name: string;
	/** The one fact about the hold, already in words: `3 unsaved`, `2m 14s`. */
	fact: string;
	/** How far through, 0–1, where the hold is a measured one. A draft has no measure. */
	share?: number;
	/**
	 * What a hand can do about it from the chrome, in the order the cells sit. The slug carries
	 * them on both faces and the page gives them up below `lg`.
	 *
	 * There is no glyph here: the row's way back to the page wears **the destination's own**, which
	 * the frame reads off the place. A glyph a page chose for itself would be one more glyph meaning
	 * *this page*.
	 */
	cells: HoldCell[];
	/**
	 * Whether the hold's own run is going — the record is being written, the run is running.
	 *
	 * The chrome draws it in `kit/Button`'s own `BUSY` vocabulary: the ground empties to `ui-beat`
	 * and the ink stays at the top of the ramp. Undefined default.
	 */
	busy?: boolean;
	/** What the confirm says — the hold's own words, from wherever the errand keeps its words. */
	words: HoldWords;
	/**
	 * **Whether the hold is the nested page's own record**: a note being added or changed inside
	 * Notes. Then the way back to the page loses the work, so it asks like every other way out — the
	 * step's glyph from `lg`, the level row's below it, and the page's own rail cell. A hold that
	 * stands over a nested page, the page's own rather than the record's, leaves the way back free.
	 * Undefined default.
	 */
	nested?: boolean;
	/**
	 * **The hold reads only the record the page has open**: the record a page is editing is read on
	 * the sheet — a press there could change it under the work — and every other record stays
	 * worked. Undefined, the whole sheet is read.
	 */
	reads?: 'record';
	/**
	 * Runs when the hold ends by any press — a cell, the confirm, a way out. `cell` is the id of the
	 * cell that ended it, directly or through the confirm it raised; `null` for a way out or a stop
	 * from the page.
	 */
	onEnd?: (cell: string | null) => void;
};

/** The confirm's words for one kind of hold. */
export type HoldWords = {
	/** The primary: the outcome that loses nothing — `Keep editing`, `Keep running`. */
	stay: string;
	/** Leaving for somewhere else — `Discard and go`, `Stop and go`. */
	leave: string;
	/** Ending it where it stands — `Stop running`. */
	stop: string;
	/** The red line: what is lost, given where the press was heading (`null` — nowhere). */
	loss: (going: string | null) => string;
};

/**
 * A value the frame follows while it changes, in the shape `useSyncExternalStore` takes. `get`
 * returns the same value until something it reads has changed.
 */
export type Live<T> = {
	subscribe: (listener: () => void) => () => void;
	get: () => T;
};

/** A hold that changes while you look at it — a run ticking. `get` is `null` while nothing is held. */
export type HoldSource = {
	subscribe: (listener: () => void) => () => void;
	get: () => Hold | null;
};
