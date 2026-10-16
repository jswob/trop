import { RouterProvider } from '@tanstack/react-router';
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import 'virtual:workbench/styles.css';
import { CENSUS_KEY, frameCensus, type FrameEntry } from './lib/frame-census';
import { installFrameClock } from './lib/motion';
import { stageEntries, stageKindOf } from './lib/registry';
import { createDesignRouter } from './router';

declare global {
	interface Window {
		[CENSUS_KEY]?: FrameEntry[];
	}
}

/**
 * Before anything mounts, and deliberately not in an effect.
 *
 * Headless UI schedules its transition work behind a double `requestAnimationFrame`,
 * which never fires in a tab the browser is not painting — every tab a screenshot is
 * taken of. An effect would leave one commit in which a frame could be scheduled
 * against the clock that is about to be replaced.
 */
installFrameClock();

/**
 * The frame census, published for `probe/run.ts`.
 *
 * The runner cannot import the registry — the chain reaches `import.meta.glob` and a dozen
 * `.tsx` modules, and neither resolves outside Vite. It asks the page instead, which is the
 * same direction `PROBES.md` sends every other measurement and keeps one list of frames
 * rather than two that agree today.
 */
window[CENSUS_KEY] = frameCensus(stageEntries(), stageKindOf);

const router = createDesignRouter();

const rootElement = document.getElementById('root');
if (!rootElement) {
	throw new Error('Root element not found');
}

createRoot(rootElement).render(
	<StrictMode>
		<RouterProvider router={router} />
	</StrictMode>
);
