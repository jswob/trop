import type { RailReading } from '@app/ui/kit';
import { SIGN_OUT } from '@app/ui/views/account';
import {
	DESTINATIONS,
	destinationFor,
	type DestId,
	type Destination,
} from '@app/ui/views/destinations';
import type { PageErrand } from '@app/ui/views/frame-types';

/** The destinations that have a route in `src/routes/` (`pages.test.ts` holds the two together). */
const ROUTED: ReadonlySet<DestId> = new Set<DestId>(['notes', 'account']);

/**
 * **The rail's cells in the app**: the destinations it has a page for, in the rail's order. One the
 * workbench stages before the app routes it stays off until its route lands, so the app never draws
 * a cell that leads nowhere.
 */
export const APP_DESTINATIONS: readonly Destination[] = DESTINATIONS.filter(dest =>
	ROUTED.has(dest.id)
);

/** The destination an address is on — `/notes/…` is Notes — or `null` for the ground. */
export function destOfPath(pathname: string): DestId | null {
	const segment = /^\/([^/]+)/.exec(pathname)?.[1];
	return segment === undefined ? null : (destinationFor(segment)?.id ?? null);
}

/** What a page declares to the frame beside its body: what the slug prints, and its errand. */
export type PageDeclaration = { reading: RailReading; errand?: PageErrand };

/**
 * **What each page tells the frame**: the ground prints its own reading; Account its name and
 * `Sign out`. A page with nothing of its own to say prints its cell's name and fact, as the
 * workbench's frame does for a page it did not draw.
 */
export function pageDeclaration(
	dest: DestId | null,
	facts: Partial<Record<DestId, string>>,
	ground: RailReading
): PageDeclaration {
	if (dest === null) return { reading: ground };
	const label = destinationFor(dest)?.label ?? dest;
	const reading = { place: label, fact: facts[dest] ?? '' };
	return dest === 'account' ? { reading, errand: SIGN_OUT } : { reading };
}
