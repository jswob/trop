import { createContext, useContext, useEffect, useLayoutEffect, useRef, useState } from 'react';
import type { ReactNode } from 'react';

import { hrefNest, nestHref } from '@app/ui/views/destinations';
import type { Nest } from '@app/ui/views/frame-types';

/**
 * **A page opening a record inside itself** — the behaviour that goes with `kit/nested`'s
 * `NestedPage`. The kit draws the step and the level row; this is which record is open, how one is
 * opened and closed, and what the browser's history says about it.
 *
 * - A link on the page written in `nestHref` opens its record, and the press is free: it never
 *   asks, and a hold standing over the page stands over the nested page too.
 * - The record belongs to the page: leaving the page lets it go, and coming back finds the page.
 * - Opening one puts it on the browser's history, so back — the browser's, or a phone's gesture —
 *   returns to the page, and the frame's own ways back go through that same entry.
 * - Closing puts the keyboard back on whatever opened it: the name in the list.
 */

/** A record open as a nested page, as the page and its ledger read it. */
export type Nested = { id: string; name: string };

const NestContext = createContext<Nested | null>(null);

/** Which record the page has open inside itself, for whatever is drawn beside it. */
export const NestProvider = NestContext.Provider;

/**
 * **The record the page has open inside itself**, or `null` while the page itself is on screen —
 * how the ledger marks the record a page has open.
 */
export function useNested(): Nested | null {
	return useContext(NestContext);
}

/**
 * **The open record, where the caller keeps it** — the app's address. Given, the nest holds
 * no record of its own and writes no history: opening and closing are reported, and the caller's
 * router puts them on the address, so back and forward are the router's.
 */
export type NestControl = { open: string | null; onOpen: (id: string | null) => void };

/** What a history entry this nest pushed carries: the record it opened. */
function nestOf(state: unknown): unknown {
	return typeof state === 'object' && state !== null ? (state as { nest?: unknown }).nest : null;
}

/**
 * **Which record the page has open inside itself, and every way in and out of it.**
 *
 * `page` is the destination whose page declares the nest, and `open` the destination the frame has
 * open: a record is only ever open while they are the same. `nested` is the record a listed state
 * opens on (`ViewDraw.nested`); it is never on the browser's history, so the way back simply
 * lets it go. The result's `follow` is the frame's link press for a nested address — it returns
 * whether it took the press. `control` hands the record to the caller (`NestControl`).
 */
export function useNest(
	nest: Nest | undefined,
	page: string | null,
	open: string | null,
	nested?: string,
	control?: NestControl
) {
	const here = open !== null && open === page;

	const [ownLeaf, setOwnLeaf] = useState<string | null>(nested ?? null);
	const [leafDest, setLeafDest] = useState(open);
	if (leafDest !== open) {
		setLeafDest(open);
		if (ownLeaf !== null) setOwnLeaf(null);
	}
	const leaf = control === undefined ? ownLeaf : control.open;
	const setLeaf = control === undefined ? setOwnLeaf : control.onOpen;
	const name = leaf === null || nest === undefined ? null : nest.name(leaf);
	const shown: Nested | null = here && leaf !== null && name !== null ? { id: leaf, name } : null;

	/** Whether the open record was put on the browser's history by a press here. */
	const pushed = useRef(false);
	/** What opened it, so the way back returns the keyboard there. */
	const opener = useRef<HTMLElement | null>(null);

	const openLeaf = (id: string) => {
		if (page === null || nest === undefined) return;
		opener.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
		setLeaf(id);
		if (control !== undefined) return;
		const state: unknown = window.history.state;
		window.history.pushState(
			{ ...(typeof state === 'object' ? state : {}), nest: id },
			'',
			nestHref(page, nest.kind, id)
		);
		pushed.current = true;
	};

	/** Back to the page: the browser's own back where a press here put the entry on its history. */
	const close = () => {
		if (control !== undefined) {
			setLeaf(null);
			return;
		}
		if (pushed.current && nestOf(window.history.state) === leaf) {
			window.history.back();
			return;
		}
		setLeaf(null);
	};

	/** A link on the page to a record it opens inside itself is a nested press: free, never asked. */
	const follow = (href: string): boolean => {
		const inner = hrefNest(href);
		if (
			inner === null ||
			nest === undefined ||
			!here ||
			inner.dest !== page ||
			inner.kind !== nest.kind ||
			nest.name(inner.id) === null
		)
			return false;
		openLeaf(inner.id);
		return true;
	};

	/** The browser's back and forward, and a phone's back gesture, move between page and nested. */
	const controlled = control !== undefined;
	useEffect(() => {
		if (nest === undefined || controlled) return;
		const onPop = (event: PopStateEvent) => {
			const id = nestOf(event.state);
			const next = typeof id === 'string' && nest.name(id) !== null ? id : null;
			pushed.current = next !== null;
			setOwnLeaf(next);
		};
		window.addEventListener('popstate', onPop);
		return () => {
			window.removeEventListener('popstate', onPop);
		};
	}, [nest, controlled]);

	/**
	 * Leaving the page lets its nested page go, and the address with it: a hash still naming a
	 * record over its list is an address that lies about where you are.
	 */
	useEffect(() => {
		if (nest === undefined || page === null || here || controlled) return;
		if (!window.location.hash.startsWith(nestHref(page, nest.kind, ''))) return;
		const state: unknown = window.history.state;
		const rest =
			typeof state === 'object' && state !== null
				? Object.fromEntries(Object.entries(state).filter(([key]) => key !== 'nest'))
				: state;
		window.history.replaceState(rest, '', window.location.pathname + window.location.search);
		pushed.current = false;
	}, [nest, here, page, controlled]);

	/** Closing puts the keyboard back on whatever opened it — the name in the list. */
	const wasLeaf = useRef(leaf);
	useEffect(() => {
		const was = wasLeaf.current;
		wasLeaf.current = leaf;
		if (was === null || leaf !== null) return;
		const back = opener.current;
		opener.current = null;
		requestAnimationFrame(() => {
			back?.focus();
		});
	}, [leaf]);

	return { open: shown, follow, close };
}

