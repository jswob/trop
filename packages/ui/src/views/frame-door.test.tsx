import { act, cleanup, fireEvent, render, screen } from '@testing-library/react';
import { Undo2 } from 'lucide-react';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';

import { DESTINATIONS, GROUND, RESTING, type DestId } from './destinations';
import { Frame, createFrameDoor, type FrameDoor } from './frame';
import type { Hold } from './frame-types';

/**
 * The door against the real `Frame`: a press from outside the page — on the ground, say — leaves
 * through the frame's door, which asks first while the page holds anything.
 */

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

const FACTS = Object.fromEntries(DESTINATIONS.map(dest => [dest.id, ''])) as Record<DestId, string>;

function holdOf(): Hold {
	return {
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
}

function Layout({ held, door }: { held: Hold | null; door: FrameDoor }) {
	return (
		<Frame
			place={{ id: 'app', dest: 'notes' }}
			draw={{
				page: <p>The page</p>,
				reading: { place: 'Groceries', fact: '' },
				held: held ?? undefined,
			}}
			destinations={DESTINATIONS}
			home={GROUND}
			resting={RESTING}
			ledger={<p>The ground</p>}
			facts={FACTS}
			groundReading={{ place: GROUND, fact: '' }}
			server={() => () => Promise.resolve('ok')}
			session={{ signedIn: true, signOut: () => undefined }}
			door={door}
		/>
	);
}

describe('the frame’s door', () => {
	it('goes straight through with nothing held', () => {
		const door = createFrameDoor();
		const then = vi.fn();
		render(<Layout held={null} door={door} />);

		act(() => door.go('Errands', then));

		expect(then).toHaveBeenCalledTimes(1);
		expect(screen.queryByRole('button', { name: 'Keep editing' })).toBeNull();
	});

	it('asks first while something is held, naming where the press was going', () => {
		const door = createFrameDoor();
		const then = vi.fn();
		render(<Layout held={holdOf()} door={door} />);

		act(() => door.go('Errands', then));

		expect(then).not.toHaveBeenCalled();
		expect(screen.getAllByText('Going to Errands discards them.').length).toBeGreaterThan(0);

		act(() => {
			fireEvent.click(screen.getAllByRole('button', { name: 'Keep editing' })[0]!);
		});
		expect(then).not.toHaveBeenCalled();

		act(() => door.go('Errands', then));
		act(() => {
			fireEvent.click(screen.getAllByRole('button', { name: 'Discard and go' })[0]!);
		});
		expect(then).toHaveBeenCalledTimes(1);
	});
});
