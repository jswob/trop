import { z } from 'zod';

import { VIEWPORT_NAMES } from './viewports';

/**
 * Everything the stage shows is in the URL, so a comparison is deep-linkable and
 * "screenshot that again" is one navigation.
 *
 * `variants` is a comma-separated list rather than the router's default JSON
 * array encoding, so the URL stays typeable by hand.
 */
export const stageSearchSchema = z.object({
	variants: z.string().optional(),
	/**
	 * A cross-subject frame list, `subject:variant` comma-separated. It is what puts
	 * a redrawn view beside the mirror it replaces: `variants` can only ever name
	 * states of one subject, and "better" means nothing without the before-picture.
	 */
	frames: z.string().optional(),
	width: z.enum(VIEWPORT_NAMES).optional(),
	chrome: z.enum(['bare']).optional(),
	/** Absent means fit the frames to the space available. */
	scale: z.coerce.number().min(0.1).max(2).optional(),
	/**
	 * Milliseconds to pin every animation at, which is how a transition is
	 * captured: the same variant at several `t` values is a filmstrip, and each
	 * frame of it is an exact, repeatable screenshot. Wins over `rate`.
	 *
	 * Bounded because an unparseable param does not fail alone where the whole object is
	 * parsed at once, as the frame route parses it, so a fat-fingered `t` would silently
	 * reset the chrome too.
	 */
	t: z.coerce.number().min(0).max(10_000).optional(),
	/**
	 * How fast the frame's clock runs, as a multiplier on real time. Absent means
	 * `1`, and `1` is the default a frame is opened at — animation is how the thing
	 * behaves, and a stage that never shows it is reviewing half the design.
	 *
	 * The fractions are what make a transition inspectable: 150ms is over before the
	 * eye can say what moved, and a fifth of that is the same design with the answer
	 * visible in it.
	 *
	 * **`0` is off, not a stopped clock.** It settles every animation at its end
	 * state — the frame a capture should take, and the only one that repeats exactly.
	 * A clock held at zero and no animation at all are indistinguishable in a still
	 * and are different things to reason about, so only one of them is spelled here.
	 *
	 * Capped at `1`: speeding motion up tells you nothing you cannot get by
	 * watching it once, and an animation running faster than it was designed to is
	 * a different design.
	 */
	rate: z.coerce.number().min(0).max(1).optional(),
	/**
	 * The appearance every frame on the page is drawn in. Absent is dark, as it is in the
	 * app for a device that has not chosen. On the page's own URL, so each page keeps its own.
	 */
	theme: z.enum(['dark', 'light']).optional(),
});

/**
 * Splits the `variants` search value, dropping the empties a trailing comma leaves.
 *
 * The value is typed `unknown` because nothing guarantees the declared shape at
 * runtime: the route's validator rejects a `variants` it cannot parse, but the
 * raw value still reaches the stage through the match's merged search, and
 * `useTypedRoute` hands it over as `any`. A hand-typed URL that repeats the
 * param arrives as an array, so that shape is read rather than thrown on.
 */
export function splitVariants(value: unknown): string[] | undefined {
	const parts = Array.isArray(value) ? value : [value];
	const ids = parts
		.flatMap(part => (typeof part === 'string' ? part.split(',') : []))
		.filter(Boolean);

	return ids.length > 0 ? ids : undefined;
}

/** One entry of a `frames` list: a subject id, and optionally the variant to show. */
export type FrameRef = { subject: string; variant?: string };

/**
 * Splits `frames` into subject/variant pairs, dropping empties and anything with
 * no subject id. Read the same defensive way as `splitVariants`: a hand-typed URL
 * can repeat the param, which arrives as an array.
 */
export function splitFrames(value: unknown): FrameRef[] | undefined {
	const parts = Array.isArray(value) ? value : [value];
	const refs = parts
		.flatMap(part => (typeof part === 'string' ? part.split(',') : []))
		.map(entry => entry.trim())
		.filter(Boolean)
		.map(entry => {
			const [subject = '', variant] = entry.split(':');
			return variant ? { subject, variant } : { subject };
		})
		.filter(ref => ref.subject.length > 0);

	return refs.length > 0 ? refs : undefined;
}
