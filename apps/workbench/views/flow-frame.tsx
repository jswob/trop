import { DESTINATIONS, GROUND, RESTING } from '@app/ui/views/destinations';
import { Frame, type FrameProps } from '@app/ui/views/frame';
import { useFramePosition } from '@jswob/design-workbench';
import { Fragment, useMemo, useState } from 'react';

import type { SplitControl } from '../kit';

import { positionOf, standingAt } from './flow';
import type { Place, PlaceVariant } from './types';

export type FlowFrameProps = Pick<FrameProps, 'ledger' | 'facts' | 'groundReading' | 'server'> & {
	/** The place the frame was opened on, and the state of it. */
	home: Place;
	state: PlaceVariant;
	/** Every place there is to walk to. */
	places: readonly Place[];
};

/**
 * **A place's frame, which walks** — one `Frame`, mounted once, around whichever place it stands on.
 *
 * A rail cell and a register row land on the place drawn there, in that place's first state, at
 * every width. The frame's own destination is its own place again, in the state it was opened on.
 *
 * It is the app's arrangement with the address swapped for one word. The app keeps `where` in its
 * route and hands it to the frame as `control`; this keeps it in the frame's own address
 * (`lib/frame-position`), so a reload and the bench's `Open` find the frame where it was left, and
 * hands it over the same way. The moves themselves are still `kit/SplitShell`'s one copy.
 *
 * **What a walk does not carry.** A place walked to is drawn fresh, in its first state, and holds
 * nothing of the page that was left. A hold declared by the home state is read once by the frame,
 * so coming home after discarding it finds the page without its hold.
 */
export function FlowFrame({
	home,
	state,
	places,
	ledger,
	facts,
	groundReading,
	server,
}: FlowFrameProps) {
	const [position, setPosition] = useFramePosition();
	const standing = standingAt(position, home, places);
	const away = standing.place === home ? null : standing.place;

	const homeDraw = useMemo(() => state.draw(), [state]);
	const awayDraw = useMemo(() => away?.variants[0]?.draw() ?? null, [away]);
	const draw = awayDraw ?? homeDraw;
	const page =
		draw.page === undefined ? undefined : <Fragment key={standing.place.id}>{draw.page}</Fragment>;

	const [glass, setGlass] = useState({ ledger: true, menu: homeDraw.menu ?? false });
	const control: SplitControl = {
		where: { ...glass, dest: standing.dest },
		onWhere: next => {
			setGlass({ ledger: next.ledger, menu: next.menu });
			if (next.dest !== standing.dest) setPosition(positionOf(next.dest, home, places));
		},
	};

	return (
		<Frame
			place={standing.place}
			draw={{ ...draw, page }}
			destinations={DESTINATIONS}
			home={GROUND}
			resting={RESTING}
			ledger={ledger}
			facts={facts}
			groundReading={groundReading}
			server={server}
			control={control}
		/>
	);
}