/** The scrolling box an element stands in. */
function scrollerAround(element: HTMLElement | null): HTMLElement | null {
	let node = element?.parentElement ?? null;
	while (node !== null) {
		const overflow = getComputedStyle(node).overflowY;
		if (overflow === 'auto' || overflow === 'scroll') return node;
		node = node.parentElement;
	}
	return null;
}

/**
 * **The page and its nested page, one of them on screen.**
 *
 * The page stays mounted while a nested one is open — `hidden`, not gone — so coming back finds it
 * exactly as it was left: its search, its sort, its page of the list. The scroll is the one thing
 * `hidden` cannot keep, because the page and the nested one share a scroller, so it is remembered
 * while the page is up and put back when it is up again; a nested page opens at its top.
 */
export function NestSwitch({
	open,
	parent,
	nested,
}: {
	open: string | null;
	parent: ReactNode;
	nested: ReactNode;
}) {
	const box = useRef<HTMLDivElement | null>(null);
	const kept = useRef(0);
	/**
	 * Read by the listener rather than closed over: the scroller clamps to 0 the moment the page is
	 * hidden, and that scroll arrives before the listener is taken off (a kept 293 became 0 at lg).
	 */
	const opened = useRef(open);
	useLayoutEffect(() => {
		opened.current = open;
	}, [open]);

	useEffect(() => {
		if (open !== null) return;
		const scroller = scrollerAround(box.current);
		if (scroller === null) return;
		const keep = () => {
			if (opened.current !== null) return;
			kept.current = scroller.scrollTop;
		};
		scroller.addEventListener('scroll', keep, { passive: true });
		return () => {
			scroller.removeEventListener('scroll', keep);
		};
	}, [open]);

	/**
	 * Put back once the page has been laid out again. The spread re-measures its own height in a
	 * layout effect that runs *after* this one (a parent's runs after its child's), so a scroll set
	 * here alone lands on the nested page's short height and clamps to 0.
	 */
	useLayoutEffect(() => {
		const scroller = scrollerAround(box.current);
		if (scroller === null) return;
		if (open !== null) {
			scroller.scrollTop = 0;
			return;
		}
		const top = kept.current;
		scroller.scrollTop = top;
		const frame = requestAnimationFrame(() => {
			scroller.scrollTop = top;
		});
		return () => {
			cancelAnimationFrame(frame);
		};
	}, [open]);

	return (
		<>
			<div ref={box} hidden={open !== null}>
				{parent}
			</div>
			{open === null ? null : nested}
		</>
	);
}
