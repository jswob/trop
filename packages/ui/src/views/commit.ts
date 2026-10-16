/**
 * **Whether a record can be saved, what it owes, and the words and cells of its hold** — the
 * record-free half of committing, shared by every page that edits a record. What reads a particular
 * record's fields stays with that record's page and builds on this.
 *
 * **Nothing here decides how any of it looks.** A page's foot and `kit/RailShell`'s hold row both
 * take their words and their conditions from here: one enum with two drawings stays one enum only
 * while nobody writes the mapping a second time.
 */

/**
 * The four conditions the record can be in.
 *
 * They are conditions on the **record**, not presses: no control on screen turns a clean document
 * into a dirty one, so each earns a frame of its own. `saving` is the fifth thing a button knows how
 * to be (`Button`'s `running`) and it is here because a foot that cannot draw its own run is a foot
 * that will grow a spinner later.
 */
export type Commit = 'clean' | 'ready' | 'blocked' | 'saving';

/** Whether the record differs from what is stored. `clean` is the only one that does not. */
export function isDirty(commit: Commit): boolean {
	return commit !== 'clean';
}

/** Whether the errand could be finished right now. */
export function isSavable(commit: Commit): boolean {
	return commit === 'ready';
}

/* ------------------------------------------------------------------------- *
 * What a record owes
 * ------------------------------------------------------------------------- */

/**
 * One thing a section still wants, named by the section that wants it.
 *
 * **Both halves matter and neither is optional.** `section` is what the reader has to go and open —
 * the band's own title, verbatim, so the sentence and the page agree on what a place is called.
 * `wants` is what to do when they get there, written as a noun phrase that reads after *Needs*.
 */
export type Owed = { section: string; wants: string };

/**
 * The report, as one sentence — **built from whatever the record still owes.**
 *
 * `Needs a name and an item on the checklist.` It is a sentence and not a bulleted list because
 * everything the app already knows is typeset text, and what the document owes is something the app
 * already knows.
 *
 * Two joins and no third: two clauses take `and`, three or more take commas and a final `and`.
 * A section whose name is already in the sentence's own words is not repeated — `an item on the
 * checklist` carries its own address, which is why `wants` is written the way it is.
 */
export function owedSentence(owed: Owed[]): string {
	if (owed.length === 0) return '';

	const wants = owed.map(entry => entry.wants);
	const last = wants[wants.length - 1] ?? '';
	const joined =
		wants.length === 1
			? last
			: `${wants.slice(0, -1).join(', ')}${wants.length > 2 ? ',' : ''} and ${last}`;

	return `Needs ${joined}.`;
}

/**
 * The same fact as a count, for a slot that holds no sentence — the chrome's own 320px box. The
 * count is written out in words, not drawn as a badge.
 */
export function owedCount(owed: Owed[]): string {
	if (owed.length === 0) return '';
	return owed.length === 1 ? '1 section needs work' : `${String(owed.length)} sections need work`;
}

/**
 * How many changes are being held, as the hold's one fact — `3 unsaved`.
 *
 * Written here rather than in a place because the chrome and the document both print it, and one
 * fact said in two places only reads as one if the two agree on the string.
 */
export function changesFact(changes: number): string {
	return `${String(changes)} unsaved`;
}

/* ------------------------------------------------------------------------- *
 * The words
 * ------------------------------------------------------------------------- */

/**
 * **One action, one word.** A screen that renames `Save changes` to `Save` is a second vocabulary
 * nobody asked for, and the disagreement would be invisible on any single screenshot.
 *
 * The verbs are active and say what happens: `Save changes`, not `Submit`; `Discard changes`, not
 * `Cancel`.
 */
export const SAVE = 'Save changes';
export const SAVING = 'Saving';
/** Modify: put the record back the way it was stored. */
export const REVERT = 'Discard changes';
/** Create: there is no record to put back, so the word is about leaving. */
export const ABANDON = 'Discard';
/** Why the finisher is off when nothing has been changed: a disabled control always says why. */
export const NOTHING_CHANGED = 'Nothing changed yet';

/**
 * **Why the leaver cannot be pressed, where it cannot.**
 *
 * The foot's leaver presses the hold's discard cell, and a record nothing has changed on holds
 * nothing. Drawn pressable there — `Discard changes` beside a `Save changes` that is off, or
 * `Discard` on a new record nothing has been typed into — a press would do nothing. Both are off for
 * the reason the finisher already gives, said once.
 */
export function leaveReason(commit: Commit): string | null {
	return isDirty(commit) ? null : NOTHING_CHANGED;
}

/* ------------------------------------------------------------------------- *
 * What the chrome carries
 * ------------------------------------------------------------------------- */

/**
 * **What the hold's row is carrying, per condition — read by both surfaces, written once.**
 *
 * The box builds its cells from this and the page's foot reads the same three booleans to know what
 * not to draw twice. Two conditions, one on each side, agree only until one of them changes, so
 * they are one function.
 */
export type HoldParts = {
	/** The commit, as a cell. Only where the record could actually be saved. */
	commit: boolean;
	/** The run — the row's own fact and its busy ground, never a cell. */
	running: boolean;
	/** Putting it back. Unconditional: every hold can be discarded. */
	discard: boolean;
};

/** The hold's two cells, by id — pressed from the slug and from the page's own foot alike. */
export const COMMIT_CELL = 'commit';
export const DISCARD_CELL = 'discard';

export function holdParts(commit: Commit): HoldParts {
	return {
		commit: isSavable(commit),
		running: commit === 'saving',
		discard: isDirty(commit),
	};
}
