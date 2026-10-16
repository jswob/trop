import type { ChromeKind, Subject } from '../lib/subject';
import type { FrameKind } from './frame-url';

/**
 * One frame-able variant, flattened: everything the probe runner needs to build an address.
 */
export type FrameEntry = {
	id: string;
	kind: FrameKind;
	title: string;
	variant: string;
	variantLabel: string;
	chrome: ChromeKind;
};

/**
 * Every variant of every subject, flattened to one address each.
 *
 * The probe runner cannot import `registry.ts`: the chain reaches `import.meta.glob` and a
 * dozen `.tsx` modules, neither of which resolves outside Vite. So the workbench publishes
 * its own census on `window` instead and the runner reads it from the page — the same
 * direction every other measurement goes, and for the same reason (`PROBES.md`: ask the
 * page, never the image, and never a second copy of the truth).
 */
export function frameCensus(entries: Subject[], kindOf: (id: string) => FrameKind): FrameEntry[] {
	return entries.flatMap(entry =>
		entry.variants.map(variant => ({
			id: entry.id,
			kind: kindOf(entry.id),
			title: entry.title,
			variant: variant.id,
			variantLabel: variant.label,
			chrome: entry.chrome,
		}))
	);
}

/** The global the workbench publishes and `probe/run.ts` reads. */
export const CENSUS_KEY = '__DESIGN_FRAMES__';
