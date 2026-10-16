import type { DestId } from '@app/ui/views/destinations';
import type { ViewDraw } from '@app/ui/views/frame-types';
import type { Subject } from '@jswob/design-workbench';

export type {
	Detour,
	DetourRead,
	Hold,
	HoldCell,
	HoldSource,
	HoldWords,
	Live,
	Nest,
	PageErrand,
	SignedOut,
	ViewDraw,
} from '@app/ui/views/frame-types';

/**
 * A view is a **place you can be**: the ground, or one destination over it.
 *
 * The frame is the **ground**, which is the app's home, and the **destinations**, which are pages
 * opened beside it. There is no second rail to be at the crossing of, so a place is a destination,
 * or it is the ground. A place is the smallest thing a person can arrive at and the largest thing
 * that can be judged in one frame. A destination with nothing drawn in it is not a place yet: it
 * stays a cell in the rail, because the rail is what the app is, and it becomes a place in the round
 * that draws it.
 *
 * **A place never draws chrome.** It declares a page, a head, what is in the ledger and what the
 * slug should print; `@app/ui/views/frame` supplies the split, the rail, the scroll, the page
 * margins and the slug. That is mechanical, not a convention: a place file has no import of
 * `SplitShell` and `views/frame.test` asserts it never gains one. A view that owned its own frame
 * could disagree with every other view about where you are.
 *
 * **A variant is a state of the app, not a screen size.** Every state draws at every width — the
 * split above `lg` and the slug frame below it, from one seed — so `?width=mobile` is a control on
 * the workbench rather than a variant of the place.
 *
 * **A variant is a state of the app, not a press.** The frame is live: the logo collapses the
 * ledger, the slug raises the register, a status row peeks and a guarded cell raises the confirm.
 * So **two entries that differ only in what a hand owns are one entry** — the second is a
 * screenshot of something you could go and do. A variant declares what the app has *open*; the hand
 * declares what has been *done* to it.
 *
 * This is why `ViewDraw` and `Hold` carry no field for a hand's position: nothing seeds the ledger
 * collapsed, a peek, a guarded exit or a scroll. The exception is `menu`, and it is counted — see
 * the field.
 */

/**
 * One state of a place, and **the test for whether it is one**.
 *
 * Before adding an entry here, ask what a hand would have to do to reach this screen from an entry
 * that already exists. If the answer is a press, a scroll or a tap — it is not a variant. Open the
 * place in the workbench and do it.
 *
 * What earns an entry is a state the frame cannot be driven into:
 *
 * - a **different record** — empty, populated, the longest one production has;
 * - a **different mode of the record** — being created, being changed;
 * - a **different condition on it** — holding unsaved changes, still loading, a read that failed.
 *
 * What does not, with the press that reaches it instead: the ledger collapsed (the logo), a note's
 * delete armed (its own `Delete`), the page scrolled (the scroller), a peek or a guarded exit (the
 * status row, any rail cell).
 */
export type PlaceVariant = {
	/** URL-safe; it is what `?variants=` carries. */
	id: string;
	label: string;
	/**
	 * Why this state is listed: the record or the condition no press produces. One line over its
	 * frame, so at most `NOTE_CAP` characters. A note that describes a press is the tell that the
	 * entry should not exist.
	 */
	note: string;
	draw: () => ViewDraw;
};

/**
 * One place, declared. `views/place-frame` turns it into the `Subject` the stage renders.
 *
 * **A place is one errand, not one destination.** A destination that is two errands — adding a
 * record and changing one, each with its own head — is two places, so neither carries a grid of
 * states about a record the other is about. The **ground is still exactly one**, and
 * `views/frame.test` holds that.
 */
export type Place = {
	/** URL-safe; it is the `$view` path param. */
	id: string;
	/**
	 * Which destination this place is, or `null` for the **ground** — the app's home, which is a
	 * destination in its own right on a phone and the other half of the screen above `lg`.
	 *
	 * Not unique: one destination is one or more errands, and each errand is a place.
	 */
	dest: DestId | null;
	title: string;
	/** What the place is, on one line beside its name: at most `DESCRIPTION_CAP` characters. */
	description: string;
	variants: PlaceVariant[];
};

/** A place, framed — what `/views/$view` and the frame route actually render. */
export type View = Subject & Pick<Place, 'dest'>;
