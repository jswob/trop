import { describe, expect, it } from 'vitest';

import { DESTINATIONS, hrefDest, hrefNest, nestHref, nestPath, recordHref } from './destinations';

/**
 * **A link on a page is a way to a destination**, and the frame can only guard the ones it can
 * read. These two functions are the whole of that reading, so they are held to each other.
 */
describe('a link, read back as a destination', () => {
	it('reads back every destination a record link is written for', () => {
		for (const dest of DESTINATIONS) expect(hrefDest(recordHref(dest.id, 'r-1'))).toBe(dest.id);
	});

	it('writes a note’s page under Notes, in the one shape the frame reads', () => {
		expect(recordHref('notes', 'n-groceries')).toBe('#notes/n-groceries');
	});

	it('leaves a link to no destination to the browser', () => {
		expect(hrefDest('#note-n-groceries')).toBeNull();
		expect(hrefDest('#projects/x')).toBeNull();
		expect(hrefDest('https://example.org/notes/1')).toBeNull();
		expect(hrefDest('')).toBeNull();
	});
});

/**
 * **A nested page's address is a record address under the page's own destination**, so a frame
 * reads it back as the page you are on, and the nest reads the record out of it.
 */
describe('a nested page, read back', () => {
	it('writes a note inside Notes under Notes', () => {
		expect(nestHref('notes', 'note', 'n-groceries')).toBe('#notes/note/n-groceries');
		expect(hrefDest(nestHref('notes', 'note', 'n-groceries'))).toBe('notes');
	});

	it('reads back every destination, kind and record it is written for', () => {
		for (const dest of DESTINATIONS)
			expect(hrefNest(nestHref(dest.id, 'note', 'n-long-902'))).toEqual({
				dest: dest.id,
				kind: 'note',
				id: 'n-long-902',
			});
	});

	it('reads the app’s address form as it reads the workbench’s', () => {
		expect(nestPath('notes', 'note', 'n-errands')).toBe('/notes/note/n-errands');
		expect(hrefNest(nestPath('notes', 'note', 'a b'))).toEqual({
			dest: 'notes',
			kind: 'note',
			id: 'a b',
		});
		expect(hrefNest('/projects/note/n-errands')).toBeNull();
		expect(hrefNest('/notes')).toBeNull();
	});

	it('is not a record address one level up, and not a link to no destination', () => {
		expect(hrefNest(recordHref('notes', 'n-groceries'))).toBeNull();
		expect(hrefNest('#projects/note/n-groceries')).toBeNull();
		expect(hrefNest('https://example.org/notes/note/1')).toBeNull();
	});
});

/** The rail holds the app's records, then you; what each cell reports is handed in by the caller. */
describe('the rail', () => {
	it('holds Notes, then Account last', () => {
		expect(DESTINATIONS.map(dest => dest.label)).toEqual(['Notes', 'Account']);
	});

	it('carries no fact of its own, so the register prints only what the caller hands it', () => {
		expect(DESTINATIONS.map(dest => dest.fact)).toEqual(['', '']);
	});
});
