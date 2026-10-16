import type { Specimen } from '@jswob/design-workbench';

import { KIT_ENTRIES } from '../index';

/**
 * Every specimen beside its component: `Button.specimen.tsx` next to `Button.tsx`, in the UI
 * package's kit and in the design root's own. A component's page is its file's default export.
 */
const FOUND = import.meta.glob<Specimen>(
	['../../../../packages/ui/src/kit/*.specimen.tsx', '../../kit/*.specimen.tsx'],
	{ eager: true, import: 'default' }
);

const ORDER = new Map(KIT_ENTRIES.map((entry, at) => [entry.id, at]));

/** Where a specimen stands in the catalogue: one the catalogue does not list goes last. */
const placeOf = (spec: Specimen): number => ORDER.get(spec.id) ?? ORDER.size;

/**
 * Every component of the kit, as its declared page, in the catalogue's own order.
 *
 * A component drawn at its own size is cut to it. One that needs a whole screen, a shell or a
 * list twenty rows tall, is fitted to the stage as a place is.
 */
export const SPECIMENS: Specimen[] = Object.values(FOUND).sort((a, b) => placeOf(a) - placeOf(b));
