import { pagedListFrames } from '@app/ui/kit/PagedList.specimen';
import { splitShellFrames } from '@app/ui/kit/SplitShell.specimen';
import { applyAppearance } from '@app/ui/views/appearance';
import { DESTINATIONS } from '@app/ui/views/destinations';
import { defineSetup } from '@jswob/design-workbench';
import type { ReactNode } from 'react';

import { KIT_ENTRIES } from './kit-docs';
import { SPECIMENS } from './kit-docs/specimens';
import { SPECIMEN_SUBJECTS } from './kit-docs/specimens/frames';
import { kitStress } from './kit-docs/stress';
import { PROTOS } from './proto';
import { VIEWS } from './views';

/**
 * The app, handed to the workbench: the browser's half of the binding (`workbench.config.ts` is the
 * terminal's).
 *
 * Everything the workbench draws of the app comes through this file, so this is the list of what
 * it draws: the destinations, every place under them, what is unsettled, and the kit.
 */

/** The app's components read no provider: what a frame draws comes in through its props. */
function Wrap({ children }: { children: ReactNode }) {
	return children;
}

/**
 * The app's ground and ink, at the height of the window.
 *
 * It assumes it owns the viewport, which is why the stage loads a frame in an iframe, and it
 * stands at `h-dvh` rather than `h-screen` for the reason `kit/screen`'s `SCREEN_H` gives: opened
 * directly on a phone, `100vh` is the height the glass would have with every browser toolbar
 * retracted, and the frame's foot ends up under the tab bar.
 */
function Ground({ children }: { children: ReactNode }) {
	return (
		<div className="flex h-dvh min-h-0 flex-col bg-ui-bg text-ui-ink [color-scheme:var(--ui-scheme)]">
			{children}
		</div>
	);
}

export default defineSetup({
	Wrap,
	Ground,
	applyAppearance,
	sections: DESTINATIONS.map(dest => ({ id: dest.id, label: dest.label })),
	groundLabel: 'Home',
	places: VIEWS,
	protos: PROTOS,
	kit: {
		entries: KIT_ENTRIES,
		specimens: SPECIMENS,
		stress: kitStress,
		subjects: [splitShellFrames, pagedListFrames, ...SPECIMEN_SUBJECTS],
	},
});
