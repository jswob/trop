import { fireEvent, render, screen } from '@testing-library/react';
import { useState } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { Modal } from './Modal';

afterEach(() => {
	document.body.innerHTML = '';
});

function Opened({ onClose }: { onClose?: () => void }) {
	return (
		<Modal
			title="Remove Groceries"
			footer={<button type="button">Keep it</button>}
			onClose={onClose}
		>
			<p>Gone for good.</p>
		</Modal>
	);
}

/** A page with one control that raises the modal, as a page's errand does. */
function Page() {
	const [open, setOpen] = useState(false);

	return (
		<>
			<button type="button" onClick={() => setOpen(true)}>
				Remove this note
			</button>
			{open && <Opened onClose={() => setOpen(false)} />}
		</>
	);
}

describe('Modal', () => {
	it('closes on Escape, by the handler it was given', () => {
		const onClose = vi.fn();
		render(<Opened onClose={onClose} />);

		fireEvent.keyDown(window, { key: 'Escape' });

		expect(onClose).toHaveBeenCalledTimes(1);
	});

	it('closes from its own Close control', () => {
		const onClose = vi.fn();
		render(<Opened onClose={onClose} />);

		fireEvent.click(screen.getByRole('button', { name: 'Close' }));

		expect(onClose).toHaveBeenCalledTimes(1);
	});

	it('leaves a key something else already took, and every key but Escape', () => {
		const onClose = vi.fn();
		render(<Opened onClose={onClose} />);

		const taken = new KeyboardEvent('keydown', { key: 'Escape', cancelable: true });
		taken.preventDefault();
		window.dispatchEvent(taken);
		fireEvent.keyDown(window, { key: 'Enter' });

		expect(onClose).not.toHaveBeenCalled();
	});

	it('does nothing on Escape while it cannot be closed, as while its errand runs', () => {
		render(<Opened />);

		expect(() => fireEvent.keyDown(window, { key: 'Escape' })).not.toThrow();
		expect(screen.getByRole('dialog')).toBeDefined();
	});

	it('takes the keyboard when it opens', () => {
		render(<Page />);

		const opener = screen.getByRole('button', { name: 'Remove this note' });
		opener.focus();
		fireEvent.click(opener);

		expect(document.activeElement).toBe(screen.getByRole('dialog'));
	});

	it('hands the keyboard back to what raised it when it goes', () => {
		render(<Page />);

		const opener = screen.getByRole('button', { name: 'Remove this note' });
		opener.focus();
		fireEvent.click(opener);
		fireEvent.keyDown(window, { key: 'Escape' });

		expect(screen.queryByRole('dialog')).toBeNull();
		expect(document.activeElement).toBe(opener);
	});

	it('does not pull the keyboard back from something that took it on the way out', () => {
		function Leaves() {
			const [open, setOpen] = useState(true);
			return (
				<>
					<input aria-label="Name" />
					{open && (
						<Modal
							title="Leave"
							footer={
								<button
									type="button"
									onClick={() => {
										screen.getByLabelText('Name').focus();
										setOpen(false);
									}}
								>
									Go
								</button>
							}
						>
							<p>Going.</p>
						</Modal>
					)}
				</>
			);
		}
		const outside = document.createElement('button');
		document.body.append(outside);
		outside.focus();
		render(<Leaves />);

		fireEvent.click(screen.getByRole('button', { name: 'Go' }));

		expect(document.activeElement).toBe(screen.getByLabelText('Name'));
	});
});
