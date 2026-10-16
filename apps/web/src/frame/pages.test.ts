import { readdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { DESTINATIONS } from '@app/ui/views/destinations';
import { describe, expect, it } from 'vitest';

import { APP_DESTINATIONS, destOfPath, pageDeclaration } from './pages';

const ROUTES = resolve(dirname(fileURLToPath(import.meta.url)), '../routes');

const FACTS = { notes: '3', account: '' };
const GROUND = { place: 'App', fact: '' };

describe('APP_DESTINATIONS', () => {
	it('is every destination the app has a route for and no other, in the rail’s order', () => {
		const routed = new Set(readdirSync(ROUTES));

		expect(APP_DESTINATIONS.map(dest => dest.id)).toEqual(
			DESTINATIONS.map(dest => dest.id).filter(id => routed.has(`${id}.tsx`))
		);
	});
});

describe('destOfPath', () => {
	it('reads the destination an address is on, and the ground as none', () => {
		expect(destOfPath('/')).toBeNull();
		expect(destOfPath('/notes')).toBe('notes');
		expect(destOfPath('/notes/note/n-1')).toBe('notes');
		expect(destOfPath('/account')).toBe('account');
	});

	it('reads anything that is not a destination as the ground', () => {
		expect(destOfPath('/enable')).toBeNull();
		expect(destOfPath('/settings')).toBeNull();
	});
});

describe('pageDeclaration', () => {
	it('prints the ground’s own reading on the ground', () => {
		expect(pageDeclaration(null, FACTS, GROUND)).toEqual({ reading: GROUND });
	});

	it('gives Account its Sign out and no fact', () => {
		expect(pageDeclaration('account', FACTS, GROUND)).toMatchObject({
			reading: { place: 'Account', fact: '' },
			errand: { label: 'Sign out', press: 'sign-out' },
		});
	});

	it('gives a page with nothing of its own to say its cell’s name and fact', () => {
		expect(pageDeclaration('notes', FACTS, GROUND)).toEqual({
			reading: { place: 'Notes', fact: FACTS.notes },
		});
	});

	it('prints an empty fact for a cell nobody gave one', () => {
		expect(pageDeclaration('notes', {}, GROUND)).toEqual({ reading: { place: 'Notes', fact: '' } });
	});
});
