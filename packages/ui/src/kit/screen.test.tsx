import { act, cleanup, fireEvent, render, renderHook, screen } from '@testing-library/react';
import { useRef } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { useTyping, useTypingScroll } from './screen';

/**
 * The hook's whole job is a number written to one scroller, so the specimen is a scroller with a
 * control far enough down it that there is a number to check.
 */
function Specimen({ readOnly = false }: { readOnly?: boolean }) {
	useTypingScroll();
	return (
		<div data-testid="scroller" style={{ overflowY: 'auto' }}>
			<div style={{ height: '2000px' }} />
			<label data-testid="box">
				Title
				<input aria-label="Title" readOnly={readOnly} />
			</label>
		</div>
	);
}

/** jsdom lays nothing out, so the two facts the hook reads have to be stated. */
function measure(scroller: HTMLElement, box: HTMLElement, top: number) {
	Object.defineProperty(scroller, 'scrollHeight', { value: 2400, configurable: true });
	Object.defineProperty(scroller, 'clientHeight', { value: 800, configurable: true });
	scroller.getBoundingClientRect = () => new DOMRect(0, 100, 390, 800);
	box.getBoundingClientRect = () => new DOMRect(0, top, 358, 40);
}

function touch(matches: boolean) {
	window.matchMedia = vi.fn().mockReturnValue({
		matches,
		addEventListener: vi.fn(),
		removeEventListener: vi.fn(),
	});
}

describe('useTypingScroll', () => {
	beforeEach(() => touch(true));
	afterEach(cleanup);

	it('puts the focused control 12px under the top of its scroller', () => {
		const view = render(<Specimen />);
		const scroller = view.getByTestId('scroller');
		measure(scroller, view.getByTestId('box'), 700);
		scroller.scrollTop = 500;

		fireEvent.focusIn(view.getByLabelText('Title'));

		/* 700 − 100 − 12 = 588 to travel, from 500. */
		expect(scroller.scrollTop).toBe(1088);
	});

	it('never writes the horizontal axis', () => {
		const view = render(<Specimen />);
		const scroller = view.getByTestId('scroller');
		measure(scroller, view.getByTestId('box'), 700);
		scroller.scrollLeft = 0;

		fireEvent.focusIn(view.getByLabelText('Title'));

		expect(scroller.scrollLeft).toBe(0);
	});

	it('leaves a pointer that can hover alone', () => {
		touch(false);
		const view = render(<Specimen />);
		const scroller = view.getByTestId('scroller');
		measure(scroller, view.getByTestId('box'), 700);
		scroller.scrollTop = 500;

		fireEvent.focusIn(view.getByLabelText('Title'));

		expect(scroller.scrollTop).toBe(500);
	});

	/**
	 * iOS pans the document to make room for a keyboard, and a field that leaves the document while
	 * focused (`Select`'s filter, shut by an outside press) takes the keyboard with it without the pan
	 * coming back — the page's ground shows under the shell where the slug should float.
	 */
	it('puts the document back at its top once the focused control leaves it', async () => {
		const scrollTo = vi.fn();
		window.scrollTo = scrollTo;
		const view = render(<Specimen />);
		measure(view.getByTestId('scroller'), view.getByTestId('box'), 700);
		const input = view.getByLabelText('Title');

		act(() => {
			input.focus();
		});
		Object.defineProperty(window, 'scrollY', { value: 280, configurable: true });

		await act(async () => {
			view.getByTestId('box').remove();
			await Promise.resolve();
		});
		expect(scrollTo).toHaveBeenCalledWith(0, 0);
		Object.defineProperty(window, 'scrollY', { value: 0, configurable: true });
	});

	it('leaves the document alone when it has not moved', async () => {
		const scrollTo = vi.fn();
		window.scrollTo = scrollTo;
		const view = render(<Specimen />);
		measure(view.getByTestId('scroller'), view.getByTestId('box'), 700);

		act(() => {
			view.getByLabelText('Title').focus();
		});
		await act(async () => {
			view.getByTestId('box').remove();
			await Promise.resolve();
		});
		expect(scrollTo).not.toHaveBeenCalled();
	});

	it('leaves a control that raises no keyboard alone', () => {
		const view = render(<Specimen readOnly />);
		const scroller = view.getByTestId('scroller');
		measure(scroller, view.getByTestId('box'), 700);
		scroller.scrollTop = 500;

		fireEvent.focusIn(view.getByLabelText('Title'));

		expect(scroller.scrollTop).toBe(500);
	});
});

