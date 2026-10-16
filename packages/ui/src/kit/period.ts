import { formatDays } from '@app/ui/kit/month';

/**
 * **A period: a run of days with either end open**. `null` at `from` is no start, at `to`
 * no end; both `null` is every day. ISO days, inclusive at both ends.
 *
 * The words and the arithmetic `kit/PeriodSentence` states a list of periods with, on their own so a
 * caller can print the same words where the sentence is shut.
 */
export type Period = { from: string | null; to: string | null };

/** Which end of a period. */
export type PeriodEdge = 'from' | 'to';

/** One ISO day in the ledger's own words: `Mar 1`. */
export function periodDay(iso: string): string {
	return formatDays(iso, iso);
}

/** One end of a period in words: its day, or `no start` / `no end` where it is open. */
export function endWords(period: Period, edge: PeriodEdge): string {
	const day = period[edge];
	if (day !== null) return periodDay(day);
	return edge === 'from' ? 'no start' : 'no end';
}

/**
 * **One period with both ends always said**, an open one in words: `Mar 1 – Mar 31`
 * · `May 1 – no end` · `no start – Apr 30`. What a reader hears a press is about.
 */
export function periodWords(period: Period): string {
	return `${endWords(period, 'from')} – ${endWords(period, 'to')}`;
}

/**
 * **One named end set to a day** — the end being changed is the one that was pressed, never the
 * nearer one. An end set past the other pulls the other to meet it, so the period is that one
 * day rather than running backwards; `null` opens the end.
 */
export function setEnd(period: Period, edge: PeriodEdge, iso: string | null): Period {
	if (edge === 'from') {
		const to = iso !== null && period.to !== null && period.to < iso ? iso : period.to;
		return { from: iso, to };
	}
	const from = iso !== null && period.from !== null && period.from > iso ? iso : period.from;
	return { from, to: iso };
}

/** Periods in date order, an open start first — the order the sentence reads them in. */
export function sortPeriods<P extends Period>(periods: readonly P[]): P[] {
	return [...periods].sort((a, b) => (a.from ?? '').localeCompare(b.from ?? ''));
}

/** Whether a day falls inside a period, open ends running on for ever. */
export function inPeriod(period: Period, iso: string): boolean {
	return (period.from === null || iso >= period.from) && (period.to === null || iso <= period.to);
}
