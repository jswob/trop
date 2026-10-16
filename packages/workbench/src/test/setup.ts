import { cleanup } from '@testing-library/react';
import { afterEach, vi } from 'vitest';

/**
 * What jsdom lacks and the bench's components ask for.
 *
 * The same stand-ins the app's own tests run on, so a test of the bench behaves here as it did
 * while the bench lived in the app.
 */

afterEach(() => {
	cleanup();
});

Object.defineProperty(window, 'matchMedia', {
	writable: true,
	value: vi.fn().mockImplementation((query: string) => ({
		matches: false,
		media: query,
		onchange: null,
		addListener: vi.fn(),
		removeListener: vi.fn(),
		addEventListener: vi.fn(),
		removeEventListener: vi.fn(),
		dispatchEvent: vi.fn(),
	})),
});

// A `function`, not an arrow: the observer is built with `new`, which an arrow cannot be.
global.ResizeObserver = vi.fn().mockImplementation(function observer() {
	return { observe: vi.fn(), unobserve: vi.fn(), disconnect: vi.fn() };
});

window.HTMLElement.prototype.scrollIntoView = vi.fn();
window.HTMLElement.prototype.hasPointerCapture = vi.fn();
window.HTMLElement.prototype.releasePointerCapture = vi.fn();

Element.prototype.getAnimations = () => [];
