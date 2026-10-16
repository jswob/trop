import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import { Select, useSelect, type SelectOption } from './Select';

const TIMES: SelectOption[] = [
	{ id: '1', label: '1 min' },
	{ id: '3', label: '3 min' },
	{ id: '5', label: '5 min' },
	{ id: '10', label: '10 min' },
];

/** The component as a caller draws it: the hand spread straight into the box. */
function Specimen({
	value = TIMES[1] ?? null,
	describedBy,
	error = null,
}: {
	value?: SelectOption | null;
	describedBy?: string;
	error?: string | null;
}) {
	const hand = useSelect({ options: TIMES, value });
	return (
		<>
			<Select label="Snooze for" {...hand.props} describedBy={describedBy} error={error} />
			<p id="note">Longer is quieter.</p>
			<button type="button">Elsewhere</button>
		</>
	);
}

const box = () => screen.getByRole('button', { name: /^Snooze for/ });
const filter = () => screen.getByRole('combobox');

afterEach(cleanup);

describe('Select', () => {
	it('names the closed box with its label and its value', () => {
		render(<Specimen />);
		expect(box().getAttribute('aria-label')).toBe('Snooze for, 3 min');
	});

	it('names the closed box with the label alone when nothing is chosen', () => {
		render(<Specimen value={null} />);
		expect(box().getAttribute('aria-label')).toBe('Snooze for');
	});

	it('wires describedBy onto the closed box, beside the error', () => {
		const { unmount } = render(<Specimen describedBy="note" />);
		expect(box().getAttribute('aria-describedby')).toBe('note');
		unmount();

		render(<Specimen describedBy="note" error="Too short" />);
		const ids = box().getAttribute('aria-describedby')?.split(' ') ?? [];
		expect(ids).toHaveLength(2);
		expect(ids).toContain('note');
	});

	it('leaves aria-describedby off when there is nothing to point at', () => {
		render(<Specimen />);
		expect(box().hasAttribute('aria-describedby')).toBe(false);
	});

	it('opens standing on the value, and the arrows go on from there', () => {
		render(<Specimen />);
		fireEvent.click(box());
		expect(filter().getAttribute('aria-activedescendant')).toBe('option-3');
		fireEvent.keyDown(filter(), { key: 'ArrowDown' });
		expect(filter().getAttribute('aria-activedescendant')).toBe('option-5');
	});

	it('opens with nothing active when nothing is chosen, and the first arrow finds the top', () => {
		render(<Specimen value={null} />);
		fireEvent.click(box());
		expect(filter().hasAttribute('aria-activedescendant')).toBe(false);
		fireEvent.keyDown(filter(), { key: 'ArrowDown' });
		expect(filter().getAttribute('aria-activedescendant')).toBe('option-1');
	});

	it('hands focus back to the box after a pick', () => {
		render(<Specimen />);
		fireEvent.click(box());
		fireEvent.keyDown(filter(), { key: 'ArrowDown' });
		fireEvent.keyDown(filter(), { key: 'Enter' });
		expect(document.activeElement).toBe(box());
		expect(box().getAttribute('aria-label')).toBe('Snooze for, 5 min');
	});

	it('hands focus back to the box after Escape', () => {
		render(<Specimen />);
		fireEvent.click(box());
		fireEvent.keyDown(filter(), { key: 'Escape' });
		expect(document.activeElement).toBe(box());
	});

	it('leaves focus where an outside click put it', () => {
		render(<Specimen />);
		fireEvent.click(box());
		const elsewhere = screen.getByRole('button', { name: 'Elsewhere' });
		fireEvent.pointerDown(elsewhere);
		fireEvent.mouseDown(elsewhere);
		expect(screen.queryByRole('combobox')).toBeNull();
		expect(document.activeElement).not.toBe(box());
		elsewhere.focus();
		fireEvent.click(elsewhere);
		expect(screen.queryByRole('combobox')).toBeNull();
		expect(document.activeElement).toBe(elsewhere);
	});
});
