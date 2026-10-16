import { describe, expect, it } from 'vitest';

import { positionOf, standingAt } from './flow';
import type { Place } from './types';

const place = (id: string, dest: Place['dest']): Place => ({
	id,
	dest,
	title: id,
	description: '',
	variants: [],
});

const GROUND = place('home', null);
const DRAFT = place('note-draft', 'notes');
const NOTES = place('notes', 'notes');
const SIGNED_IN = place('account-signed-in', 'account');
const SIGNED_OUT = place('account-signed-out', 'account');

/** Account is a destination the rail offers and this list draws nothing in. */
const PLACES = [GROUND, DRAFT, NOTES];

/** The same space with Account drawn, in two places of which neither is named after it. */
const WITH_ACCOUNT = [...PLACES, SIGNED_IN, SIGNED_OUT];

describe('where a walked frame stands', () => {
	it('is its own place while the address names no position', () => {
		expect(standingAt(null, NOTES, PLACES)).toEqual({ place: NOTES, dest: 'notes' });
	});

	it('is the place the address names', () => {
		expect(standingAt('note-draft', NOTES, PLACES)).toEqual({ place: DRAFT, dest: 'notes' });
	});

	it('is the ground, with nothing open over it', () => {
		expect(standingAt('home', NOTES, PLACES)).toEqual({ place: GROUND, dest: null });
	});

	it('is its own place when the address names nothing the workbench has', () => {
		expect(standingAt('not-a-place', NOTES, PLACES)).toEqual({ place: NOTES, dest: 'notes' });
	});

	it('is the ground under a destination nothing is drawn in, which the rail still opens', () => {
		expect(standingAt('account', NOTES, PLACES)).toEqual({ place: GROUND, dest: 'account' });
	});

	it('is the first place of a destination the address names', () => {
		expect(standingAt('account', NOTES, WITH_ACCOUNT)).toEqual({
			place: SIGNED_IN,
			dest: 'account',
		});
	});

	it('is a place before it is a destination, where one name is both', () => {
		expect(standingAt('notes', DRAFT, PLACES)).toEqual({ place: NOTES, dest: 'notes' });
	});
});

describe('the position a move to a destination writes', () => {
	it('is none for the frame’s own destination, which is its own place again', () => {
		expect(positionOf('notes', NOTES, PLACES)).toBeNull();
	});

	it('is the first place drawn in another destination', () => {
		expect(positionOf('account', NOTES, WITH_ACCOUNT)).toBe('account-signed-in');
	});

	it('is the ground’s place when the page closes', () => {
		expect(positionOf(null, NOTES, PLACES)).toBe('home');
	});

	it('is none for the ground when the ground is the frame’s own place', () => {
		expect(positionOf(null, GROUND, PLACES)).toBeNull();
	});

	it('is the destination itself where nothing is drawn in it', () => {
		expect(positionOf('account', NOTES, PLACES)).toBe('account');
	});

	it('reads back as the standing it was written for', () => {
		for (const places of [PLACES, WITH_ACCOUNT]) {
			for (const dest of [null, 'notes', 'account']) {
				const standing = standingAt(positionOf(dest, SIGNED_OUT, places), SIGNED_OUT, places);

				expect(standing.dest).toBe(dest);
			}
		}
	});
});
