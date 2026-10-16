import { describe, expect, it } from 'vitest';

import { frameUrl } from './frame-url';

describe('frameUrl', () => {
	it('addresses one variant of a place', () => {
		expect(frameUrl('view', 'people', 'empty', 'bare')).toBe(
			'/frame/view/people?variant=empty&chrome=bare'
		);
	});

	it('addresses a prototype without chrome', () => {
		expect(frameUrl('proto', 'scaffold', 'populated', 'bare')).toBe(
			'/frame/proto/scaffold?variant=populated&chrome=bare'
		);
	});

	it('escapes values that are not URL-safe', () => {
		expect(frameUrl('view', 'x', 'a b&c', 'bare')).toBe(
			'/frame/view/x?variant=a+b%26c&chrome=bare'
		);
	});

	it('names a clock speed when the frame is slowed', () => {
		expect(frameUrl('view', 'timeline-sheet', 'populated', 'bare', { rate: 0.2 })).toBe(
			'/frame/view/timeline-sheet?variant=populated&chrome=bare&rate=0.2'
		);
	});

	it('names the rate that turns animation off', () => {
		expect(frameUrl('view', 'timeline-sheet', 'populated', 'bare', { rate: 0 })).toBe(
			'/frame/view/timeline-sheet?variant=populated&chrome=bare&rate=0'
		);
	});

	it('pins a frame at a millisecond', () => {
		expect(frameUrl('view', 'timeline-sheet', 'populated', 'bare', { t: 120 })).toBe(
			'/frame/view/timeline-sheet?variant=populated&chrome=bare&t=120'
		);
	});

	it('leaves real time off the URL, so every existing address is unchanged', () => {
		expect(frameUrl('view', 'people', 'empty', 'bare', {})).toBe(
			frameUrl('view', 'people', 'empty', 'bare')
		);
	});

	it('names the light mode', () => {
		expect(frameUrl('view', 'people', 'empty', 'bare', {}, 'light')).toBe(
			'/frame/view/people?variant=empty&chrome=bare&theme=light'
		);
	});

	it('leaves dark off the URL, because dark is the tokens with nothing set', () => {
		expect(frameUrl('view', 'people', 'empty', 'bare', {}, 'dark')).toBe(
			frameUrl('view', 'people', 'empty', 'bare')
		);
	});

	it('carries the position a frame has walked to', () => {
		expect(frameUrl('view', 'people', 'empty', 'bare', {}, 'dark', 'billing')).toBe(
			'/frame/view/people?variant=empty&chrome=bare&where=billing'
		);
	});

	it('leaves a frame that has not moved off the URL, so every existing address is unchanged', () => {
		expect(frameUrl('view', 'people', 'empty', 'bare', {}, 'dark', null)).toBe(
			frameUrl('view', 'people', 'empty', 'bare')
		);
	});
});
