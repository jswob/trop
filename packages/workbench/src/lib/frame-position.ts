import { useSyncExternalStore } from 'react';

/**
 * The one search param a frame may keep its position in.
 *
 * Not `at`: the bench's stage already stands on that, and the two would be read as one. What the
 * value means is the frame's own business. A place keeps the place it has walked to
 * (`views/flow-frame`); a frame that never moves never writes it, so its address is unchanged.
 */
export const POSITION_PARAM = 'where';

const readers = new Set<() => void>();

/** What could not be written to the address, kept so the frame still moves. */
let unwritten: { value: string | null } | null = null;

/** The position a search string names: how the bench reads a frame that is another document. */
export function positionIn(search: string): string | null {
	return new URLSearchParams(search).get(POSITION_PARAM);
}

/** Where the frame's address says it is, or `null` when it names nowhere. */
export function readPosition(): string | null {
	if (unwritten !== null) return unwritten.value;

	return positionIn(window.location.search);
}

/**
 * Moves the frame's address to a position, in place.
 *
 * The entry is replaced and its state kept, so the document is not loaded again, a router reading
 * the same history sees the entry it wrote, and the browser's back still leaves the frame rather
 * than stepping through it. A fragment goes with the move: it was the page that was left saying
 * which record it had open. A document whose address cannot be written, a board's frame in a
 * sandbox, keeps the position in memory and loses it on a reload.
 */
export function writePosition(next: string | null): void {
	const url = new URL(window.location.href);

	if (next === null) url.searchParams.delete(POSITION_PARAM);
	else url.searchParams.set(POSITION_PARAM, next);
	url.hash = '';

	try {
		window.history.replaceState(window.history.state, '', url);
		unwritten = null;
	} catch {
		unwritten = { value: next };
	}

	for (const tell of readers) tell();
}

function subscribe(tell: () => void): () => void {
	readers.add(tell);
	window.addEventListener('popstate', tell);

	return () => {
		readers.delete(tell);
		window.removeEventListener('popstate', tell);
	};
}

/**
 * A frame's position, kept in its own address: what it is, and the way to move it.
 *
 * Reading it re-renders the reader on every move and remounts nothing, which is the point: a frame
 * that walked from one page to another is still the frame that was opened. Because it is in the
 * address, a reload and the bench's `Open` find the frame where it was left.
 */
export function useFramePosition(): [string | null, (next: string | null) => void] {
	return [useSyncExternalStore(subscribe, readPosition), writePosition];
}
