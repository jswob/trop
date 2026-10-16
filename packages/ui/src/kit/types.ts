/** The state of one slot. The whole system's status vocabulary is these four. */
export type SlotState = 'confirmed' | 'pending' | 'proposed' | 'open';

/**
 * The word for each state. Status is always shape plus word — the word is what makes
 * the shape learnable on first sight and what keeps the mark from being the sole
 * carrier. Never tint it.
 */
export const STATE_WORD: Record<SlotState, string> = {
	confirmed: 'confirmed',
	pending: 'pending',
	proposed: 'proposed',
	open: 'open',
};

/**
 * What each state means, said once.
 *
 * People's own names for two of these are looser than the enum: an `open` slot is the one they call
 * **empty**, and a `confirmed` slot is the one they call **accepted** — `accepted` is the verb for
 * the press that produces it, not a fifth state. The enum keeps the stored words; this record keeps
 * the person's.
 *
 * - **open** — nothing is stored for the slot at all.
 * - **proposed** — a suggestion names someone and nothing has been written yet. It exists in the
 *   client, not in storage, which is why refusing one costs nothing.
 * - **pending** — a name *was* written, and the person it names still has to say yes.
 * - **confirmed** — settled, whoever put the name there.
 *
 * Keeping it as data rather than as prose in a comment is what stops the words and their meanings
 * drifting apart.
 */
export const STATE_MEANING: Record<SlotState, string> = {
	confirmed: 'The slot is settled. Nothing is owed on it, and emptying it is the only way back.',
	pending: 'The slot is filled, and the person in it still has to confirm.',
	proposed: 'A suggestion fills this slot. Nothing is written until it is accepted.',
	open: 'Nobody holds the slot. Pick a name from the list to fill it.',
};
