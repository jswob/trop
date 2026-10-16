import { ICON_BOX, IconButton, TextButton } from '@app/ui/kit/Button';
import { Minus } from 'lucide-react';
import { useRef, useState } from 'react';
import type { RefObject } from 'react';

/**
 * **A row a press takes away, in two steps** — the marker column's `−`, and the two words that
 * answer it.
 *
 * ```
 *  −   Groceries                          any time     ← at rest: the marker is a press
 *  −   Groceries                Remove  Keep it        ← armed: ui-red marker, the row's words
 *      From Mar 1 until …                                ui-red, the end holds the two answers
 * ```
 *
 * Nothing is ever taken on one press: a row that deletes at once leaves no step to confirm it. The
 * first press **arms**: the marker stops being a control and turns `ui-red`, and the caller turns
 * its own words `ui-red` and says what goes. The end then holds the answer — the verb in `ui-ink`,
 * then the word that keeps it.
 *
 * The marker, the two words and the red live here so that no caller carries them by hand. The
 * words are the caller's: the kit knows nothing about what is being removed.
 */

/** The marker: a quiet `−` press at rest, the same square in `ui-red` and inert once armed. */
export function ArmMarker({
	armed,
	label,
	onArm,
}: {
	armed: boolean;
	/** What the press takes away, for a reader: `Take Groceries away`. */
	label: string;
	onArm: () => void;
}) {
	return armed ? (
		<span aria-hidden className={`${ICON_BOX} text-ui-red`}>
			<Minus size={16} />
		</span>
	) : (
		<IconButton label={label} onClick={onArm}>
			<Minus size={16} />
		</IconButton>
	);
}

/**
 * The two answers an armed row holds at its end: the verb that takes it (`ui-ink`), then the word that
 * keeps it. Each accessible name says what it acts on — `Remove Groceries`, `Keep it — Groceries` —
 * because two words alone do not say which row they answer.
 */
export function ArmEnds({
	named,
	remove,
	keep,
	onRemove,
	onKeep,
}: {
	/** What the row is called, for a reader. */
	named: string;
	/** The verb that takes it: `Remove`. */
	remove: string;
	/** The word that keeps it: `Keep it`. */
	keep: string;
	onRemove: () => void;
	onKeep: () => void;
}) {
	return (
		<>
			<TextButton ink="ink" label={`${remove} ${named}`} onClick={onRemove}>
				{remove}
			</TextButton>
			<TextButton edge="end" label={`${keep} — ${named}`} onClick={onKeep}>
				{keep}
			</TextButton>
		</>
	);
}

/**
 * **Where the keyboard goes when a row is armed or kept.** Arming takes the pressed marker away and
 * keeping takes the pressed word away, so each hands the keyboard to the control that now answers:
 * armed, to `keep`; kept, back to the marker, found by the start of its accessible name.
 */
export function useArm({ keep, marker }: { keep: string; marker: string }): {
	armed: boolean;
	arm: (next: boolean) => void;
	row: RefObject<HTMLDivElement | null>;
} {
	const [armed, setArmed] = useState(false);
	const row = useRef<HTMLDivElement>(null);
	const arm = (next: boolean) => {
		setArmed(next);
		requestAnimationFrame(() => {
			row.current
				?.querySelector<HTMLElement>(next ? `[aria-label^="${keep}"]` : `[aria-label^="${marker}"]`)
				?.focus();
		});
	};
	return { armed, arm, row };
}
