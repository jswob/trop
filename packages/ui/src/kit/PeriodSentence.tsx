import { TextButton } from '@app/ui/kit/Button';
import { MonthPicker } from '@app/ui/kit/MonthPicker';
import { OpenThread } from '@app/ui/kit/OpenRows';
import { Plus } from 'lucide-react';
import { useId, useRef, useState } from 'react';
import { CONTROL_H } from '@app/ui/kit/control';
import { monthPage, monthShift } from '@app/ui/kit/month';
import type { BookedOn } from '@app/ui/kit/month';
import { endWords, inPeriod, periodWords, setEnd, sortPeriods } from '@app/ui/kit/period';
import type { Period, PeriodEdge } from '@app/ui/kit/period';

/**
 * **A list of periods, stated as the sentence that is also its control.**
 *
 * ```
 * ┌  From [Mar 1] until [Mar 31], and from May 1 until       ← the statement at the name's x
 * │  no end, or + add a period                                  a date ui-ink, a word ui-ink-2
 * │  ‹ March 2026 ›                                          ← kit/MonthPicker at the full measure
 * │   M  T  W  T  F  S  S                                      this period's run in ui-ink, the
 * │  ─────────────────────────────────────────────              others' in ui-ink-3
 * └  No end                              Remove this period  ← the pressed end's two presses
 * ```
 *
 * - **The statement is the control**, so the words that describe the range are never quieter than
 *   controls beside them that change it. Every period is a phrase, `From … until …`, the next
 *   joined by `and from`, the last closed by `or + add a period`; each end is its own press. A date
 *   is `ui-ink`, an open end (`no start`, `no end`) and every word between them `ui-ink-2`, all at
 *   weight 400 and `text-ui-body` on a 26px line that wraps wherever the measure ends. The first
 *   line is centred on the control floor, so it reads as the line under the name.
 * - **A pressed end is `TextButton`'s `held`** — the control's own `ui-beat` ground under `ui-ink`,
 *   never the ink plate: what a row opens is never louder than the page.
 * - **The grid opens under the whole statement**, 8px under it, at the full measure, opened on the
 *   pressed end's own month. A day sets **the end that was pressed**; a day past the other end pulls
 *   that end to meet it (`setEnd`). The subject's other periods are the grid's `also` run.
 * - **The pressed end's two presses stand under the grid**, over a `ui-rule`: the open word (`No start`
 *   / `No end`, a toggle — `held` and `aria-pressed` while that end is open) and `Remove this period`
 *   at the row's end.
 * - **`OpenThread`** runs from the statement to the grid's foot while an end is open, so the grid
 *   says which row it belongs to.
 * - **`+ add a period`** adds one from today with no end, opened on its start — or opens the one
 *   already there. With no periods at all the statement is `Any time, or + add a period`.
 *
 * **The keyboard stays where the hand is.** A period is keyed by its place and the grid by the
 * press that opened it, so a day that re-sorts the period re-renders both rather than replacing
 * them; where a press takes its own control away — a period removed, the grid shut — the keyboard
 * goes to the end that held it, to the next period's start, or to `add a period`. `Escape` anywhere
 * in the block shuts the grid and hands the keyboard back to its end.
 *
 * It stands where every list here puts a row's own content — after a marker column and `gap-2`, at
 * the name's x — because that is what `OpenThread`'s fixed offsets assume.
 */

const ADD_PERIOD = 'add a period';
const REMOVE_PERIOD = 'Remove this period';
const ANY_TIME = 'Any time,';

/**
 * Which end of which period holds the grid open, by its place in the sorted list. `turn` counts the
 * presses that opened one, so a day that re-sorts the period keeps the grid mounted under the hand.
 */
export type PeriodOpen = { index: number; edge: PeriodEdge; turn: number };

/** The month an end opens on: its own day, else the other end's, else the month of `today`. */
function monthOf(period: Period, edge: PeriodEdge, today: string): { year: number; month: number } {
	const iso = period[edge] ?? period[edge === 'from' ? 'to' : 'from'] ?? today;
	const [year = '2026', month = '1'] = iso.split('-');
	return { year: Number(year), month: Number(month) };
}

/** A word of the statement that is never a press. */
function Word({ children }: { children: string }) {
	return <span className="text-ui-ink-2">{children}</span>;
}

/** **The end being changed**: the grid, then the end's two presses on one row under it. */
function EndEditor({
	id,
	subject,
	period,
	edge,
	others,
	today,
	booked,
	onSet,
	onRemove,
}: {
	id: string;
	subject: string;
	period: Period;
	edge: PeriodEdge;
	others: readonly Period[];
	today: string;
	booked?: BookedOn;
	onSet: (iso: string | null) => void;
	onRemove: () => void;
}) {
	const [page, setPage] = useState(() => monthOf(period, edge, today));
	const open = period[edge] === null;
	const openWord = edge === 'from' ? 'No start' : 'No end';

	const grid = monthPage(
		page.year,
		page.month,
		{ start: period.from ?? '0000-01-01', end: period.to ?? '9999-12-31' },
		{ today, booked, also: iso => others.some(other => inPeriod(other, iso)) }
	);

	return (
		<div
			id={id}
			role="group"
			aria-label={`When ${subject} ${edge === 'from' ? 'starts' : 'ends'}: ${periodWords(period)}`}
			className="pt-2"
		>
			<MonthPicker
				page={grid}
				onStep={by => setPage(at => monthShift(at.year, at.month, by))}
				onPick={iso => onSet(iso)}
			/>
			<div
				className={`mt-2 flex ${CONTROL_H} items-center justify-between gap-3 border-t border-ui-rule`}
			>
				<TextButton
					inline
					held={open}
					pressed={open}
					className="py-1"
					label={`${openWord} for ${subject}, ${periodWords(period)}`}
					onClick={() => onSet(null)}
				>
					{openWord}
				</TextButton>
				<TextButton
					inline
					edge="end"
					className="py-1"
					label={`${REMOVE_PERIOD}: ${subject}, ${periodWords(period)}`}
					onClick={onRemove}
				>
					{REMOVE_PERIOD}
				</TextButton>
			</div>
		</div>
	);
}

