import { describe, expect, it } from 'vitest';

import { dayName, isoDay, monthPage, monthStep, type DayRange } from './month';

const NONE: DayRange = { start: null, end: null };
const RUN: DayRange = { start: '2026-04-05', end: '2026-04-07' };
const ONE: DayRange = { start: '2026-03-08', end: '2026-03-08' };
const TODAY = '2026-03-04';

/** A world with one thing in it, so `booked` is exercised without a fixture. */
function booked(iso: string): string | null {
	return iso >= '2026-04-12' && iso <= '2026-04-14' ? 'Errands' : null;
}

describe('monthPage', () => {
	const april = monthPage(2026, 4, RUN, { today: TODAY, booked });

	it('starts every week on Monday and spills rather than pads', () => {
		// 1 Apr 2026 is a Wednesday, so the first row opens on 30 March.
		expect(april.weeks[0]?.map(cell => cell.date)).toEqual([30, 31, 1, 2, 3, 4, 5]);
		expect(april.weeks[0]?.[0]?.outside).toBe(true);
		expect(april.weeks[0]?.[2]?.outside).toBe(false);
	});

	it('draws whole weeks and no week that holds none of the month', () => {
		for (const month of [1, 2, 3, 4, 11, 12]) {
			const page = monthPage(2026, month, NONE);
			for (const week of page.weeks) expect(week).toHaveLength(7);
			for (const week of page.weeks) expect(week.some(cell => cell.month === month)).toBe(true);
		}
	});

	it('holds every day of the month exactly once', () => {
		const dates = monthPage(2026, 3, NONE)
			.weeks.flat()
			.filter(cell => cell.month === 3)
			.map(cell => cell.date);
		expect(dates).toHaveLength(31);
		expect(new Set(dates).size).toBe(31);
	});

	it('marks the two ends of the run and everything between them', () => {
		const picked = april.weeks
			.flat()
			.filter(cell => cell.picked !== null)
			.map(cell => [cell.iso, cell.picked]);
		expect(picked).toEqual([
			['2026-04-05', 'start'],
			['2026-04-06', 'mid'],
			['2026-04-07', 'end'],
		]);
	});

	it('calls a one-day run both ends at once', () => {
		const picked = monthPage(2026, 3, ONE)
			.weeks.flat()
			.filter(cell => cell.picked !== null);
		expect(picked.map(cell => [cell.iso, cell.picked])).toEqual([['2026-03-08', 'only']]);
	});

	it('disables the days behind today and marks today itself', () => {
		const cells = monthPage(2026, 3, NONE, { today: TODAY })
			.weeks.flat()
			.filter(cell => cell.month === 3);
		expect(cells.filter(cell => cell.past).map(cell => cell.date)).toEqual([1, 2, 3]);
		expect(cells.filter(cell => cell.today).map(cell => cell.iso)).toEqual([TODAY]);
	});

	it('has no past day at all when it is not told what today is', () => {
		expect(
			monthPage(2026, 3, NONE)
				.weeks.flat()
				.some(cell => cell.past)
		).toBe(false);
	});

	it('carries what a day already holds rather than reducing it to a dot', () => {
		expect(april.weeks.flat().find(cell => cell.iso === '2026-04-12')?.booked).toBe('Errands');
		expect(april.weeks.flat().find(cell => cell.iso === '2026-04-05')?.booked).toBeNull();
	});

	it('names the months either side, across a year end', () => {
		expect(monthStep(april, -1)).toBe('March 2026');
		expect(monthStep(april, 1)).toBe('May 2026');
		expect(monthStep(monthPage(2026, 12, NONE), 1)).toBe('January 2027');
		expect(monthStep(monthPage(2026, 1, NONE), -1)).toBe('December 2025');
	});

	it('spells a day the one way', () => {
		expect(isoDay(2026, 4, 5)).toBe('2026-04-05');
	});
});

describe('dayName', () => {
	const april = monthPage(2026, 4, RUN, { today: '2026-04-06', booked });

	function nameOf(iso: string): string {
		return dayName(april.weeks.flat().find(cell => cell.iso === iso) ?? april.weeks[0]![0]!);
	}

	it('says every mark a day is carrying, so none of them is only a shape', () => {
		expect(nameOf('2026-04-05')).toBe('5, first day');
		expect(nameOf('2026-04-06')).toBe('6, today, picked');
		expect(nameOf('2026-04-07')).toBe('7, last day');
		expect(nameOf('2026-04-13')).toBe('13, Errands');
		expect(nameOf('2026-04-20')).toBe('20');
	});
});
