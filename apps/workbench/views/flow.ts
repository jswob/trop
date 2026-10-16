import { destinationFor } from '@app/ui/views/destinations';

import type { Place } from './types';

/**
 * **Where a frame that walks stands** — the arithmetic of `views/flow-frame`, with nothing drawn.
 *
 * A frame is opened on one place, its **home**. A rail press or a register row takes it to another,
 * and the frame's address keeps where (`lib/frame-position`) as one word: a place's id, or the id of
 * a destination nothing is drawn in. No word is home, so a frame that never moves has the address it
 * always had.
 */

/** The place a frame draws, and the destination its rail marks open. `null` is the ground. */
export type Standing = { place: Place; dest: string | null };

const groundOf = (places: readonly Place[]): Place | undefined =>
	places.find(place => place.dest === null);

/**
 * Reads a position back into a standing. A place's id wins over a destination's, since the first
 * place of a destination is usually named after it; a destination's id is its first place. A
 * destination with no place is still somewhere the rail goes: the frame stands on the ground with
 * that destination open, and draws the undrawn page in it. Anything else is home.
 */
export function standingAt(
	position: string | null,
	home: Place,
	places: readonly Place[]
): Standing {
	const rest: Standing = { place: home, dest: home.dest };
	if (position === null) return rest;

	const named = places.find(place => place.id === position);
	if (named !== undefined) return { place: named, dest: named.dest };

	const dest = destinationFor(position);
	if (dest === undefined) return rest;

	const first = places.find(place => place.dest === dest.id);

	return { place: first ?? groundOf(places) ?? home, dest: dest.id };
}

/**
 * The position a move to a destination writes: none for the frame's own, the first place drawn in
 * any other, and the destination's own id where nothing is drawn in it yet.
 */
export function positionOf(
	dest: string | null,
	home: Place,
	places: readonly Place[]
): string | null {
	if (dest === home.dest) return null;

	return places.find(place => place.dest === dest)?.id ?? dest;
}