export function PeriodSentence({
	id,
	subject,
	periods,
	today,
	booked,
	opened = null,
	onChange,
}: {
	/** What the press that opened the sentence points `aria-controls` at. */
	id: string;
	/** Whose periods these are, for a reader: `Ada`. */
	subject: string;
	/** In date order (`sortPeriods`). */
	periods: readonly Period[];
	/** The day to treat as today: where a new period starts and a grid with no day opens. */
	today: string;
	/** What a day already holds, named by the grid. */
	booked?: BookedOn;
	/** The end it opens on, for a state a stage seeds rather than presses to. */
	opened?: PeriodOpen | null;
	onChange: (next: Period[]) => void;
}) {
	const [editing, setEditing] = useState<PeriodOpen | null>(opened);
	const editorId = useId();
	const block = useRef<HTMLDivElement>(null);
	const turns = useRef(opened?.turn ?? 0);

	/** Puts the keyboard on an end, or on `add a period`, once the press has drawn. */
	const focusOn = (target: string) =>
		requestAnimationFrame(() => {
			block.current?.querySelector<HTMLElement>(`[data-focus="${target}"] button`)?.focus();
		});

	/** Opens one end's grid: a new turn, so it opens on that end's own month. */
	const openOn = (index: number, edge: PeriodEdge) => {
		turns.current += 1;
		setEditing({ index, edge, turn: turns.current });
	};

	const held = editing === null ? undefined : periods[editing.index];

	const end = (period: Period, index: number, edge: PeriodEdge) => {
		const pressed = editing?.index === index && editing.edge === edge;
		return (
			<span data-focus={`${String(index)}-${edge}`} className="contents">
				<TextButton
					inline
					ink={period[edge] === null ? 'ink-2' : 'ink'}
					held={pressed}
					label={`${edge === 'from' ? 'Change when it starts' : 'Change when it ends'}: ${subject}, ${periodWords(period)}`}
					expanded={pressed}
					controls={pressed ? editorId : undefined}
					onClick={() => (pressed ? setEditing(null) : openOn(index, edge))}
				>
					{endWords(period, edge)}
				</TextButton>
			</span>
		);
	};

	const add = () => {
		const there = periods.findIndex(each => each.from === today && each.to === null);
		if (there !== -1) {
			openOn(there, 'from');
			focusOn(`${String(there)}-from`);
			return;
		}
		const made: Period = { from: today, to: null };
		const next = sortPeriods([...periods, made]);
		onChange(next);
		openOn(next.indexOf(made), 'from');
		focusOn(`${String(next.indexOf(made))}-from`);
	};

	return (
		<div
			id={id}
			ref={block}
			className="pb-1"
			onKeyDown={event => {
				if (event.key !== 'Escape' || editing === null) return;
				event.stopPropagation();
				setEditing(null);
				focusOn(`${String(editing.index)}-${editing.edge}`);
			}}
		>
			<OpenThread drawn={editing !== null && held !== undefined}>
				<div className="flex flex-wrap items-center gap-x-1.5 py-[7px] text-ui-body leading-[26px] lg:py-[3px]">
					{periods.length === 0 ? (
						<span className="text-ui-ink">{ANY_TIME}</span>
					) : (
						periods.map((period, index) => (
							<span key={index} className="contents">
								<Word>{index === 0 ? 'From' : 'and from'}</Word>
								{end(period, index, 'from')}
								<Word>until</Word>
								<span className="inline-flex items-center">
									{end(period, index, 'to')}
									<Word>,</Word>
								</span>
							</span>
						))
					)}
					<Word>or</Word>
					<span data-focus="add" className="contents">
						<TextButton
							inline
							className="inline-flex items-center gap-1"
							label={`Add a period to ${subject}`}
							onClick={add}
						>
							<Plus aria-hidden size={12} />
							{ADD_PERIOD}
						</TextButton>
					</span>
				</div>

				{editing !== null && held !== undefined ? (
					<EndEditor
						key={editing.turn}
						id={editorId}
						subject={subject}
						period={held}
						edge={editing.edge}
						others={periods.filter((_period, at) => at !== editing.index)}
						today={today}
						booked={booked}
						onSet={iso => {
							const set = setEnd(held, editing.edge, iso);
							const next = sortPeriods(
								periods.map((each, at) => (at === editing.index ? set : each))
							);
							onChange(next);
							setEditing({ ...editing, index: next.indexOf(set) });
						}}
						onRemove={() => {
							const at = editing.index;
							onChange(periods.filter((_period, each) => each !== at));
							setEditing(null);
							focusOn(at < periods.length - 1 ? `${String(at)}-from` : 'add');
						}}
					/>
				) : null}
			</OpenThread>
		</div>
	);
}
