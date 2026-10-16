/**
 * A month as whole weeks of days, and what each day is.
 *
 * This is the arithmetic behind `MonthPicker`. It lives in the kit rather than beside any
 * one screen because the picker is a kit component now: whatever needs a day range asks
 * for the same month, worked out the same way.
 *
 * It knows nothing about what the days are for. What a day already carries is supplied by the
 * caller as a lookup, so a picker in a different section can hand it a different world.
 */

export const WEEKDAYS = ['Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa', 'Su'];

export const MONTHS = [
	'January',
	'February',
	'March',
	'April',
	'May',
	'June',
	'July',
	'August',
	'September',
	'October',
	'November',
	'December',
];

/** The three letters a date is printed with: `Mar`. */
export const SHORT_MONTHS = MONTHS.map(name => name.slice(0, 3));

/**
 * A run of ISO days, inclusive, in words: `Mar 8` · `Apr 12 – 14` · `Apr 30 – May 2`. The month
 * repeats only when it changes.
 */
export function formatDays(startDay: string, endDay: string): string {
	const [, startMonth = 1, startDate = 1] = startDay.split('-').map(Number);
	const [, endMonth = 1, endDate = 1] = endDay.split('-').map(Number);
	const start = `${SHORT_MONTHS[startMonth - 1] ?? ''} ${String(startDate)}`;
	if (startDay === endDay) return start;
	if (startMonth === endMonth) return `${start} – ${String(endDate)}`;
	return `${start} – ${SHORT_MONTHS[endMonth - 1] ?? ''} ${String(endDate)}`;
}

function pad(value: number): string {
	return value < 10 ? `0${String(value)}` : String(value);
}

/** A day, as every date in this system is spelled. */
export function isoDay(year: number, month: number, day: number): string {
	return `${String(year)}-${pad(month)}-${pad(day)}`;
}

/** Where a day falls in the picked run. `only` is a one-day run: both ends at once. */
export type PickState = 'start' | 'mid' | 'end' | 'only' | null;

/**
 * One cell of the month.
 *
 * Everything a day can be is worked out here rather than in the drawing of it, so the
 * picker only ever decides how a state looks and never which days are in it.
 */
export type MonthCell = {
	iso: string;
	date: number;
	month: number;
	/** A spill day from the month either side. Drawn, so the weeks are whole weeks. */
	outside: boolean;
	/** Behind today. Drawn fainter and never refused: a run can be recorded after the fact. */
	past: boolean;
	today: boolean;
	/** What the day already carries, or null. Marked, and never reduced to a colour. */
	booked: string | null;
	picked: PickState;
	/**
	 * Inside another run the caller holds but is not picking — a record's other periods. Drawn
	 * as the same full-bleed rule one rung down the ramp, `ui-ink-3`; the picked run beats it. Opt-in
	 * through `MonthOptions.also`, and `false` for every caller that gives none.
	 */
	also: boolean;
};

export type MonthPage = { label: string; year: number; month: number; weeks: MonthCell[][] };

/** The range being picked. Both ends null until the first day is chosen. */
export type DayRange = { start: string | null; end: string | null };

/** What the world already holds on a day, asked one day at a time. */
export type BookedOn = (iso: string) => string | null;

export type MonthOptions = {
	/** The day to treat as today. Passed in so a fixture world can name its own. */
	today?: string;
	booked?: BookedOn;
	/** Days inside the caller's other runs, drawn a rung fainter than the picked one. */
	also?: (iso: string) => boolean;
};

/** The month `by` months from this one. */
export function monthShift(
	year: number,
	month: number,
	by: number
): { year: number; month: number } {
	const index = year * 12 + (month - 1) + by;
	return { year: Math.floor(index / 12), month: (index % 12) + 1 };
}

/** What a month is called. */
export function monthLabel(year: number, month: number): string {
	return `${MONTHS[month - 1] ?? ''} ${String(year)}`;
}

/** The month before and the month after, as the header's two controls say them. */
export function monthStep(page: MonthPage, by: number): string {
	const { year, month } = monthShift(page.year, page.month, by);
	return monthLabel(year, month);
}

/** Which month a range opens on: the one it starts in, or the one we are in. */
export function rangeMonth(range: DayRange, today: string): { year: number; month: number } {
	const iso = range.start ?? today;
	const [year = '1970', month = '1'] = iso.split('-');
	return { year: Number(year), month: Number(month) };
}

/**
 * A month as whole weeks of cells, Monday first, spilled rather than padded.
 *
 * The spill days are real days — pressable, and able to be the day you pick — because a
 * run across the end of a month is an ordinary run, and a grid whose last row is half blank
 * makes it look like one that cannot be.
 */
export function monthPage(
	year: number,
	month: number,
	range: DayRange,
	options: MonthOptions = {}
): MonthPage {
	const { today = '', booked, also } = options;
	const first = new Date(Date.UTC(year, month - 1, 1));
	const lead = (first.getUTCDay() + 6) % 7;
	const cursor = new Date(Date.UTC(year, month - 1, 1 - lead));

	const { start, end } = range;
	const weeks: MonthCell[][] = [];
	const after = new Date(Date.UTC(year, month, 1)).getTime();

	while (cursor.getTime() < after) {
		const week: MonthCell[] = [];
		for (let column = 0; column < 7; column += 1) {
			const cellMonth = cursor.getUTCMonth() + 1;
			const date = cursor.getUTCDate();
			const iso = isoDay(cursor.getUTCFullYear(), cellMonth, date);

			const picked: PickState =
				start === null || end === null || iso < start || iso > end
					? null
					: start === end
						? 'only'
						: iso === start
							? 'start'
							: iso === end
								? 'end'
								: 'mid';

			week.push({
				iso,
				date,
				month: cellMonth,
				outside: cellMonth !== month,
				past: today !== '' && iso < today,
				today: iso === today,
				booked: booked?.(iso) ?? null,
				picked,
				also: also?.(iso) ?? false,
			});

			cursor.setUTCDate(date + 1);
		}
		weeks.push(week);
	}

	return { label: monthLabel(year, month), year, month, weeks };
}

/** The accessible name a day carries, wherever it is drawn. */
export function dayName(cell: MonthCell): string {
	const parts = [String(cell.date)];
	if (cell.today) parts.push('today');
	if (cell.booked !== null) parts.push(cell.booked);
	if (cell.picked === 'start') parts.push('first day');
	if (cell.picked === 'end') parts.push('last day');
	if (cell.picked === 'only') parts.push('the day');
	if (cell.picked === 'mid') parts.push('picked');
	if (cell.also && cell.picked === null) parts.push('in another period');
	return parts.join(', ');
}
