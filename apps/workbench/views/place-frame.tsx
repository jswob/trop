import { DESTINATIONS, GROUND } from '@app/ui/views/destinations';
import { HomePage } from '@app/ui/views/home';

import { RESTING_FACTS, RESTING_READING } from '../fixtures/rail-facts';
import { standInServer } from '../fixtures/sign-in-server';
import { FlowFrame } from './flow-frame';
import type { Place, View } from './types';

/**
 * **A place, framed on the fixtures** — what the workbench stages.
 *
 * `@app/ui/views/frame`'s `Frame` is the frame itself and carries no data; this hands it what the
 * app hands it from its records — the ground's page, the cells' resting facts, the ground's reading
 * — taken from the fixtures instead, and the stand-in sign-in server. It is split from the frame
 * because the app frames its routes with the same `Frame` and none of this ever ships.
 *
 * The frame walks (`views/flow-frame`): a rail cell lands on the place drawn in that destination,
 * so a place is staged with every other place there is to go to.
 */

/**
 * The ground as it usually is: the home page, as the app's own ground draws it. A place overrides
 * it only to show what its own page did to it.
 */
function restingLedger() {
	return <HomePage name={GROUND} destinations={DESTINATIONS} />;
}

/**
 * A place, framed — the `Subject` the stage, the frame route and the frame list all take.
 *
 * The `Subject` shape is kept on purpose: a view is compared, screenshotted and put beside another
 * frame by exactly the same machinery as any other, and a second shape for places would mean a
 * second copy of all of it.
 *
 * `places` is every place its frame can walk to, the place itself among them.
 */
export function framePlace(place: Place, places: readonly Place[]): View {
	return {
		id: place.id,
		title: place.title,
		description: place.description,
		chrome: 'bare',
		dest: place.dest,
		variants: place.variants.map(variant => ({
			id: variant.id,
			label: variant.label,
			note: variant.note,
			render: () => (
				<FlowFrame
					home={place}
					state={variant}
					places={places}
					ledger={restingLedger()}
					facts={RESTING_FACTS}
					groundReading={RESTING_READING}
					server={standInServer}
				/>
			),
		})),
	};
}
