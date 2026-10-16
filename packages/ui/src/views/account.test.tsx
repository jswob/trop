import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { AccountPage, APPEARANCE_LINE } from './account';

describe('AccountPage', () => {
	it('draws Sign out alone when it is handed no appearance', () => {
		render(<AccountPage />);

		expect(screen.queryByRole('group', { name: APPEARANCE_LINE })).toBeNull();
		expect(screen.getByRole('button', { name: 'Sign out' })).toBeDefined();
	});

	it("draws the device's appearance over Sign out, the current mode pressed", () => {
		render(<AccountPage appearance={{ value: 'light', choose: () => {} }} />);

		const group = screen.getByRole('group', { name: APPEARANCE_LINE });
		expect(screen.getByRole('button', { name: 'Light' }).getAttribute('aria-pressed')).toBe('true');
		expect(
			group.compareDocumentPosition(screen.getByRole('button', { name: 'Sign out' })) &
				Node.DOCUMENT_POSITION_FOLLOWING
		).toBeTruthy();
	});

	it('hands a press to choose', () => {
		const choose = vi.fn();
		render(<AccountPage appearance={{ value: 'dark', choose }} />);

		fireEvent.click(screen.getByRole('button', { name: 'Light' }));
		expect(choose).toHaveBeenCalledWith('light');
	});
});
