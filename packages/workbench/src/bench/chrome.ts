import { createContext, use } from 'react';

import type { WorkbenchAppearance } from './appearance';
import type { BenchSearch } from './search';
import type { Tried } from './tokens';

/** What the shell holds for the pages inside it: the sidebar, the jump list and the appearance. */
export type Chrome = {
	folded: boolean;
	toggleFolded: () => void;
	/** The sidebar as a drawer, below the width it stands open at. */
	drawer: boolean;
	setDrawer: (open: boolean) => void;
	palette: boolean;
	setPalette: (open: boolean) => void;
	appearance: WorkbenchAppearance;
	setAppearance: (next: WorkbenchAppearance) => void;
	/** The options a link to another place takes with it. */
	carried: BenchSearch;
	/** Token values being tried on every frame of the bench, from the address. */
	tried: Tried;
	setTried: (next: Tried) => void;
};

export const ChromeContext = createContext<Chrome | null>(null);

export function useChrome(): Chrome {
	const chrome = use(ChromeContext);
	if (chrome === null) throw new Error('useChrome must be used inside the bench shell');
	return chrome;
}
