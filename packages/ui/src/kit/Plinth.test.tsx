import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import { Plinth } from './Plinth';

afterEach(cleanup);

describe('Plinth', () => {
	it('turns a member off by a sentence the page already prints, and adds no line of its own', () => {
		render(
			<>
				<p id="scope">Every note in 5 projects is picked.</p>
				<Plinth
					members={[
						{ id: 'finish', word: 'Finish', weight: 'primary' },
						{ id: 'all', word: 'Select all', saidBy: 'scope' },
					]}
				/>
			</>
		);

		const all = screen.getByRole('button', { name: 'Select all' });

		expect(all.hasAttribute('disabled')).toBe(true);
		expect(all.getAttribute('aria-describedby')).toBe('scope');
		expect(screen.getByRole('button', { name: 'Finish' }).hasAttribute('disabled')).toBe(false);
		expect(document.querySelectorAll('p')).toHaveLength(1);
	});

	it('prints a member’s own reason once, as before', () => {
		render(
			<Plinth
				members={[
					{ id: 'save', word: 'Save changes', weight: 'primary', reason: 'Nothing changed yet' },
					{ id: 'leave', word: 'Discard changes', reason: 'Nothing changed yet' },
				]}
			/>
		);

		expect(screen.getAllByText('Nothing changed yet')).toHaveLength(1);
		expect(screen.getByRole('button', { name: 'Discard changes' }).hasAttribute('disabled')).toBe(
			true
		);
	});
});
