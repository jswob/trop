import { CircleUser, NotebookPen } from 'lucide-react';

import type { PageHeading, SplitDest } from '@app/ui/kit';

/**
 * The app's destinations — the rail standing on the split, everywhere.
 *
 * They live here rather than beside any one screen because no destination owns the list of
 * destinations. The ground — the other half of the screen, and below `lg` what every page sits
 * on — is not in the list: it is reached from the logo, which is its name and its switch.
 *
 * Ordered by what they act on: the app's records, then you. `group` is the only thing that draws a
 * hairline between two cells.
 *
 * **A cell's `fact` is empty here.** Facts are data — the app reads them off its records and the
 * workbench off `apps/workbench/fixtures/rail-facts` — so the list carries none that could ship.
 */

export type DestId = 'notes' | 'account';

/** One destination, with its id narrowed so a slug can ask for its fact without a cast. */
export type Destination = SplitDest & { id: DestId };

export const DESTINATIONS: Destination[] = [
	{
		id: 'notes',
		label: 'Notes',
		icon: NotebookPen,
		fact: '',
		blurb: 'Write a note, read them all, delete one.',
		group: 'data',
	},
	{
		id: 'account',
		label: 'Account',
		icon: CircleUser,
		fact: '',
		blurb: 'This device’s sign-in and appearance, and how to end it.',
		group: 'you',
	},
];

/** What the ground is called: the app's name, the logo's word. */
export const GROUND = 'App';

/** Where the frame goes when the ground is collapsed with nothing else open. */
export const RESTING: DestId = 'notes';

export function destinationFor(id: string): Destination | undefined {
	return DESTINATIONS.find(dest => dest.id === id);
}

/**
 * The address of one record's own page — `#notes/n-groceries`.
 *
 * **A link on a page is a way to a destination**, so where it goes is written in the one shape the
 * frame can read back (`hrefDest`). A page that invented its own shape would draw a link the frame
 * cannot see, and a link the frame cannot see is a way out of a hold that nothing guards.
 */
export function recordHref(dest: string, id: string): string {
	return `#${dest}/${id}`;
}

/**
 * The destination a record link names — `notes` in `#notes/n-groceries` — whether or not there
 * is one, or `null` for a link of any other shape. A frame checks it against its own list.
 */
export function hrefId(href: string): string | null {
	return /^#([a-z]+)\//.exec(href)?.[1] ?? null;
}

/** Which of the app's destinations a link on a page goes to, or `null` for a link that goes to none. */
export function hrefDest(href: string): DestId | null {
	const id = hrefId(href);
	return DESTINATIONS.find(dest => dest.id === id)?.id ?? null;
}

/**
 * The address of a record a page opens inside itself — `#notes/note/n-groceries`.
 *
 * It is a record address under the page's own destination, so `hrefDest` reads it back as that
 * destination: a frame that knows nothing of nested pages sees a link to the page you are on and
 * leaves it alone, and one that does reads the rest with `hrefNest`.
 */
export function nestHref(dest: string, kind: string, id: string): string {
	return recordHref(dest, `${kind}/${id}`);
}

/** A destination's address in the app: `/notes`, or `/` for the ground. */
export function destPath(dest: string | null): string {
	return dest === null ? '/' : `/${dest}`;
}

/**
 * The same record's address in the app, where the open destination is the path:
 * `/notes/note/n-groceries`. A reload or a new tab lands on it.
 */
export function nestPath(dest: string, kind: string, id: string): string {
	return `${destPath(dest)}/${kind}/${encodeURIComponent(id)}`;
}

/**
 * A nested page's address, read back into its three parts, or `null` for any other link — in the
 * workbench's form (`#notes/note/n-groceries`) or the app's (`/notes/note/n-groceries`).
 */
export function hrefNest(href: string): { dest: DestId; kind: string; id: string } | null {
	const path = /^\/([a-z]+)\/([a-z-]+)\/([^/]+)$/.exec(href);
	if (path !== null) {
		const dest = destinationFor(path[1] ?? '')?.id ?? null;
		const [, , kind, id] = path;
		if (dest === null || kind === undefined || id === undefined) return null;
		return { dest, kind, id: decodeURIComponent(id) };
	}
	const match = /^#[a-z]+\/([a-z-]+)\/(.+)$/.exec(href);
	const dest = hrefDest(href);
	const kind = match?.[1];
	const id = match?.[2];
	if (dest === null || kind === undefined || id === undefined) return null;
	return { dest, kind, id };
}

/**
 * What a page calls itself, when nothing says otherwise.
 *
 * A page is named by what it holds, so the cell's label and its blurb are the head — one string, in
 * one place, printed wherever the page is. A place overrides it only where one destination is two
 * errands under one word: *Add a note* with nothing open and the note's own title when one is.
 */
export function headOf(dest: SplitDest): PageHeading {
	return { title: dest.label, lede: dest.blurb };
}
