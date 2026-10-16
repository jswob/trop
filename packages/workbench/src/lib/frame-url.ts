import type { ChromeKind } from '../lib/subject';
import type { Appearance } from './appearance';

import { POSITION_PARAM } from './frame-position';

/**
 * Which space a frame's entry came from. It is a label in the URL, not a lookup key:
 * `stageEntryById` crosses every space, so a frame list can put entries from two side by side.
 */
export type FrameKind = 'proto' | 'view' | 'kit';

/**
 * What a frame does with its animations, as the URL carries it.
 *
 * **Real time is the default, and it is left off the address.** An unadorned URL is the
 * component moving the way it was designed to move, which is what every other control on
 * the bar does too: the stage shows the thing as it is, not as a capture would prefer it.
 */
export type FrameMotion = {
	/** Milliseconds to pin every animation at. Wins over `rate`. */
	t?: number;
	/**
	 * Clock speed as a multiplier on real time. Absent means `1`. `0` is not a slow clock
	 * but **off**: every animation at its end state, which is the frame a capture wants.
	 */
	rate?: number;
};

/**
 * The URL of a single variant rendered on its own. Both the compare grid (which
 * loads these into iframes) and a direct screenshot use the same address, so a
 * frame can always be opened full-size in a browser tab.
 *
 * `theme` is the appearance the frame is drawn in. Dark is the tokens with nothing set, so it
 * is left off the address and every existing URL is unchanged.
 *
 * `where` is the position a frame has walked to (`lib/frame-position`): the address then opens the
 * frame where it was left. A frame that has not moved names none.
 */
export function frameUrl(
	kind: FrameKind,
	id: string,
	variant: string,
	chrome: ChromeKind,
	motion: FrameMotion = {},
	theme: Appearance = 'dark',
	where: string | null = null
): string {
	const search = new URLSearchParams({ variant, chrome });

	if (motion.t !== undefined) search.set('t', String(motion.t));
	if (motion.rate !== undefined) search.set('rate', String(motion.rate));
	if (theme === 'light') search.set('theme', 'light');
	if (where !== null) search.set(POSITION_PARAM, where);

	return `/frame/${kind}/${id}?${search.toString()}`;
}
