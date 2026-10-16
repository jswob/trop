import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { Switch } from './Switch';

const OPTIONS = [
	{ value: 'dark', word: 'Dark' },
	{ value: 'light', word: 'Light' },
] as const;

describe('Switch', () => {
	it('is one labelled group of pressed-state buttons, the chosen one pressed', () => {
		render(<Switch label="Appearance" options={OPTIONS} value="light" choose={() => {}} />);

		const group = screen.getByRole('group', { name: 'Appearance' });
		const buttons = group.querySelectorAll('button');
		expect(buttons).toHaveLength(2);
		expect(screen.getByRole('button', { name: 'Dark' }).getAttribute('aria-pressed')).toBe('false');
		expect(screen.getByRole('button', { name: 'Light' }).getAttribute('aria-pressed')).toBe('true');
	});

	it('draws the chosen word on the ink plate and the rest as bare words', () => {
		render(<Switch label="Appearance" options={OPTIONS} value="dark" choose={() => {}} />);

		expect(screen.getByRole('button', { name: 'Dark' }).classList.contains('bg-ui-ink')).toBe(true);
		expect(screen.getByRole('button', { name: 'Light' }).classList.contains('bg-ui-ink')).toBe(
			false
		);
		expect(screen.getByRole('button', { name: 'Light' }).classList.contains('text-ui-ink-2')).toBe(
			true
		);
	});

	it('hands the pressed option to choose', () => {
		const choose = vi.fn();
		render(<Switch label="Appearance" options={OPTIONS} value="dark" choose={choose} />);

		fireEvent.click(screen.getByRole('button', { name: 'Light' }));
		expect(choose).toHaveBeenCalledWith('light');
	});

	it('takes null as a value, for a setting whose first option is none', () => {
		const choose = vi.fn();
		render(
			<Switch
				label="Project"
				options={[
					{ value: null, word: 'All projects' },
					{ value: 'Errands', word: 'Errands' },
				]}
				value={null}
				choose={choose}
			/>
		);

		expect(screen.getByRole('button', { name: 'All projects' }).getAttribute('aria-pressed')).toBe(
			'true'
		);
		fireEvent.click(screen.getByRole('button', { name: 'Errands' }));
		expect(choose).toHaveBeenCalledWith('Errands');
	});
});
