import { Frame } from '@app/ui/views/frame';
import { act, cleanup, fireEvent, render, screen } from '@testing-library/react';
import { useEffect, useState } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { DESTINATIONS, GROUND, RESTING, type DestId } from '@app/ui/views/destinations';

/**
 * The frame mounts the open page once, whichever setting draws it: the logo is free, so
 * collapsing and restoring the ground must never remount the page and lose what it holds.
 */

const FACTS = Object.fromEntries(DESTINATIONS.map(dest => [dest.id, ''])) as Record<DestId, string>;

let mounts = 0;
/** Wipes still running, each settled by hand. */
let wipes: (() => void)[] = [];

function Page() {
	const [count, setCount] = useState(0);
	useEffect(() => {
		mounts += 1;
	}, []);
	return (
		<button
			type="button"
			onClick={() => {
				setCount(was => was + 1);
			}}
		>
			{`Pressed ${String(count)}`}
		</button>
	);
}

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

function renderFrame() {
	render(
		<Frame
			place={{ id: 'app', dest: 'notes' }}
			draw={{ page: <Page />, reading: { place: 'Notes', fact: '' } }}
			destinations={DESTINATIONS}
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

async function pressLogo(name: string) {
	await act(async () => {
		fireEvent.click(screen.getAllByRole('button', { name })[0]!);
		await Promise.resolve();
	});
}

/** The page's one live element, and whatever copies stand beside it. */
const pages = () => document.querySelectorAll('[data-ui-page-host] button');
const live = () => screen.getByRole('button', { name: /Pressed/ });

beforeEach(() => {
	mounts = 0;
	wipes = [];
	Object.defineProperty(document, 'fonts', {
		value: { ready: Promise.resolve() },
		configurable: true,
	});
	Element.prototype.animate = function animate() {
		const finished = new Promise<void>(resolve => {
			wipes.push(resolve);
		});
		return { finished, cancel: () => undefined } as unknown as Animation;
	};
});

afterEach(cleanup);

describe.for([
	{ width: 'below lg', wide: false },
	{ width: 'from lg', wide: true },
])('the open page, $width', ({ wide }) => {
	it('is mounted once, and keeps its state through a collapse and a restore', async () => {
		screenAt(wide);
		renderFrame();
		fireEvent.click(live());
		expect(mounts).toBe(1);

		await pressLogo(`Collapse ${GROUND}`);
		await act(async () => {
			wipes.forEach(finish => {
				finish();
			});
			await Promise.resolve();
		});
		await pressLogo(`Expand ${GROUND}`);

		expect(mounts).toBe(1);
		expect(live().textContent).toBe('Pressed 1');
	});

	it('draws the sheet once, in the shell on screen', () => {
		screenAt(wide);
		renderFrame();

		const sheets = screen.getAllByText('The sheet');
		expect(sheets).toHaveLength(1);
		expect(sheets[0]!.closest('.lg\\:hidden') === null).toBe(wide);
	});
});

describe('the wide screen', () => {
	it('moves the page into the two-column setting, leaving the one it takes away an inert copy', async () => {
		screenAt(true);
		renderFrame();
		expect(pages()).toHaveLength(1);

		await pressLogo(`Collapse ${GROUND}`);

		const drawn = Array.from(pages());
		expect(drawn).toHaveLength(2);
		const [copy, page] = drawn;
		expect(page).toBe(live());
		expect(copy!.closest('[inert]')).not.toBeNull();

		await act(async () => {
			wipes.forEach(finish => {
				finish();
			});
			await Promise.resolve();
		});
		expect(pages()).toHaveLength(1);
		expect(pages()[0]).toBe(page);
	});
});
