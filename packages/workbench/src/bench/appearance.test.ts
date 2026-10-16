import { describe, expect, it } from 'vitest';

import { resolveAppearance, resolveFolded } from './appearance';

describe('resolveAppearance', () => {
	it('follows the device when nothing was chosen', () => {
		expect(resolveAppearance(null, true)).toBe('dark');
		expect(resolveAppearance(null, false)).toBe('light');
	});

	it('keeps a choice over what the device prefers', () => {
		expect(resolveAppearance('light', true)).toBe('light');
		expect(resolveAppearance('dark', false)).toBe('dark');
	});

	it('follows the device when what was stored is not an appearance', () => {
		expect(resolveAppearance('sepia', true)).toBe('dark');
	});
});

describe('resolveFolded', () => {
	it('opens the sidebar unless it was left folded', () => {
		expect(resolveFolded(null)).toBe(false);
		expect(resolveFolded('open')).toBe(false);
		expect(resolveFolded('folded')).toBe(true);
	});
});
