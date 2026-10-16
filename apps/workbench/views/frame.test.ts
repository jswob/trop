import { readFileSync, readdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { DESTINATIONS } from '@app/ui/views/destinations';
import { describe, expect, it } from 'vitest';

import { PLACES, VIEWS, undrawnDests, viewById } from './index';

const VIEW_DIR = dirname(fileURLToPath(import.meta.url));

/** Where the app's views live, the UI package — the frame among them. */
const APP_VIEW_DIR = resolve(VIEW_DIR, '../../../packages/ui/src/views');

/** Reading from disk, not `?raw`: Vitest stubs raw source imports out to an empty string. */
function sourceOf(file: string, dir = VIEW_DIR): string {
	return readFileSync(resolve(dir, file), 'utf8');
}

/**
 * The code, without the prose. A doc comment naming `kit/SplitShell` is how these files
 * explain themselves; only a reference the compiler sees is a file drawing chrome.
 */
function codeOf(file: string, dir = VIEW_DIR): string {
	return sourceOf(file, dir)
		.replace(/\/\*[\s\S]*?\*\//g, '')
		.replace(/^[ \t]*\/\/.*$/gm, '');
}

const placeFiles = readdirSync(VIEW_DIR).filter(file => file.endsWith('-places.tsx'));

describe('places', () => {
	it('has at least one place file', () => {
		expect(placeFiles.length).toBeGreaterThan(0);
	});

	it.each(placeFiles)('%s draws no chrome of its own', file => {
		const code = codeOf(file);
		expect(code).not.toMatch(/\bSplitShell\b/);
		expect(code).not.toMatch(/\bRailShell\b/);
		expect(code).not.toMatch(/\bDESTINATIONS\b/);
	});

	it('leaves the shell to views/frame alone, wherever the views live', () => {
		const shellUsers = [
			{ dir: VIEW_DIR, at: 'apps/workbench/views' },
			{ dir: APP_VIEW_DIR, at: 'packages/ui/src/views' },
		].flatMap(({ dir, at }) =>
			readdirSync(dir)
				.filter(file => file.endsWith('.tsx'))
				.filter(file => /\b(SplitShell|RailShell)\b/.test(codeOf(file, dir)))
				.map(file => `${at}/${file}`)
		);

		expect(shellUsers).toEqual(['packages/ui/src/views/frame.tsx']);
	});

	it('gives every place a unique id and at least one variant', () => {
		expect(new Set(PLACES.map(place => place.id)).size).toBe(PLACES.length);

		for (const place of PLACES) {
			expect(place.variants.length).toBeGreaterThan(0);
			expect(new Set(place.variants.map(variant => variant.id)).size).toBe(place.variants.length);
		}
	});

	/**
	 * A place is one **errand**, so one destination may hold several. The ground is not an errand —
	 * it is the surface every page opens over — so it stays exactly one, and a second would mean two
	 * screens both claiming to be where you land.
	 */
	it('draws exactly one ground', () => {
		expect(PLACES.filter(place => place.dest === null)).toHaveLength(1);
	});

	/**
	 * **The rule that a state a hand reaches is no entry, as the two things about it a test can
	 * hold.**
	 *
	 * Two entries of one place differing only in what a hand owns are one entry. Most of what a hand
	 * owns is not expressible — the types have no field for the ledger's position, a peek or a scroll
	 * seed, so the compiler refuses them. `menu` is the exception that survives, and an exception
	 * nobody counts is a rule that has been reopened.
	 *
	 * **One, not two.** The register cannot be raised over a hold, so the only state to list is the
	 * register raised over the ground. The count is the point of the test — loosening it to *at most
	 * two* would let the exception grow.
	 */
	it('raises the register at most once per place, and once in the whole space', () => {
		const raised = PLACES.flatMap(place =>
			place.variants.filter(variant => variant.draw().menu === true).map(() => place.id)
		);

		expect(raised).toHaveLength(1);
		expect(new Set(raised).size).toBe(1);
	});

	/**
	 * **The second counted exception: a nested page listed open.** A place lists at most one, and it
	 * names a record that place's own nest can open — a `nested` with no nest, or naming nobody,
	 * would list a state the frame draws as the page itself.
	 */
	it('opens a nested page in at most one state per place, on a record its nest can open', () => {
		for (const place of PLACES) {
			const draws = place.variants
				.map(variant => variant.draw())
				.filter(draw => draw.nested !== undefined);

			expect(draws.length).toBeLessThanOrEqual(1);
			for (const draw of draws)
				expect(draw.nest?.name(draw.nested ?? '')).toEqual(expect.any(String));
		}
	});

	it.each(placeFiles)('%s seeds no position a hand owns', file => {
		const code = codeOf(file);

		expect(code).not.toMatch(/\bledgerOpen\b/);
		expect(code).not.toMatch(/\bpeeking\b/);
		expect(code).not.toMatch(/\bgoing\b/);
		expect(code).not.toMatch(/\bseed\b/);
	});

	it('gives every place a destination the rail actually offers', () => {
		const ids = new Set(DESTINATIONS.map(dest => dest.id));

		for (const place of PLACES) {
			if (place.dest !== null) expect(ids).toContain(place.dest);
		}
	});

	it('leaves no destination undrawn', () => {
		expect(undrawnDests()).toEqual([]);
	});

	it('frames every place into a view with the same id', () => {
		expect(VIEWS.map(view => view.id)).toEqual(PLACES.map(place => place.id));
		expect(viewById('notes')?.dest).toBe('notes');
		expect(viewById('account')?.dest).toBe('account');
		expect(viewById('home')?.dest).toBeNull();
		expect(viewById('not-a-place')).toBeUndefined();
	});
});
