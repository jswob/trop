import { useLayoutEffect, useRef } from 'react';
import type { RefObject } from 'react';

/**
 * **Keeps every copy of one animation under `root` on the one start time**, while `active` — for a
 * field drawn as many compositor animations (the glaze's layers, the breath's washes) that must
 * read as one. The start time is the moment the first copy began, so an element that mounts later
 * joins the field where it is rather than starting its own. Runs after every render of the caller,
 * which is when its elements mount.
 *
 * Only a running animation is moved: the workbench pins motion by pausing every animation at one
 * moment, and a paused one given a start time would play again.
 * @param root - The element holding every copy
 * @param name - The keyframes' name
 * @param active - Whether the field runs now
 */
export function useInStep(
	root: RefObject<HTMLElement | null>,
	name: string,
	active: boolean
): void {
	const origin = useRef<CSSNumberish | null>(null);

	useLayoutEffect(() => {
		if (!active) {
			origin.current = null;
			return;
		}
		for (const animation of root.current?.getAnimations({ subtree: true }) ?? []) {
			if ((animation as Partial<CSSAnimation>).animationName !== name) continue;
			if (animation.playState !== 'running') continue;
			origin.current ??= animation.startTime ?? document.timeline.currentTime;
			if (origin.current !== null && animation.startTime !== origin.current) {
				animation.startTime = origin.current;
			}
		}
	});
}
