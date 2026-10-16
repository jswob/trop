import type { ReactNode } from 'react';

/**
 * Which frame an entry is rendered in. There is one since switch-over: `bare`, the entry drawing
 * its own chrome. `app` framed the current app's mirrors in its real `AppLayout`, and went with
 * them (migration D123).
 */
export type ChromeKind = 'bare';

/**
 * How long a subject's two lines may be.
 *
 * A description sits on one line beside the subject's name and a note on one line over its frame.
 * The essays these replaced ran to 1,152 characters over one place and pushed its frames 344px down
 * the window. `registry.test` fails a longer one in the settled spaces.
 */
export const DESCRIPTION_CAP = 100;
export const NOTE_CAP = 60;

export type SubjectVariant = {
	/** URL-safe; it is what `?variants=` carries. */
	id: string;
	label: string;
	/** Why this state is listed, in at most `NOTE_CAP` characters. */
	note?: string;
	render: () => ReactNode;
};

export type Subject = {
	/** URL-safe; it is the `$subject` path param. */
	id: string;
	title: string;
	/** What it is, in at most `DESCRIPTION_CAP` characters. */
	description: string;
	/** The frame used unless `?chrome=` overrides it. */
	chrome: ChromeKind;
	variants: SubjectVariant[];
};
