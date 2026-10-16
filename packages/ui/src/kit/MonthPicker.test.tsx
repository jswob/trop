import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { MonthPicker } from './MonthPicker';
import { dayName, monthPage } from './month';

afterEach(cleanup);

describe('a past day', () => {
	it('is drawn fainter but takes a press like any other day', () => {
		const page = monthPage(2026, 3, { start: null, end: null }, { today: '2026-03-04' });
		const past = page.weeks.flat().find(cell => cell.iso === '2026-03-02')!;
		const onPick = vi.fn();
		render(<MonthPicker page={page} onPick={onPick} />);

		const [day] = screen.getAllByRole('button', { name: dayName(past) });
		if (day === undefined) throw new Error('no such day');
		expect(day.hasAttribute('disabled')).toBe(false);
		expect(day.className).toContain('text-ui-ink-3');

		fireEvent.click(day);
		expect(onPick).toHaveBeenCalledWith('2026-03-02');
	});
});
