import { act, cleanup, fireEvent, render, screen } from '@testing-library/react';
import { useState } from 'react';
import { afterEach, describe, expect, it } from 'vitest';

import type { Period } from '@app/ui/kit/period';
import { PeriodSentence } from './PeriodSentence';
import type { PeriodOpen } from './PeriodSentence';

const TODAY = '2026-03-01';

const PERIODS: Period[] = [
	{ from: '2026-03-01', to: '2026-03-31' },
	{ from: '2026-05-01', to: null },
];

/** The sentence as a caller holds it: the periods in state, handed straight back. */
function Held({ periods, opened = null }: { periods: Period[]; opened?: PeriodOpen | null }) {
	const [held, setHeld] = useState(periods);
	return (
		<PeriodSentence
			id="periods"
			subject="Ada"
			periods={held}
			today={TODAY}
			opened={opened}
			onChange={setHeld}
		/>
	);
}

/** Runs the frame the component hands the keyboard over on. */
async function nextFrame() {
	await act(() => new Promise(resolve => requestAnimationFrame(() => resolve(null))));
}

const statement = () => document.getElementById('periods')?.firstElementChild?.textContent ?? '';
const grid = () => screen.queryByRole('group');

/**
 * The sentence, asserted where a still cannot answer: the statement is the control, a pressed end
 * is the end a day sets, and the keyboard stays where the hand is.
 *
 * `jest-dom`'s matchers are deliberately not used.
 */
describe('PeriodSentence', () => {
	afterEach(cleanup);

	it('states every period as one sentence, the ends its presses', () => {
		render(<Held periods={PERIODS} />);
		expect(statement()).toContain('FromMar 1untilMar 31,and fromMay 1untilno end,oradd a period');
		expect(grid()).toBeNull();
	});

	it('says any time when there are no periods', () => {
		render(<Held periods={[]} />);
		expect(statement()).toContain('Any time,oradd a period');
	});

	it('opens the grid for the pressed end and sets that end, never the nearer one', () => {
		render(<Held periods={PERIODS} />);
		const end = screen.getByRole('button', {
			name: 'Change when it ends: Ada, Mar 1 – Mar 31',
		});
		fireEvent.click(end);
		expect(end.getAttribute('aria-expanded')).toBe('true');
		expect(grid()?.getAttribute('aria-label')).toBe('When Ada ends: Mar 1 – Mar 31');

		fireEvent.click(screen.getByRole('button', { name: /^20,/ }));
		expect(grid()?.getAttribute('aria-label')).toBe('When Ada ends: Mar 1 – Mar 20');
	});

	it('holds the open word pressed while its end is open', () => {
		render(<Held periods={PERIODS} opened={{ index: 1, edge: 'to', turn: 0 }} />);
		const open = screen.getByRole('button', { name: 'No end for Ada, May 1 – no end' });
		expect(open.getAttribute('aria-pressed')).toBe('true');

		fireEvent.click(
			screen.getByRole('button', { name: 'Change when it starts: Ada, May 1 – no end' })
		);
		const start = screen.getByRole('button', { name: 'No start for Ada, May 1 – no end' });
		expect(start.getAttribute('aria-pressed')).toBe('false');
	});

	it('removes the pressed period and hands the keyboard to the next one', async () => {
		render(<Held periods={PERIODS} opened={{ index: 0, edge: 'to', turn: 0 }} />);
		fireEvent.click(screen.getByRole('button', { name: /^Remove this period: Ada, Mar 1/ }));
		await nextFrame();
		expect(statement()).toContain('FromMay 1untilno end,');
		expect(grid()).toBeNull();
		expect(document.activeElement?.getAttribute('aria-label')).toBe(
			'Change when it starts: Ada, May 1 – no end'
		);
	});

	it('adds a period from today with no end, opened on its start', async () => {
		render(<Held periods={[{ from: '2026-05-01', to: null }]} />);
		fireEvent.click(screen.getByRole('button', { name: 'Add a period to Ada' }));
		await nextFrame();
		expect(statement()).toContain('FromMar 1untilno end,and fromMay 1');
		expect(grid()?.getAttribute('aria-label')).toBe('When Ada starts: Mar 1 – no end');
		expect(document.activeElement?.getAttribute('aria-label')).toBe(
			'Change when it starts: Ada, Mar 1 – no end'
		);
	});

	it('shuts the grid on Escape and hands the keyboard back to its end', async () => {
		render(<Held periods={PERIODS} opened={{ index: 0, edge: 'to', turn: 0 }} />);
		fireEvent.keyDown(screen.getByRole('button', { name: /^20,/ }), { key: 'Escape' });
		await nextFrame();
		expect(grid()).toBeNull();
		expect(document.activeElement?.getAttribute('aria-label')).toBe(
			'Change when it ends: Ada, Mar 1 – Mar 31'
		);
	});
});
