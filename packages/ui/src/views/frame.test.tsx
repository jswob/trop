import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';

import { DESTINATIONS, GROUND, RESTING, type DestId, type Destination } from './destinations';
import { Frame } from './frame';

/**
 * The rail draws the destinations its caller hands it and no other, so a destination the workbench
 * stages before the app has a page for it never becomes a cell in the app that leads nowhere.
 */

afterEach(cleanup);

beforeAll(() => {
	Object.defineProperty(document, 'fonts', {
		value: { ready: Promise.resolve() },
		configurable: true,
	});
	if (typeof Element.prototype.animate !== 'function')
		Element.prototype.animate = function animate() {
			return { finished: Promise.resolve(), cancel: () => undefined } as unknown as Animation;
		};
});

const FACTS = Object.fromEntries(DESTINATIONS.map(dest => [dest.id, ''])) as Record<DestId, string>;

function screenAt(wide: boolean) {
	vi.mocked(window).matchMedia.mockImplementation(
		(query: string) =>
			({
				matches: wide && query === '(min-width: 64rem)',
				media: query,
				addEventListener: () => undefined,
				removeEventListener: () => undefined,
			}) as unknown as MediaQueryList
	);
}

function renderFrame(destinations: readonly Destination[]) {
	render(
		<Frame
			place={{ id: 'app', dest: 'notes' }}
			draw={{ page: <p>The page</p>, reading: { place: 'Notes', fact: '' } }}
			destinations={destinations}
			home={GROUND}
			resting={RESTING}
			ledger={<p>The sheet</p>}
			facts={FACTS}
			groundReading={{ place: GROUND, fact: '' }}
			server={() => () => Promise.resolve('ok')}
			session={{ signedIn: true, signOut: () => undefined }}
		/>
	);
}

/** Whether the rail draws a cell for this destination. */
const drawn = (label: string) =>
	screen.queryAllByRole('button', { name: new RegExp(`^${label}\\b`) }).length > 0;

describe.for([
	{ width: 'below lg', wide: false },
	{ width: 'from lg', wide: true },
])('the rail’s cells, $width', ({ wide }) => {
	it('draws a cell for each destination it is handed and none for any other', () => {
		screenAt(wide);
		renderFrame(DESTINATIONS.filter(dest => dest.id !== 'account'));

		expect(drawn('Notes')).toBe(true);
		expect(drawn('Account')).toBe(false);
	});
});
