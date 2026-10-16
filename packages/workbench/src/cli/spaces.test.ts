import { describe, expect, it } from 'vitest';

import { resolveSpace, resolveSpaces } from './spaces';

describe('resolveSpace', () => {
	/**
	 * The bug this exists to stop: the routes are `/views`, the census kind is `view`, and
	 * `--space views` matched nothing while printing a sentence about an empty space — which reads
	 * like a true statement rather than a typo, so the run looked like it had passed.
	 */
	it('resolves the plural the workbench routes use', () => {
		expect(resolveSpace('views')).toBe('view');
		expect(resolveSpace('view')).toBe('view');
		expect(resolveSpace('protos')).toBe('proto');
		expect(resolveSpace('components')).toBe('kit');
	});

	it('ignores case and surrounding space', () => {
		expect(resolveSpace('  Views ')).toBe('view');
	});

	it('refuses an unknown name loudly rather than matching nothing', () => {
		expect(() => resolveSpace('screens')).toThrow(/unknown space "screens"/);
		expect(() => resolveSpace('screens')).toThrow(/proto, view, kit/);
	});

	it('collapses spellings of one space to one kind', () => {
		expect(resolveSpaces(['views', 'view', 'places'])).toEqual(['view']);
	});
});
