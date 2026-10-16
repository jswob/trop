import { GROUND, type DestId } from '@app/ui/views/destinations';

import type { RailReading } from '../kit';
import { NOTES } from './notes';

/**
 * **What each rail cell prints at rest on the bench**: the count beside Notes, nothing beside
 * Account.
 *
 * A fact is data, so it lives with the fixtures rather than in `DESTINATIONS`, where it would ship
 * with the app. The app works its facts out from its records; the workbench's frame reads these,
 * and a place still overrides one through `ViewDraw.facts` where its own page changes it.
 */
export const RESTING_FACTS: Record<DestId, string> = {
	notes: String(NOTES.length),
	account: '',
};

/** The ground's reading at rest: the app's name, and nothing about it. */
export const RESTING_READING: RailReading = { place: GROUND, fact: '' };