describe('useTyping', () => {
	beforeEach(() => touch(true));
	afterEach(() => {
		document.body.innerHTML = '';
	});

	function field(markup: string): HTMLElement {
		const holder = document.createElement('div');
		holder.innerHTML = markup;
		document.body.append(holder);
		return holder.firstElementChild as HTMLElement;
	}

	it('is false while nothing has focus', () => {
		const { result } = renderHook(() => useTyping());
		expect(result.current).toBe(false);
	});

	it('is true while a control that raises a keyboard has focus, and false once it lets go', () => {
		const input = field('<input>');
		const { result } = renderHook(() => useTyping());

		act(() => {
			input.focus();
		});
		expect(result.current).toBe(true);

		act(() => {
			input.blur();
		});
		expect(result.current).toBe(false);
	});

	it.for(['<textarea></textarea>', '<input type="search">', '<input type="number">'])(
		'counts %s',
		markup => {
			const control = field(markup);
			const { result } = renderHook(() => useTyping());
			act(() => {
				control.focus();
			});
			expect(result.current).toBe(true);
		}
	);

	it.for([
		'<button type="button">Go</button>',
		'<input type="checkbox">',
		'<input type="radio">',
		'<input readonly>',
	])('does not count %s', markup => {
		const control = field(markup);
		const { result } = renderHook(() => useTyping());
		act(() => {
			control.focus();
		});
		expect(result.current).toBe(false);
	});

	/**
	 * `Select`'s filter exists only while its list is open, and an outside press shuts the list
	 * without handing focus back — so the focused field leaves the document, and no browser fires
	 * `focusout` for that. The keyboard goes; the chrome must come back with it.
	 */
	it('is false once the focused control leaves the document', async () => {
		const input = field('<input>');
		const { result } = renderHook(() => useTyping());

		act(() => {
			input.focus();
		});
		expect(result.current).toBe(true);

		await act(async () => {
			input.remove();
			await Promise.resolve();
		});
		expect(result.current).toBe(false);
	});

	it('is false once the focused control leaves the document with its container', async () => {
		const input = field('<input>');
		const { result } = renderHook(() => useTyping());

		act(() => {
			input.focus();
		});

		await act(async () => {
			input.parentElement?.remove();
			await Promise.resolve();
		});
		expect(result.current).toBe(false);
	});

	it('is false where the pointer can hover, whatever has focus', () => {
		touch(false);
		const input = field('<input>');
		const { result } = renderHook(() => useTyping());

		act(() => {
			input.focus();
		});
		expect(result.current).toBe(false);
	});

	it('is true from the start when a control already has focus', () => {
		field('<input>').focus();
		const { result } = renderHook(() => useTyping());
		expect(result.current).toBe(true);
	});

	it('counts only a control inside `within` when given', () => {
		function Probe() {
			const box = useRef<HTMLDivElement>(null);
			const typing = useTyping(box);
			return (
				<>
					<div ref={box}>
						<input aria-label="inside" />
					</div>
					<input aria-label="outside" />
					<output>{typing ? 'typing' : 'idle'}</output>
				</>
			);
		}
		render(<Probe />);

		act(() => {
			screen.getByLabelText('outside').focus();
		});
		expect(screen.getByRole('status').textContent).toBe('idle');

		act(() => {
			screen.getByLabelText('inside').focus();
		});
		expect(screen.getByRole('status').textContent).toBe('typing');
	});
});
