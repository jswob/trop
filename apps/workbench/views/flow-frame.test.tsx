import { DESTINATIONS, type DestId } from '@app/ui/views/destinations';
import { readPosition, writePosition } from '@jswob/design-workbench';
import { act, cleanup, fireEvent, render, screen } from '@testing-library/react';
import { Undo2 } from 'lucide-react';
import { useEffect, useState } from 'react';
import { afterEach, beforeAll, beforeEach, describe, expect, it } from 'vitest';

import { FlowFrame } from './flow-frame';
import type { Hold, Place, PlaceVariant, ViewDraw } from './types';

/**
 * A place's frame, walked, against the real `Frame`: a rail press lands on a drawn place, the
 * address keeps where, and it is one frame from the first press to the last.
 */

const FRAME = '/frame/view/notes?variant=notes&chrome=bare';

afterEach(cleanup);

beforeAll(() => {
	if (!('fonts' in document))
		Object.defineProperty(document, 'fonts', {
			value: { ready: Promise.resolve() },
			configurable: true,
		});
	if (typeof Element.prototype.animate !== 'function')
		Element.prototype.animate = function animate() {
			return {
				finished: Promise.resolve(),
				cancel: () => undefined,
				finish: () => undefined,
				onfinish: null,
			} as unknown as Animation;
		};
});

let ledgerMounts = 0;

beforeEach(() => {
	ledgerMounts = 0;
	writePosition(null);
	window.history.replaceState(null, '', FRAME);
});

const FACTS = Object.fromEntries(DESTINATIONS.map(dest => [dest.id, ''])) as Record<DestId, string>;

/** The resting ledger, as far as a walk can tell: mounted once. */
function Ledger() {
	useEffect(() => {
		ledgerMounts += 1;
	}, []);

	return <p>The resting ledger</p>;
}

/** A page with something of its own to lose, so a page mounted again is told from one kept. */
function Counted({ of }: { of: string }) {
	const [count, setCount] = useState(0);

	return (
		<button
			type="button"
			onClick={() => {
				setCount(was => was + 1);
			}}
		>
			{`${of} pressed ${String(count)}`}
		</button>
	);
}

const HOLD: Hold = {
	name: 'Groceries',
	fact: '1 unsaved',
	cells: [{ id: 'discard', icon: Undo2, label: 'Discard the changes', asks: true }],
	words: {
		stay: 'Keep editing',
		leave: 'Discard and go',
		stop: 'Discard changes',
		loss: going => `Going to ${String(going)} discards them.`,
	},
};

const state = (id: string, draw: () => ViewDraw): PlaceVariant => ({
	id,
	label: id,
	note: '',
	draw,
});

const reading = { place: 'A place', fact: '' };

const GROUND: Place = {
	id: 'home',
	dest: null,
	title: 'Home',
	description: '',
	variants: [state('home', () => ({ reading, ledger: <p>The ground’s own ledger</p> }))],
};

const NOTES: Place = {
	id: 'notes',
	dest: 'notes',
	title: 'Notes',
	description: '',
	variants: [
		state('notes', () => ({ reading, page: <Counted of="Notes" /> })),
		state('long', () => ({ reading, page: <p>The long list</p> })),
		state('holding', () => ({ reading, page: <p>A note, changed</p>, held: HOLD })),
	],
};

const ACCOUNT: Place = {
	id: 'account',
	dest: 'account',
	title: 'Account',
	description: '',
	variants: [
		state('account', () => ({ reading, page: <Counted of="Account" /> })),
		state('signed-out', () => ({ reading, page: <p>Signed out</p> })),
	],
};

const PLACES = [GROUND, NOTES, ACCOUNT];

function renderFlow(variant = 'notes', places: readonly Place[] = PLACES) {
	return render(
		<FlowFrame
			home={NOTES}
			state={NOTES.variants.find(one => one.id === variant)!}
			places={places}
			ledger={<Ledger />}
			facts={FACTS}
			groundReading={{ place: 'App', fact: '' }}
			server={() => () => Promise.resolve('ok')}
		/>
	);
}

async function press(name: string | RegExp) {
	await act(async () => {
		fireEvent.click(screen.getAllByRole('button', { name })[0]!);
		await Promise.resolve();
	});
}

const drawn = (text: string | RegExp): boolean => screen.queryAllByText(text).length > 0;

describe('a place’s frame at rest', () => {
	it('draws its own place in the state it was opened on, and names no position', () => {
		renderFlow();

		expect(drawn('Notes pressed 0')).toBe(true);
		expect(readPosition()).toBeNull();
		expect(window.location.pathname + window.location.search).toBe(FRAME);
	});
});

describe('a rail press', () => {
	it('lands on the first state of the place drawn in that destination', async () => {
		renderFlow();

		await press('Account');

		expect(drawn('Account pressed 0')).toBe(true);
		expect(drawn('Signed out')).toBe(false);
		expect(drawn(/^Notes pressed/)).toBe(false);
	});

	it('writes where it landed onto the frame’s own address', async () => {
		renderFlow();

		await press('Account');

		expect(readPosition()).toBe('account');
		expect(window.location.pathname).toBe('/frame/view/notes');
		expect(new URLSearchParams(window.location.search).get('variant')).toBe('notes');
	});

	it('comes home on the frame’s own destination, to the state it was opened on', async () => {
		renderFlow('long');

		await press('Account');
		await press('Notes');

		expect(drawn('The long list')).toBe(true);
		expect(readPosition()).toBeNull();
	});

	it('stands on the ground’s own place when it closes the page', async () => {
		renderFlow();

		await press('Notes');

		expect(drawn('The ground’s own ledger')).toBe(true);
		expect(readPosition()).toBe('home');
	});

	it('draws the waiting page in a destination nothing is drawn in', async () => {
		renderFlow('notes', [GROUND, NOTES]);

		await press('Account');

		expect(readPosition()).toBe('account');
		expect(drawn(/pressed/)).toBe(false);
		expect(drawn('The ground’s own ledger')).toBe(true);
	});

	it('hands the page it lands on nothing the page it left was holding', async () => {
		renderFlow();
		await press('Notes pressed 0');
		expect(drawn('Notes pressed 1')).toBe(true);

		await press('Account');

		expect(drawn('Account pressed 0')).toBe(true);
	});

	it('moves one frame: the ledger beside the page is never mounted again', async () => {
		renderFlow();

		await press('Account');
		await press('Notes');
		await press('Account');

		expect(ledgerMounts).toBe(1);
	});

	it('asks first while the page holds something, and goes on the discard', async () => {
		renderFlow('holding');

		await press('Account');

		expect(drawn('Going to Account discards them.')).toBe(true);
		expect(readPosition()).toBeNull();

		await press('Discard and go');

		expect(drawn('Account pressed 0')).toBe(true);
		expect(readPosition()).toBe('account');
	});

	it('stays where it is while the page keeps what it holds', async () => {
		renderFlow('holding');

		await press('Account');
		await press('Keep editing');

		expect(drawn('A note, changed')).toBe(true);
		expect(readPosition()).toBeNull();
	});
});

describe('an address that names a position', () => {
	it('opens the frame where it was left', () => {
		window.history.replaceState(null, '', `${FRAME}&where=account`);

		renderFlow();

		expect(drawn('Account pressed 0')).toBe(true);
		expect(drawn(/^Notes pressed/)).toBe(false);
	});

	it('opens the frame at home when the position names nothing', () => {
		window.history.replaceState(null, '', `${FRAME}&where=nowhere`);

		renderFlow();

		expect(drawn('Notes pressed 0')).toBe(true);
	});
});
