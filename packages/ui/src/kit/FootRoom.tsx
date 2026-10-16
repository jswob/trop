import { createContext, useCallback, useContext, useEffect, useState } from 'react';

/**
 * **Room at the foot of a column, asked for by what is standing there.**
 *
 * Below `lg` the chrome's own slug floats over the foot of the column — `kit/RailShell` — and it is
 * the last object on the screen that may be covered: it is how you get back. Anything else that
 * pins itself to that edge lands on top of it, because a floating object inside the scroller is
 * positioned against the scrollport and knows nothing about the chrome outside it.
 *
 * So the claim runs the other way. A thing pinned to the foot says how tall it is; the chrome lifts
 * its slug by that much and puts it back when the claim goes to zero. The slug moves rather than the
 * column, because the column's own floor (`SLUG_FLOOR`) is what keeps its **last row** reachable, and
 * a floor that grew and shrank with a floating object would change the document's height every time
 * that object appeared — which is the one thing a sticky overlay is not allowed to do.
 *
 * **It is opt-in and costs nothing unclaimed.** A shell wraps its columns in the provider; a column
 * with nothing at its foot never calls the hook, the room stays 0, and the chrome draws exactly what
 * it drew before.
 */
type Claim = (room: number) => void;

const ClaimContext = createContext<Claim | null>(null);

/** The chrome's side: how much room is claimed right now, and the claim to hand to the columns. */
export function useFootRoom(): [number, Claim] {
	const [room, setRoom] = useState(0);
	const claim = useCallback((next: number) => {
		setRoom(was => (was === next ? was : next));
	}, []);
	return [room, claim];
}

export const FootRoomProvider = ClaimContext.Provider;

/**
 * The column's side: ask the chrome for `room` px of clearance at the foot, and give it back on the
 * way out. Outside a shell that provides one, it does nothing.
 */
export function useClaimFootRoom(room: number) {
	const claim = useContext(ClaimContext);

	useEffect(() => {
		if (claim === null) return;
		claim(room);
		return () => {
			claim(0);
		};
	}, [claim, room]);
}
