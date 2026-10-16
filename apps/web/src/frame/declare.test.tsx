import type { DestId } from '@app/ui/views/destinations';
import { act, cleanup, render, screen } from '@testing-library/react';
import { useState } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { DeclareProvider, createDeclareStore, useDeclare, useDeclared } from './declare';
import type { PageDeclared } from './declare';

/**
 * What a page under `/` tells the frame beside its body: the layout mounts the frame once, so a
 * page cannot pass props up to it — it declares, and the layout merges.
 */

afterEach(cleanup);

function Page({ declared }: { declared: PageDeclared }) {
	useDeclare('notes', declared);
	return null;
}

function Layout({ open, children }: { open: DestId | null; children: React.ReactNode }) {
	const [store] = useState(createDeclareStore);
	return (
		<DeclareProvider store={store}>
			<Seen store={store} open={open} />
			{children}
		</DeclareProvider>
	);
}

function Seen({
	store,
	open,
}: {
	store: ReturnType<typeof createDeclareStore>;
	open: DestId | null;
}) {
	const declared = useDeclared(store, open);
	return <output aria-label="declared">{declared.facts?.notes ?? 'none'}</output>;
}

const seen = () => screen.getByLabelText('declared').textContent;

describe('a page’s declaration', () => {
	it('reaches the layout while its destination is the one open', () => {
		render(
			<Layout open="notes">
				<Page declared={{ facts: { notes: '5' } }} />
			</Layout>
		);

		expect(seen()).toBe('5');
	});

	it('follows the page as it changes', () => {
		function Counting() {
			const [n, setN] = useState(1);
			useDeclare('notes', { facts: { notes: String(n) } });
			return (
				<button type="button" onClick={() => setN(n + 1)}>
					more
				</button>
			);
		}
		render(
			<Layout open="notes">
				<Counting />
			</Layout>
		);

		act(() => {
			screen.getByRole('button').click();
		});

		expect(seen()).toBe('2');
	});

	it('is not merged over another destination', () => {
		render(
			<Layout open="account">
				<Page declared={{ facts: { notes: '5' } }} />
			</Layout>
		);

		expect(seen()).toBe('none');
	});

	it('is cleared when the page leaves', () => {
		const { rerender } = render(
			<Layout open="notes">
				<Page declared={{ facts: { notes: '5' } }} />
			</Layout>
		);

		rerender(<Layout open="notes">{null}</Layout>);

		expect(seen()).toBe('none');
	});
});

describe('a declaration that has not changed', () => {
	it('leaves the layout alone, so a page ticking on its own clock never re-renders it', () => {
		const held = { subscribe: () => () => undefined, get: () => null };
		const facts = { notes: '4' };
		const layoutRendered = vi.fn();

		function Counted({ store }: { store: ReturnType<typeof createDeclareStore> }) {
			useDeclared(store, 'notes');
			layoutRendered();
			return null;
		}

		function Ticking({ tick }: { tick: number }) {
			useDeclare('notes', { held, facts });
			return <output aria-label="tick">{tick}</output>;
		}

		function Harness({ tick }: { tick: number }) {
			const [store] = useState(createDeclareStore);
			return (
				<DeclareProvider store={store}>
					<Counted store={store} />
					<Ticking tick={tick} />
				</DeclareProvider>
			);
		}

		const { rerender } = render(<Harness tick={0} />);
		const settled = layoutRendered.mock.calls.length;
		rerender(<Harness tick={1} />);
		rerender(<Harness tick={2} />);

		expect(screen.getByLabelText('tick').textContent).toBe('2');
		expect(layoutRendered.mock.calls.length - settled).toBe(2);
	});
});
