import { TextButton } from '@app/ui/kit/Button';
import { Plus } from 'lucide-react';
import type { ReactNode } from 'react';

import { PROSE } from '@app/ui/kit/prose';

/**
 * **What a row of a list opens is more rows, in the list's own row grid** — and `OpenThread`, the
 * one mark that says which row an open block belongs to, which `kit/PeriodSentence` shares.
 *
 * ```
 * ──────────────────────────────────────────────────── ui-rule, over every row
 * Remind me a day before each errand        Switch off   ← ActRow: the words, then the verb
 * ────────────────────────────────────────────────────
 * Mark a note done once it is ticked        Switch on    ← a copy switched off, struck at ui-ink-2
 * ──────────────────────────────────────────────────── ← RowGap: 12px, closed by its hairline
 *
 * ────────────────────────────────────────────────────
 * Keep groceries at the top                 Rewrite
 * ────────────────────────────────────────────────────
 * + Write a reminder                                     ← AddRow: + and what it adds
 * ──────────────────────────────────────────────────── the block's hairline under its last row
 * ```
 *
 * - **A row** is the control floor at least — 32px from `lg`, 40 below — with a `ui-rule` hairline
 *   over it; the block draws one under its last row.
 * - **The words are never the press**: a resting row carries no control on any of its words. The
 *   one word at the row's end is, and it says what the press does; the accessible name says the
 *   verb and the words together.
 * - **The block stands at the name's x at every width** — one layout at heart, with no step out of
 *   the indent below `md` — and `threaded` draws `OpenThread` beside it.
 */

/** One row's box: the control floor, the hairline over it, and a word column at its end. */
export const OPEN_ROW = `grid min-h-ui-touch grid-cols-[minmax(0,1fr)_auto] items-center gap-x-3 border-t border-ui-rule lg:min-h-ui-control`;

/**
 * **The thread: what a row opens, tied back to the row.** Without it nothing says which row an
 * open block or an open date grid belongs to. A 1px `ui-ink-3` bracket down the centre of the
 * marker column, from the centre of the block's first line to its foot, a tick at each end reaching
 * to 6px short of the name's x. It says which row an open block belongs to without a second copy of
 * the row's words, and it is never louder than the page: one hairline in the lowest ink that still
 * reads.
 *
 * It measures nothing. It assumes what every row of a list here is: a marker column at the control's
 * width, `gap-2`, then the name — so the block it wraps stands at the name's x (48px below `lg`, 40
 * from it) and its first line is centred on the control floor. That is why the offsets are fixed:
 * `-left-7` / `-left-6` is the marker's centre, `top-5` / `top-4` the first line's.
 *
 * `drawn` is the caller's: `kit/PeriodSentence` draws it only while an end's grid is open,
 * `OpenBlock` whenever it is `threaded`.
 */
export function OpenThread({ drawn, children }: { drawn: boolean; children: ReactNode }) {
	return (
		<div className="relative">
			{children}
			{drawn ? (
				<span
					aria-hidden
					className="pointer-events-none absolute top-5 bottom-0 -left-7 w-[22px] border-y border-l border-ui-ink-3 lg:top-4 lg:-left-6 lg:w-[18px]"
				/>
			) : null}
		</div>
	);
}

/**
 * **The open block**: its rows, and the hairline under the last. Each row brings the hairline over
 * itself. `id` is what the press that opened it points `aria-controls` at. It stands at the name's x
 * at every width, and `threaded` ties it to its row.
 */
export function OpenBlock({
	id,
	threaded = false,
	children,
}: {
	id: string;
	/** Draw `OpenThread` beside it, from its first row to its hairline. */
	threaded?: boolean;
	children: ReactNode;
}) {
	return (
		<div id={id} className="pt-2 pb-1">
			<OpenThread drawn={threaded}>
				<div className="border-b border-ui-rule">{children}</div>
			</OpenThread>
		</div>
	);
}

/**
 * **A row that says what its press does**: typeset words, then the one word at the end that is the
 * press.
 */
export function ActRow({
	children,
	verb,
	label,
	onPress,
}: {
	children: ReactNode;
	/** The press's one word: `Switch off`, `Rewrite`. */
	verb: string;
	/** The verb and the words together, for a reader. */
	label: string;
	onPress: () => void;
}) {
	return (
		<div className={OPEN_ROW}>
			<div className={`${PROSE} min-w-0 py-1.5`}>{children}</div>
			<TextButton edge="end" label={label} onClick={onPress}>
				{verb}
			</TextButton>
		</div>
	);
}

/** The 12px between two groups of one block, closed by the hairline of the group above it. */
export function RowGap() {
	return <div aria-hidden className="h-3 border-t border-ui-rule" />;
}

/** The block's last row: `+` and what it adds, at the words' x. */
export function AddRow({ word, onPress }: { word: string; onPress: () => void }) {
	return (
		<div className="border-t border-ui-rule">
			<TextButton edge="both" className="w-full" onClick={onPress}>
				<span className="flex items-center gap-2">
					<Plus aria-hidden size={14} />
					<span className="min-w-0 truncate">{word}</span>
				</span>
			</TextButton>
		</div>
	);
}
