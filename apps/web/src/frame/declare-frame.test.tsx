import type { SplitWhere } from '@app/ui/kit';
import { DESTINATIONS, GROUND, RESTING, nestHref, type DestId } from '@app/ui/views/destinations';
import { Frame } from '@app/ui/views/frame';
import type { Hold, HoldSource } from '@app/ui/views/frame-types';
import { act, cleanup, fireEvent, render, screen } from '@testing-library/react';
import { Undo2 } from 'lucide-react';
import { useMemo, useState } from 'react';
import { afterEach, beforeAll, describe, expect, it } from 'vitest';

import { DeclareProvider, createDeclareStore, useDeclare, useDeclared } from './declare';

/**
 * The declaration channel against the real `Frame`: what a page declares from under the layout is
 * what the frame draws and guards — as it would be passed in `draw`.
 */

afterEach(cleanup);

/**
 * jsdom has no font set and no Web Animations: the spread fits itself once the faces are in, and
 * the nested page's wipe animates. Both finish at once here.
 */
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

const FACTS = Object.fromEntries(DESTINATIONS.map(dest => [dest.id, ''])) as Record<DestId, string>;

const HELD: Hold = {
	name: 'Groceries',
	fact: '1 unsaved',
	cells: [{ id: 'discard', icon: Undo2, label: 'Discard the changes to Groceries', asks: true }],
	words: {
		stay: 'Keep editing',
		leave: 'Discard and go',
		stop: 'Discard changes',
		loss: () => 'It goes back to how it was saved.',
	},
	nested: true,
};

const holding = (on: boolean): HoldSource => ({
	subscribe: () => () => undefined,
	get: () => (on ? HELD : null),
});

function NotesPage({ holds }: { holds: boolean }) {
	const held = useMemo(() => holding(holds), [holds]);
	useDeclare('notes', {
		nest: {
			kind: 'note',
			name: id => (id === 'n-groceries' ? 'Groceries' : null),
			lede: 'What to buy this week.',
			page: () => <p>Inside Groceries</p>,
		},
		held,
	});
	return <a href={nestHref('notes', 'note', 'n-groceries')}>Groceries</a>;
}

function Layout({ holds = false }: { holds?: boolean }) {
	const [store] = useState(createDeclareStore);
	const [where, setWhere] = useState<SplitWhere>({ ledger: true, dest: 'notes', menu: false });
	const [open, setOpen] = useState<string | null>(null);
	const dest = where.dest as DestId | null;
	const declared = useDeclared(store, dest);
	const page = useMemo(
		() => (dest === 'notes' ? <NotesPage holds={holds} /> : null),
		[dest, holds]
	);
	return (
		<DeclareProvider store={store}>
			<Frame
				place={{ id: 'app', dest }}
				draw={{ page, reading: { place: 'Notes', fact: '' }, ...declared }}
				destinations={DESTINATIONS}
				home={GROUND}
				resting={RESTING}
				ledger={null}
				facts={FACTS}
				groundReading={{ place: 'App', fact: '' }}
				server={() => () => Promise.resolve('ok')}
				control={{ where, onWhere: setWhere }}
				nestControl={{ open, onOpen: setOpen }}
				session={{ signedIn: true, signOut: () => undefined }}
			/>
			<output aria-label="nest">{open ?? 'page'}</output>
		</DeclareProvider>
	);
}

describe('a declared nest', () => {
	it('opens its record on the page’s own link', () => {
		render(<Layout />);

		act(() => {
			fireEvent.click(screen.getAllByRole('link', { name: 'Groceries' })[0]!);
		});

		expect(screen.getByLabelText('nest').textContent).toBe('n-groceries');
		expect(screen.getAllByText('Inside Groceries').length).toBeGreaterThan(0);
	});
});

describe('a declared hold', () => {
	it('stands over the page, so its own discard cell is drawn in the chrome', () => {
		render(<Layout holds />);

		expect(
			screen.getAllByRole('button', { name: 'Discard the changes to Groceries' }).length
		).toBeGreaterThan(0);
	});

	it('asks before a way out of the page', () => {
		render(<Layout holds />);

		act(() => {
			fireEvent.click(screen.getAllByRole('button', { name: /Account/ })[0]!);
		});

		expect(screen.getAllByRole('button', { name: 'Keep editing' }).length).toBeGreaterThan(0);
	});

	it('is gone with its page', () => {
		const { rerender } = render(<Layout holds />);

		rerender(<Layout holds={false} />);

		expect(screen.queryByRole('button', { name: 'Discard the changes to Groceries' })).toBeNull();
	});
});
