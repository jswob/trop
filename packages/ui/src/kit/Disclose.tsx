import { ChevronDown, ChevronUp } from 'lucide-react';

/**
 * **The mark that says this opens** — a chevron, down when the thing is shut and up when it is
 * standing open.
 *
 * *A chevron marks what opens, and nothing else*: not a read-only fact, not a row that merely has
 * more to say. It is one component so that everything that opens draws the same two glyphs, at the
 * same size and the same rung of the ink ramp, rather than copying them out of `kit/Select` or
 * going unmarked — and a control that opens something without saying so leaves the reader
 * guessing what a press will do.
 *
 * 14px, `ui-ink-3`, and `aria-hidden` throughout — **the glyph is never the announcement**. What
 * tells a screen reader is `aria-expanded` on the control itself, which is why `kit/Button`'s
 * `TextButton` takes it.
 *
 * It draws the state, and it is never itself the target: a caller puts it inside the button whose
 * press flips it, and `className` carries only where it sits in that button's own line.
 */
export function Disclose({ open, className = '' }: { open: boolean; className?: string }) {
	const Glyph = open ? ChevronUp : ChevronDown;

	return <Glyph aria-hidden className={`h-3.5 w-3.5 shrink-0 text-ui-ink-3 ${className}`} />;
}
