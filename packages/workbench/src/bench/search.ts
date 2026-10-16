import { z } from 'zod';

import { splitVariants, stageSearchSchema } from '../lib/stage-search';

/**
 * What the bench's address carries: everything the old stage's did, the mode, and the cursor.
 *
 * `show` is absent in States, the stage at rest. `at` names the state the stage is standing on:
 * the one `]` shows alone, and the one Widths draws.
 */
export const benchSearchSchema = stageSearchSchema.extend({
	show: z.enum(['widths']).optional(),
	at: z.string().optional(),
	/** The props turned on the one state showing: `weight:primary,running:Saving`. */
	args: z.string().optional(),
	/** Whether a component's named parts are outlined and measured. */
	parts: z.enum(['on']).optional(),
	/** Token values being tried, never written: `color-accent.dark:%23b8860b`. */
	try: z.string().optional(),
	/** Whether a candidate's notes are open under its frames. Proto alone has them. */
	notes: z.enum(['on']).optional(),
});

export type BenchSearch = z.infer<typeof benchSearchSchema>;

type BenchKey = keyof typeof benchSearchSchema.shape;

function pick(search: Record<string, unknown>, keys: readonly BenchKey[]): BenchSearch {
	const kept: Record<string, unknown> = {};

	for (const key of keys) {
		const parsed = benchSearchSchema.shape[key].safeParse(search[key]);
		if (parsed.success && parsed.data !== undefined) kept[key] = parsed.data;
	}

	return kept;
}

/**
 * The address, as the bench reads it. A value it cannot read is dropped alone.
 *
 * The old stage dropped the whole search on one bad value, and a route's validator does not stop
 * the raw value reaching the page, so `?width=wide` took the stage down with it. Read a key at a
 * time, a mistyped width costs the width and nothing else.
 */
export function parseBenchSearch(search: Record<string, unknown>): BenchSearch {
	return pick(search, Object.keys(benchSearchSchema.shape) as BenchKey[]);
}

/**
 * The options of the bar, which a link to another place takes with it.
 *
 * What names the states of one place is left behind: `variants`, `at`, a `frames` list and the
 * props turned on one state mean nothing somewhere else. So is a pin, which is a capture of one
 * frame rather than an option. A value being tried goes along: it is tried on the whole system.
 */
const CARRIED = ['show', 'width', 'theme', 'scale', 'rate', 'parts', 'try', 'notes'] as const;

/** The part of an address a link to another place keeps. A value it cannot read is dropped alone. */
export function carry(search: Record<string, unknown>): BenchSearch {
	return pick(search, CARRIED);
}

function without(search: BenchSearch, keys: readonly (keyof BenchSearch)[]): BenchSearch {
	return Object.fromEntries(
		Object.entries(search).filter(([key]) => !keys.includes(key as keyof BenchSearch))
	);
}

/** The values the stage has at rest, which an address leaves off. */
const AT_REST: BenchSearch = { width: 'desktop', theme: 'dark', rate: 1 };

/** Writes options into an address, leaving off any that is the stage at rest. */
export function withOption(search: BenchSearch, patch: BenchSearch): BenchSearch {
	const merged: Record<string, unknown> = { ...search, ...patch };

	return Object.fromEntries(
		Object.entries(merged).filter(
			([key, value]) => value !== undefined && value !== AT_REST[key as keyof BenchSearch]
		)
	);
}

/** The states the stage shows, in the place's own order. Every state when the address names none. */
export function shownOf(ids: readonly string[], search: BenchSearch): string[] {
	const requested = splitVariants(search.variants);
	const shown = requested ? ids.filter(id => requested.includes(id)) : [];

	return shown.length > 0 ? shown : [...ids];
}

/** The state the stage is standing on. */
export function cursorOf(ids: readonly string[], search: BenchSearch): string | undefined {
	return search.at !== undefined && ids.includes(search.at) ? search.at : shownOf(ids, search)[0];
}

/**
 * One press of `[` or `]`.
 *
 * With several states showing it shows the cursor alone, so the first press never skips a state;
 * after that it moves. In Widths one state is always showing, so it only ever moves.
 */
export function step(ids: readonly string[], search: BenchSearch, by: 1 | -1): BenchSearch {
	const cursor = cursorOf(ids, search);
	if (cursor === undefined || ids.length <= 1) return search;

	const next = ids[(ids.indexOf(cursor) + by + ids.length) % ids.length] ?? cursor;

	const kept = without(search, ['args']);

	if (search.show === 'widths') return { ...kept, at: next };
	if (shownOf(ids, search).length > 1) return { ...kept, variants: cursor, at: cursor };

	return { ...kept, variants: next, at: next };
}

/** Shows one state alone, in States. */
export function only(search: BenchSearch, id: string): BenchSearch {
	return { ...without(search, ['show', 'args']), variants: id, at: id };
}

/** Takes one state off the stage. The last one showing stays. */
export function hide(ids: readonly string[], search: BenchSearch, id: string): BenchSearch {
	const shown = shownOf(ids, search);
	if (shown.length <= 1 || !shown.includes(id)) return search;

	return {
		...without(search, search.at === id ? ['at'] : []),
		variants: shown.filter(other => other !== id).join(','),
	};
}

/** Every state back on the stage, in States. */
export function all(search: BenchSearch): BenchSearch {
	return without(search, ['show', 'variants', 'args']);
}
