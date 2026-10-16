import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { useState } from 'react';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

import { PlaceholderRows, READ_FAILED, TRY_AGAIN, Waiting, WaitingNote } from '@app/ui/kit';

import { DESTINATIONS, GROUND, RESTING, type DestId } from './destinations';
import { Frame, type FrameReadFailed } from './frame';
import type { SignedOut, ViewDraw } from './frame-types';
import { TRYING_AGAIN } from './read-failed';
import { SignIn } from './sign-in';

/**
 * Failed reads against the real `Frame`: the ground's reads failing puts the centred screen up and
 * the frame away without unmounting it; a page's own failed read is said in its wait; sign-in says
 * an unreachable server under its plate.
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

/** A page with a draft of its own, so a remount would show as the draft gone. */
function Page() {
	const [draft, setDraft] = useState('');
	return (
		<>
			<input aria-label="Draft" value={draft} onChange={event => setDraft(event.target.value)} />
			<Waiting>
				<WaitingNote />
				<PlaceholderRows rows={2} />
			</Waiting>
		</>
	);
}

function Layout({
	readFailed,
	draw = {},
	dest = 'notes',
}: {
	readFailed?: FrameReadFailed | null;
	draw?: Partial<ViewDraw>;
	dest?: DestId | null;
}) {
	return (
		<Frame
			place={{ id: 'app', dest }}
			draw={{ page: <Page />, reading: { place: 'Notes', fact: '' }, ...draw }}
			destinations={DESTINATIONS}
			home={GROUND}
			resting={RESTING}
			ledger={<p>The sheet</p>}
			facts={FACTS}
			groundReading={{ place: GROUND, fact: '' }}
			server={() => () => Promise.resolve('ok')}
			session={{ signedIn: true, signOut: () => undefined }}
			{...(readFailed === undefined ? {} : { readFailed })}
		/>
	);
}

const failure = (over: Partial<FrameReadFailed> = {}): FrameReadFailed => ({
	ground: false,
	retrying: false,
	retry: vi.fn(),
	...over,
});

describe('a read that failed under the whole frame', () => {
	it('puts the centred screen up, and the plate asks again', () => {
		const retry = vi.fn();
		render(<Layout readFailed={failure({ ground: true, retry })} />);

		expect(screen.getByRole('alert').textContent).toBe(READ_FAILED);
		fireEvent.click(screen.getByRole('button', { name: TRY_AGAIN }));
		expect(retry).toHaveBeenCalledTimes(1);
	});

	it('says it is trying again while the reads are asked again', () => {
		render(<Layout readFailed={failure({ ground: true, retrying: true })} />);
		expect(screen.getByRole('button', { name: new RegExp(TRYING_AGAIN) })).toBeTruthy();
	});

	it('keeps the frame mounted under the screen, so a page loses nothing', () => {
		const view = render(<Layout readFailed={null} />);
		fireEvent.change(screen.getByLabelText('Draft'), { target: { value: 'half a note' } });

		view.rerender(<Layout readFailed={failure({ ground: true })} />);
		expect(screen.getByRole('alert').textContent).toBe(READ_FAILED);

		view.rerender(<Layout readFailed={null} />);
		expect(screen.queryByRole('alert')).toBeNull();
		expect(screen.getByLabelText<HTMLInputElement>('Draft').value).toBe('half a note');
	});
});

