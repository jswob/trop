import { readFileSync } from 'node:fs';
import path from 'node:path';

import { describe, expect, it } from 'vitest';

import { matchBracket } from '../lib/scan-source';
import { PROBE_SCOPES, installFocusBaselines, scopeOf } from './probes';

/**
 * Read from disk rather than imported `?raw`: a raw-source import resolves to an empty
 * string under the test runner, so an assertion on file contents that looks like it passes
 * is asserting on nothing.
 */
const source = readFileSync(path.resolve(import.meta.dirname, 'probes.ts'), 'utf8');

/**
 * The functions Playwright serialises into the page. `scopeOf` is not one of them — it runs
 * in the runner, so it is allowed to read module scope.
 */
const PAGE_FUNCTIONS = [
	'hScroll',
	'vOverflow',
	'hitTest',
	'notEmpty',
	'contrast',
	'installFocusBaselines',
	'readFocusState',
];

function bodyOf(name: string): string {
	const start = source.indexOf(`export function ${name}(`);
	const open = source.indexOf('{', source.indexOf(')', start));
	return source.slice(open, matchBracket(source, open));
}

describe('probes.ts', () => {
	it('exports every probe the runner evaluates', () => {
		for (const name of PAGE_FUNCTIONS) {
			expect(source).toContain(`export function ${name}(`);
		}
	});

	/**
	 * The trap that cost the first run of this runner: `hitTest` read a selector from a
	 * module-level `const`, which does not exist in the page. Playwright serialises the
	 * function and nothing around it, so a probe that closes over anything fails at
	 * evaluate time with a bare `ReferenceError`.
	 */
	it('declares no module-level constant a page function could close over', () => {
		const topLevelConsts = [...source.matchAll(/^const (\w+)\s*=/gm)].map(match => match[1]);

		expect(topLevelConsts).toEqual([]);
	});

	it('keeps the scope map out of the page functions', () => {
		for (const name of PAGE_FUNCTIONS) {
			expect(bodyOf(name)).not.toContain('PROBE_SCOPES');
		}
	});
});

describe('scopeOf', () => {
	it('puts the a11y and layout floor in the scope that fails the command', () => {
		expect(scopeOf('h-scroll')).toBe('floor');
		expect(scopeOf('focus-visible')).toBe('floor');
	});

	it('lets contrast print without failing the command', () => {
		expect(scopeOf('contrast')).toBe('settle');
	});

	it('treats an unknown probe as the floor rather than silently passing it', () => {
		expect(scopeOf('not-a-probe')).toBe('floor');
	});

	it('names every probe the runner runs', () => {
		expect(Object.keys(PROBE_SCOPES)).toHaveLength(6);
	});
});

describe('installFocusBaselines', () => {
	/**
	 * A page that opens with the keyboard already on a control — a nested page puts it on its way
	 * back — would have that control's ring in its "before", and the probe would report the
	 * ring it is looking at as nothing. The baselines are taken from an unfocused document.
	 */
	it('takes every baseline with nothing focused', () => {
		const button = document.createElement('button');
		button.textContent = 'Back';
		document.body.append(button);
		button.focus();

		installFocusBaselines();

		expect(document.activeElement).toBe(document.body);
		button.remove();
	});
});
