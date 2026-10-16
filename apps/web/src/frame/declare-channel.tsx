import type { PageHeading, RailReading } from '@app/ui/kit';
import type { Detour, HoldSource, Live, Nest, PageErrand } from '@app/ui/views/frame-types';
import type { ReactNode } from 'react';
import { createContext, useContext, useLayoutEffect, useSyncExternalStore } from 'react';

/**
 * **What a page under `/` declares to the frame beside its body.**
 *
 * The layout mounts the one `Frame` and passes it `draw`, and the page is only its `Outlet`,
 * so a page cannot hand the frame its nest, its hold or its confirm as props. It declares them
 * here and the layout merges them into `draw` — for the page's own destination only, and never after
 * the page has gone.
 *
 * These are the fields every frame reads. An app adds its own beside them (`frame/declare`'s
 * `PageDeclared`) and binds the channel to the whole with {@link declareChannel}.
 *
 * A hold is declared live (`HoldSource`): the frame reads a plain one once.
 */
export type FrameDeclared = {
	reading?: RailReading;
	errand?: PageErrand;
	facts?: Partial<Record<string, string>>;
	nest?: Nest;
	held?: HoldSource;
	overlay?: ReactNode;
	/**
	 * The page's own head, where one destination is two errands — `Add a note` and the note's own
	 * title. Absent, the frame heads the page from the rail.
	 */
	head?: PageHeading;
	/** Whether the page is a stage right now, centred with no head: something to watch, not read. */
	centred?: Live<boolean>;
	/** How a phone reaches the ground with the page still open, for one errand there. */
	detour?: Detour;
};

type Entry<D> = { dest: string; declared: D };

/** Where the open page's declaration is kept between the page and the layout. */
export type DeclareStore<D> = {
	get: () => Entry<D> | null;
	subscribe: (listener: () => void) => () => void;
	set: (entry: Entry<D> | null) => void;
};

/** Whether two declarations hold the same values, each compared by identity. */
function sameDeclared(a: object, b: object): boolean {
	const keys = Object.keys(a) as (keyof typeof a)[];
	return keys.length === Object.keys(b).length && keys.every(key => Object.is(a[key], b[key]));
}

/**
 * **The channel a page declares through, for one shape of declaration.** An app calls it once, with
 * its {@link FrameDeclared} and whatever it adds, and exports what it returns.
 */
export function declareChannel<D extends FrameDeclared>() {
	const DeclareContext = createContext<DeclareStore<D> | null>(null);
	const NOTHING = {} as D;

	function createDeclareStore(): DeclareStore<D> {
		let entry: Entry<D> | null = null;
		const listeners = new Set<() => void>();
		return {
			get: () => entry,
			subscribe: listener => {
				listeners.add(listener);
				return () => {
					listeners.delete(listener);
				};
			},
			set: next => {
				entry = next;
				for (const listener of listeners) listener();
			},
		};
	}

	/** The layout's store, reached by every page under it. The store never changes identity. */
	function DeclareProvider({ store, children }: { store: DeclareStore<D>; children: ReactNode }) {
		return <DeclareContext value={store}>{children}</DeclareContext>;
	}

	/**
	 * What the page on `open` has declared, for the layout to merge into `draw` — nothing while
	 * another destination, or the ground, is open.
	 */
	function useDeclared(store: DeclareStore<D>, open: string | null): D {
		const entry = useSyncExternalStore(store.subscribe, store.get);
		return entry !== null && entry.dest === open ? entry.declared : NOTHING;
	}

	/**
	 * **Declare what this page tells the frame**, again on every render so the frame follows it. The
	 * layout's re-render does not render the page again (the `Outlet` is memoised), so this cannot
	 * loop. Leaving the page clears it.
	 *
	 * A render that declares the same values as the last — each compared by identity — leaves the
	 * layout alone, so a page that re-renders on its own clock (a solve run's) does not re-render the
	 * frame and its sheet every tick. A page that wants that keeps what it declares stable.
	 * @param dest - The destination this page is
	 * @param declared - What the frame should draw of it
	 */
	function useDeclare(dest: string, declared: D): void {
		const store = useContext(DeclareContext);

		useLayoutEffect(() => {
			const current = store?.get();
			if (current?.dest === dest && sameDeclared(current.declared, declared)) return;
			store?.set({ dest, declared });
		});

		useLayoutEffect(
			() => () => {
				if (store?.get()?.dest === dest) store.set(null);
			},
			[store, dest]
		);
	}

	return { createDeclareStore, DeclareProvider, useDeclared, useDeclare };
}
