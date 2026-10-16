import { describe, expect, it, vi } from 'vitest';

import { forwardKeys } from './frame-keys';

function frameWindow(): Window {
	const frame = document.createElement('iframe');
	document.body.append(frame);

	const inner = frame.contentWindow;
	if (inner === null) throw new Error('the frame has no window');
	return inner;
}

describe('forwardKeys', () => {
	it('hands a key pressed inside a frame to the page around it', () => {
		const inner = frameWindow();
		const heard = vi.fn<(event: KeyboardEvent) => void>();
		window.addEventListener('keydown', heard, { once: true });

		forwardKeys(inner, window);
		inner.document.body.dispatchEvent(new KeyboardEvent('keydown', { key: ']', bubbles: true }));

		expect(heard).toHaveBeenCalledTimes(1);
		expect(heard.mock.calls[0]?.[0].key).toBe(']');
	});

	it('keeps the modifiers, so the jump list still opens from inside a frame', () => {
		const inner = frameWindow();
		const heard = vi.fn<(event: KeyboardEvent) => void>();
		window.addEventListener('keydown', heard, { once: true });

		forwardKeys(inner, window);
		inner.document.body.dispatchEvent(
			new KeyboardEvent('keydown', { key: 'k', metaKey: true, bubbles: true })
		);

		expect(heard.mock.calls[0]?.[0].metaKey).toBe(true);
	});

	it('leaves a key alone while the frame is being typed into', () => {
		const inner = frameWindow();
		const input = inner.document.createElement('input');
		inner.document.body.append(input);
		const heard = vi.fn();
		window.addEventListener('keydown', heard);

		forwardKeys(inner, window);
		input.dispatchEvent(new KeyboardEvent('keydown', { key: 't', bubbles: true }));
		window.removeEventListener('keydown', heard);

		expect(heard).not.toHaveBeenCalled();
	});

	it('stops the frame acting on a key the page took', () => {
		const inner = frameWindow();
		const take = (event: KeyboardEvent) => event.preventDefault();
		window.addEventListener('keydown', take, { once: true });

		forwardKeys(inner, window);
		const pressed = new KeyboardEvent('keydown', { key: ']', bubbles: true, cancelable: true });
		inner.document.body.dispatchEvent(pressed);

		expect(pressed.defaultPrevented).toBe(true);
	});
});