describe('the keyboard, when the reads come back', () => {
	let waiting: FrameRequestCallback[] = [];
	let drawn: { mockRestore: () => void } | null = null;
	const frame = () => {
		const due = waiting;
		waiting = [];
		for (const run of due) run(0);
	};

	const screenAt = (wide: boolean) => {
		vi.mocked(window).matchMedia.mockImplementation(
			(query: string) =>
				({
					matches: wide && query === '(min-width: 64rem)',
					media: query,
					addEventListener: () => undefined,
					removeEventListener: () => undefined,
				}) as unknown as MediaQueryList
		);
	};

	/** jsdom draws nothing: every element is drawn but what a `hidden` box or the other shell holds. */
	const draws = (wide: boolean) => {
		screenAt(wide);
		const other = wide ? '.lg\\:hidden' : '.lg\\:flex';
		drawn = vi.spyOn(HTMLElement.prototype, 'getClientRects').mockImplementation(function rects(
			this: HTMLElement
		) {
			return (this.closest(`[hidden], ${other}`) === null
				? [new DOMRect()]
				: []) as unknown as DOMRectList;
		});
	};

	const comesBack = (dest: DestId | null) => {
		const view = render(<Layout dest={dest} readFailed={failure({ ground: true })} />);
		screen.getByRole('button', { name: TRY_AGAIN }).focus();
		view.rerender(<Layout dest={dest} readFailed={null} />);
		frame();
	};

	beforeEach(() => {
		waiting = [];
		vi.stubGlobal('requestAnimationFrame', (run: FrameRequestCallback) => waiting.push(run));
		vi.stubGlobal('cancelAnimationFrame', () => {
			waiting = [];
		});
	});

	afterEach(() => {
		vi.unstubAllGlobals();
		drawn?.mockRestore();
		drawn = null;
		screenAt(false);
	});

	it('goes to the rail’s logo on the wide ground, which has no page head to hold it', () => {
		draws(true);
		comesBack(null);

		expect(document.activeElement?.getAttribute('aria-label')).toBe(`Collapse ${GROUND}`);
	});

	it('goes to the page’s head on the wide screen with a page open', () => {
		draws(true);
		comesBack('notes');

		expect(document.activeElement?.tagName).toBe('H1');
		expect(document.activeElement?.closest('[hidden], .lg\\:hidden')).toBeNull();
	});

	it('goes to the bar below the wide screen', () => {
		draws(false);
		comesBack(null);

		expect(document.activeElement?.closest('.lg\\:hidden')).not.toBeNull();
		expect(document.activeElement?.tagName).toBe('BUTTON');
	});
});

describe('a page’s own read that failed', () => {
	it('is said in the page’s wait, with the frame still standing', () => {
		const retry = vi.fn();
		render(<Layout readFailed={failure({ retry })} />);

		expect(screen.queryByRole('alert')).toBeNull();
		const note = screen.getAllByRole('status').find(node => node.textContent.includes(READ_FAILED));
		expect(note).toBeDefined();
		fireEvent.click(screen.getAllByRole('button', { name: TRY_AGAIN })[0]!);
		expect(retry).toHaveBeenCalledTimes(1);
	});

	it('leaves every wait loading while nothing has failed', () => {
		render(<Layout readFailed={null} />);
		expect(screen.queryByText(READ_FAILED)).toBeNull();
	});
});

describe('a listed failed read, as the workbench draws it', () => {
	it('opens on the screen, and Try again clears it', () => {
		render(<Layout draw={{ readFailed: 'ground' }} />);

		expect(screen.getByRole('alert').textContent).toBe(READ_FAILED);
		fireEvent.click(screen.getByRole('button', { name: TRY_AGAIN }));
		expect(screen.queryByRole('alert')).toBeNull();
	});
});

describe('sign-in with a server that never answered', () => {
	it('says so under the plate, and never that the password was wrong', async () => {
		const opens: SignedOut = 'empty';
		render(
			<SignIn
				opens={opens}
				server={() => Promise.reject(new Error('unreachable'))}
				onSignedIn={() => undefined}
			/>
		);
		fireEvent.change(screen.getByLabelText('Password'), { target: { value: 'hunter2' } });
		fireEvent.click(screen.getByRole('button', { name: 'Sign in' }));

		expect((await screen.findByRole('alert')).textContent).toContain(READ_FAILED);
		expect(screen.queryByText(/isn’t the password/)).toBeNull();
	});

	it('opens in the condition when it is listed', () => {
		render(
			<SignIn
				opens="unreachable"
				server={() => Promise.resolve('ok')}
				onSignedIn={() => undefined}
			/>
		);
		expect(screen.getByRole('alert').textContent).toContain(READ_FAILED);
	});
});
