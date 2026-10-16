import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { useEffect, useRef, useState } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { pageHeadIn, useKeyboardKept } from './keyboard';

let waiting: FrameRequestCallback[] = [];

/** Runs the frame a press asked for. */
function frame(): void {
	const due = waiting;
	waiting = [];
	for (const run of due) run(0);
}

beforeEach(() => {
	waiting = [];
	vi.stubGlobal('requestAnimationFrame', (run: FrameRequestCallback) => waiting.push(run));
	vi.stubGlobal('cancelAnimationFrame', () => {
		waiting = [];
	});
});

afterEach(() => {
	vi.unstubAllGlobals();
	document.body.innerHTML = '';
});

/**
 * jsdom lays nothing out, so every element reads as not drawn. For the hook's own tests an
 * element is drawn while it is on the page and nothing above it is `hidden`, as a browser has it.
 */
function drawPage(): () => void {
	const drawn = vi.spyOn(HTMLElement.prototype, 'getClientRects').mockImplementation(function rects(
		this: HTMLElement
	) {
		const shown = this.isConnected && this.closest('[hidden]') === null;
		return (shown ? [new DOMRect()] : []) as unknown as DOMRectList;
	});

	return () => {
		drawn.mockRestore();
	};
}

/** Takes the late control off the page with no press, as a scroll that has ended does. */
let endScroll: () => void = () => undefined;

/** A page whose head can hold the keyboard, and a row that takes itself away when pressed. */
function Page({ land = true, away = false }: { land?: boolean; away?: boolean }) {
	const head = useRef<HTMLHeadingElement>(null);
	const [rows, setRows] = useState(['Groceries']);
	const [busy, setBusy] = useState(false);
	const [later, setLater] = useState(true);
	const [ground, setGround] = useState(true);
	useEffect(() => {
		endScroll = () => {
			setLater(false);
		};
	});
	useKeyboardKept(() => (land ? head.current : null), away);

	return (
		<>
			<h1 ref={head} tabIndex={-1}>
				Edit note
			</h1>
			{rows.map(row => (
				<button key={row} type="button" onClick={() => setRows([])}>
					Remove {row}
				</button>
			))}
			<button type="button" disabled={busy} onClick={() => setBusy(true)}>
				Save
			</button>
			<button type="button">Stays</button>
			{later && (
				<button type="button" onClick={() => undefined}>
					Scroll into view
				</button>
			)}
			<button type="button">Elsewhere</button>
			<div hidden={!ground}>
				<button type="button" onClick={() => setGround(false)}>
					Open Errands
				</button>
			</div>
			<p>Plain words</p>
		</>
	);
}

function press(name: string): HTMLElement {
	const control = screen.getByRole('button', { name });
	control.focus();
	fireEvent.click(control);
	return control;
}

describe('useKeyboardKept', () => {
	let undraw: () => void = () => undefined;
	beforeEach(() => {
		undraw = drawPage();
	});
	afterEach(() => {
		undraw();
	});

	it('lands the keyboard when a press takes its own control off the page', () => {
		render(<Page />);

		press('Remove Groceries');
		frame();

		expect(document.activeElement).toBe(screen.getByRole('heading'));
	});

	it('lands it when a press turns its own control off', () => {
		render(<Page />);

		/* A browser takes the keyboard off a control the moment it is turned off; jsdom leaves it. */
		press('Save');
		const lost = vi.spyOn(document, 'activeElement', 'get').mockReturnValue(document.body);
		frame();
		lost.mockRestore();

		expect(document.activeElement).toBe(screen.getByRole('heading'));
	});

	it('lands it when a press puts its own control out of sight, before the browser lets go of it', () => {
		render(<Page />);

		/* The control is still on the page, under something hidden, and still holds the keyboard. */
		const control = press('Open Errands');
		expect(document.activeElement).toBe(control);
		frame();

		expect(document.activeElement).toBe(screen.getByRole('heading'));
	});

	it('leaves the keyboard on a control that is still there', () => {
		render(<Page />);

		const control = press('Stays');
		frame();

		expect(document.activeElement).toBe(control);
	});

	it('leaves a press on something that is not a control alone', () => {
		render(<Page />);

		fireEvent.click(screen.getByText('Plain words'));
		frame();

		expect(document.activeElement).toBe(document.body);
	});

	it('does nothing where there is nowhere to land', () => {
		render(<Page land={false} />);

		press('Remove Groceries');
		frame();

		expect(document.activeElement).toBe(document.body);
	});

	it('lands it when the control goes away later, while it still held the keyboard', async () => {
		render(<Page />);

		const pointer = press('Scroll into view');
		frame();
		expect(document.activeElement).toBe(pointer);

		act(endScroll);

		await waitFor(() => {
			expect(document.activeElement).toBe(screen.getByRole('heading'));
		});
	});

	it('stops watching a control once the keyboard has moved on from it', async () => {
		render(<Page />);

		press('Scroll into view');
		frame();
		const elsewhere = screen.getByRole('button', { name: 'Elsewhere' });
		elsewhere.focus();
		act(endScroll);
		await waitFor(() => {
			expect(screen.queryByRole('button', { name: 'Scroll into view' })).toBeNull();
		});
		elsewhere.blur();
		await Promise.resolve();

		expect(document.activeElement).toBe(document.body);
	});
});

describe('useKeyboardKept, back from under a screen that stood in its place', () => {
	let undraw: () => void = () => undefined;
	beforeEach(() => {
		undraw = drawPage();
	});
	afterEach(() => {
		undraw();
	});

	it('lands the keyboard when nothing holds it', () => {
		const { rerender } = render(<Page away />);

		rerender(<Page />);
		frame();

		expect(document.activeElement).toBe(screen.getByRole('heading'));
	});

	it('leaves it with whatever took it meanwhile', () => {
		const { rerender } = render(<Page away />);

		rerender(<Page />);
		const taken = screen.getByRole('button', { name: 'Stays' });
		taken.focus();
		frame();

		expect(document.activeElement).toBe(taken);
	});

	it('does nothing on the way under, or while it stays', () => {
		const { rerender } = render(<Page />);

		rerender(<Page away />);
		frame();
		rerender(<Page away />);
		frame();

		expect(document.activeElement).toBe(document.body);
	});
});

describe('pageHeadIn', () => {
	it('answers with the last drawn head that can hold the keyboard, and nothing when none is drawn', () => {
		document.body.innerHTML =
			'<h1 tabindex="-1" id="old">Notes</h1><h1 tabindex="-1" id="new">Groceries</h1><h1 id="plain">Not a landing</h1>';
		const drawn = (id: string): void => {
			const head = document.getElementById(id);
			if (head !== null) head.getClientRects = () => [new DOMRect()] as unknown as DOMRectList;
		};

		expect(pageHeadIn(document)).toBeNull();

		drawn('old');
		drawn('plain');
		expect(pageHeadIn(document)?.id).toBe('old');

		drawn('new');
		expect(pageHeadIn(document)?.id).toBe('new');
	});
});
