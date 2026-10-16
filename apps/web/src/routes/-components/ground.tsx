import type { RailReading } from '@app/ui/kit';
import { GROUND, type DestId } from '@app/ui/views/destinations';
import { HomePage } from '@app/ui/views/home';
import type { QueryKey } from '@tanstack/react-query';
import type { ReactNode } from 'react';

import { APP_DESTINATIONS } from '@/frame/pages';

/**
 * The ground's own reads — what its page is drawn from. One of them failing puts the centred screen
 * up in place of the frame; any other failed read is said in the waits on screen. The home page
 * reads nothing, so there are none.
 */
export const GROUND_READS: readonly QueryKey[] = [];

/** What the frame draws of the ground: its page, each cell's fact and the slug's reading. */
export type Ground = {
	ledger: ReactNode;
	facts: Partial<Record<DestId, string>>;
	reading: RailReading;
};

const GROUND_VIEW: Ground = {
	ledger: <HomePage name={GROUND} destinations={APP_DESTINATIONS} />,
	facts: {},
	reading: { place: GROUND, fact: '' },
};

/**
 * **The ground, for `AppFrame` to hand the frame**: the home page, and what the rail and the slug
 * say of it. A ground drawn from records reads them here, and lists their keys in `GROUND_READS`.
 */
export function useGround(): Ground {
	return GROUND_VIEW;
}
