import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import {
	PlaceholderRows,
	READ_FAILED,
	TRY_AGAIN,
	WaitFailed,
	Waiting,
	WaitingNote,
} from './Waiting';

function Wait() {
	return (
		<Waiting>
			<WaitingNote />
			<PlaceholderRows rows={2} />
		</Waiting>
	);
}

describe('Waiting', () => {
	it('says it is loading with no failure above it', () => {
		const { container } = render(<Wait />);
		expect(screen.getByRole('status').textContent).toContain('loading');
		expect(container.querySelector('[data-ui-glaze-failed]')).toBeNull();
	});

	it('says it is loading under a provider that holds no failure', () => {
		render(
			<WaitFailed onRetry={null}>
				<Wait />
			</WaitFailed>
		);
		expect(screen.getByRole('status').textContent).toContain('loading');
	});

	/** A failed wait never glazes, and its note says so with the one press. */
	it('stops the glaze and says the read failed, with a press that asks again', () => {
		const retry = vi.fn();
		const { container } = render(
			<WaitFailed onRetry={retry}>
				<Wait />
			</WaitFailed>
		);

		const note = screen.getByRole('status');
		expect(note.textContent).toContain(READ_FAILED);
		expect(note.textContent).not.toContain('loading');
		expect(container.querySelector('[data-ui-glaze-failed]')).not.toBeNull();

		fireEvent.click(screen.getByRole('button', { name: TRY_AGAIN }));
		expect(retry).toHaveBeenCalledTimes(1);
	});
});
