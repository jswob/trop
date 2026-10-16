import { DESTINATIONS } from '@app/ui/views/destinations';
import type { DestId } from '@app/ui/views/destinations';
import { ACCOUNT_PLACES } from './account-places';
import { HOME_PLACES } from './home-places';
import { NOTES_PLACES } from './notes-places';
import { framePlace } from './place-frame';
import type { Place, View } from './types';

/**
 * Every settled place, framed.
 *
 * A **view is a place you can be** — the ground, or one destination over it — and `/views` holds
 * only places that are settled. Anything still being argued about lives in `/proto`, anything
 * reusable lives in `/kit`.
 *
 * A place is one **errand** rather than one destination, so a destination may hold more than one,
 * and a state earns an entry only when no press reaches it. A state of the ledger belongs to the
 * page that causes it.
 *
 * Ordered by the rail, the ground first, and within a destination by the errand you would meet
 * first.
 */

export const PLACES: Place[] = [...HOME_PLACES, ...NOTES_PLACES, ...ACCOUNT_PLACES];

export const VIEWS: View[] = PLACES.map(place => framePlace(place, PLACES));

export function viewById(id: string): View | undefined {
	return VIEWS.find(view => view.id === id);
}

/** The place that is the ground, if it has been drawn. There is at most one. */
export function groundView(): View | undefined {
	return VIEWS.find(view => view.dest === null);
}

/** The places of one destination — one per errand, so a destination may have more than one. */
export function viewsOfDest(dest: DestId): View[] {
	return VIEWS.filter(view => view.dest === dest);
}

/** Every destination in the rail that nobody has drawn a place for yet. */
export function undrawnDests(): DestId[] {
	return DESTINATIONS.filter(dest => !VIEWS.some(view => view.dest === dest.id)).map(
		dest => dest.id
	);
}

export type { Place, View, ViewDraw, PlaceVariant } from './types';
