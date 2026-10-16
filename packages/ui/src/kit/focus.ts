/**
 * The focus ring. Two pixels of ink, offset two, on every interactive thing.
 *
 * Note what is absent: `outline-none`. Tailwind's `outline-none` sets
 * `outline-style: none`, which cancels `outline-2` no matter what order the classes
 * are written in — the ring then silently never paints. Do not add it back.
 */
export const FOCUS =
	'focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[var(--color-ui-ink)]';

/**
 * The same ring, drawn by a box for a bare input inside it.
 *
 * A `<label>` cannot take focus itself, so a control whose input sits in a larger box has to ring
 * the box: `FOCUS` on the label never fires, and the input's own ring lands inside the edge instead
 * of around it.
 */
export const FOCUS_WITHIN =
	'focus-within:outline-2 focus-within:outline-offset-2 focus-within:outline-[var(--color-ui-ink)]';

/**
 * A state a pointer or a keyboard would have to produce, drawn on purpose in a variant grid.
 *
 * Hover and focus do not survive a screenshot, so the design skill's `COMPONENT.md` asks for them
 * as forced cells. These
 * are the kit's own answers, written once: the row ground under a pointer, the beat ground under a
 * press, and `FOCUS`'s ring drawn without waiting for the keyboard. A component that forces a state
 * reads them here rather than spelling them again.
 */
export const FORCED_GROUND = 'bg-ui-row';
export const FORCED_PRESS = 'bg-ui-beat';
export const FORCED_RING = 'outline-2 outline-offset-2 outline-[var(--color-ui-ink)]';
