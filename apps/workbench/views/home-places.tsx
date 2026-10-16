import { RESTING_READING } from '../fixtures/rail-facts';
import type { Place } from './types';

/**
 * **The ground**: the app's home, with nothing open over it.
 *
 * Its page is the resting ledger the frame is handed (`views/place-frame`), the home page that
 * names the app and says what each destination is for, so a variant declares only what the slug
 * prints and whether the register is raised.
 *
 * `dest: null` — nothing is open over it, so the rail marks no cell and the logo carries the mark
 * instead. On a phone it is where the app lands, which is why the raised register is listed here
 * and on no destination.
 */
export const home: Place = {
	id: 'home',
	dest: null,
	title: 'Home',
	description: 'The ground: the app’s name, and what each destination is for.',
	variants: [
		{
			id: 'home',
			label: 'At rest',
			note: 'The home page as the app opens on it.',
			draw: () => ({ reading: RESTING_READING }),
		},
		{
			id: 'menu',
			label: 'The register, raised',
			note: 'Below lg: the destinations raised from the slug.',
			draw: () => ({ menu: true, reading: RESTING_READING }),
		},
	],
};

export const HOME_PLACES: Place[] = [home];
