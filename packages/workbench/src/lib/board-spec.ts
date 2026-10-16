import type { Appearance } from './appearance';

import { POSITION_PARAM } from './frame-position';

/**
 * What one frame on a board draws.
 *
 * It is written `people:person?theme=light&rate=0` — the id and state as `drive` and `--frames`
 * already write them, then the params the workbench's own frame address takes — so a frame on a
 * board is named the way the same frame is named everywhere else.
 *
 * **It travels in the frame's `name`, not in its address.** A board is a published page whose
 * frames open one file beside it, and a query string on that file is not something the host
 * promises to pass through. The `name` an `<iframe>` is given always reaches the document inside.
 */
export type FrameSpec = {
	id: string;
	/** Which of the entry's states. Absent means its first. */
	variant?: string;
	theme: Appearance;
	/** Milliseconds to pin every animation at, as on the frame address. */
	t?: number;
	/** Clock speed, as on the frame address. `0` is off. */
	rate?: number;
	/** Where the frame has walked to, as on the frame address (`lib/frame-position`). */
	where?: string;
};

const NAME = /^[a-z0-9][a-z0-9-]*$/;

function bounded(value: string | null, max: number): number | undefined {
	if (value === null || value.trim() === '') return undefined;

	const parsed = Number(value);
	return Number.isFinite(parsed) && parsed >= 0 && parsed <= max ? parsed : undefined;
}

/** The spec a frame's `name` carries, or `null` when it names nothing that could be a frame. */
export function parseFrameSpec(name: string): FrameSpec | null {
	const [address = '', query = ''] = name.trim().split('?', 2);
	const [id = '', variant] = address.split(':', 2);
	if (!NAME.test(id) || (variant !== undefined && !NAME.test(variant))) return null;

	const params = new URLSearchParams(query);
	const spec: FrameSpec = { id, theme: params.get('theme') === 'light' ? 'light' : 'dark' };
	if (variant !== undefined) spec.variant = variant;

	const rate = bounded(params.get('rate'), 1);
	if (rate !== undefined) spec.rate = rate;

	const t = bounded(params.get('t'), 10_000);
	if (t !== undefined) spec.t = t;

	const where = params.get(POSITION_PARAM);
	if (where !== null && NAME.test(where)) spec.where = where;

	return spec;
}
