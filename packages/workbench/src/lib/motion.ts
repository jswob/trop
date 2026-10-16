/**
 * Motion in a tab that is not being painted.
 *
 * The workbench used to zero every `transition-duration` and `animation-duration`
 * globally, on the stated grounds that Headless UI clears an overlay's
 * `data-closed` styles on `transitionend`. That reading was wrong.
 * `@headlessui/react@2.2.9` waits on the Web Animations API instead —
 * `Promise.allSettled(el.getAnimations().map(a => a.finished))` — and a
 * `finished` promise does resolve in a hidden tab. What does not happen there is
 * the **double `requestAnimationFrame`** it schedules that wait behind, so the
 * overlay never reaches the commit that would settle it.
 *
 * Two facts, both measured against a Chrome MCP capture tab, are what this file
 * is built on:
 *
 * 1. `requestAnimationFrame` never fires while `visibilityState` is `hidden`,
 *    but `setTimeout(fn, 0)` is **not** throttled there — 2ms, against the ~1s
 *    clamp every non-zero delay gets. So a frame can be faked for the cost of a
 *    task.
 * 2. An animation's clock is frozen in a hidden tab, but setting `currentTime`
 *    still interpolates computed style exactly, and that scrubbed frame is what
 *    a screenshot captures. So a motion axis can be reviewed as a filmstrip of
 *    pinned frames — deterministically, which watching it never was.
 */

/**
 * An animation with only the surface these passes touch.
 *
 * A real `Animation` satisfies it; so does a stub in a test, which is the point
 * — nothing here needs a browser to be checked.
 */
export type PinnableAnimation = {
	effect: { getComputedTiming: () => { endTime?: CSSNumberish } } | null;
	currentTime: CSSNumberish | null;
	pause: () => void;
	finish: () => void;
};

/**
 * What a frame does with its animations: settle them at their end state, let the
 * clock run, or pin every one at the same millisecond.
 */
export type MotionPin = 'settled' | 'live' | number;

/**
 * Where an animation ends, in milliseconds, or `Infinity` if it never does.
 *
 * `endTime` is typed `CSSNumberish`, which is a `CSSNumericValue` for animations
 * driven by something other than a document timeline. None of those exist here,
 * and one that did could not be compared against a millisecond anyway, so it is
 * treated as endless rather than guessed at.
 */
export function endTimeOf(animation: PinnableAnimation): number {
	const endTime = animation.effect?.getComputedTiming().endTime;
	return typeof endTime === 'number' ? endTime : Infinity;
}

/**
 * Puts one animation into the state it would have reached if the tab had been
 * painting — which for a spinner is not a state at all.
 *
 * `finish()` throws `InvalidStateError` on an animation whose end time is
 * infinite, and `animate-spin` is exactly that and is already on screen in two
 * mirrors. An endless animation is pinned at its first frame instead: the same
 * thing the old zeroed `animation-duration` produced, reached honestly.
 */
export function settleAnimation(animation: PinnableAnimation): void {
	if (endTimeOf(animation) === Infinity) {
		animation.pause();
		animation.currentTime = 0;
		return;
	}

	animation.finish();
}

/**
 * Holds one animation at `at` milliseconds, clamped to its own end.
 *
 * Clamping is per-animation rather than per-frame because a filmstrip asks every
 * animation on the page for the same moment, and they do not all last as long. A
 * 120ms pin on an 80ms transition is that transition finished, not an error.
 */
export function pinAnimation(animation: PinnableAnimation, at: number): void {
	animation.pause();
	animation.currentTime = Math.min(at, endTimeOf(animation));
}

/**
 * What a frame does with its animations, decided from the two search params that name it.
 *
 * `t` wins, because naming a moment is more specific than naming a speed. Otherwise the
 * clock speed decides, and every speed but one means the animation runs: `0` is off, and
 * off is the end state rather than a clock standing still.
 */
export function pinFor({ t, rate }: { t?: number; rate?: number }): MotionPin {
	if (t !== undefined) return t;
	return rate === 0 ? 'settled' : 'live';
}

/** Applies one pin to a list of animations. `live` is the absence of a pass. */
export function applyMotion(pin: MotionPin, animations: PinnableAnimation[]): void {
	if (pin === 'live') return;

	for (const animation of animations) {
		if (pin === 'settled') {
			settleAnimation(animation);
		} else {
			pinAnimation(animation, pin);
		}
	}
}

/**
 * Runs `apply` now, once more after the style recalculation that follows, and
 * again after every mutation for as long as the watch is held.
 *
 * One pass at mount is not enough: an overlay's transition is created by the
 * commit that opens it, which is routinely later than the commit that runs this.
 * A `MutationObserver` catches those commits, for the same reason and with the
 * same caveat as `staged-interaction.ts` — `requestAnimationFrame` cannot be the
 * wait here, and mutation callbacks are not throttled the way timers are.
 *
 * **A mutation callback alone is still too early**, which cost a round to find.
 * An animation is created by the style recalculation that *follows* a mutation,
 * not by the mutation, so `getAnimations()` inside the callback can return
 * nothing and the transition that appears a moment later is never touched. That
 * showed up as a modal pinned at `t=0` rendering fully open. Each mutation
 * therefore also schedules one deferred pass — `setTimeout(…, 0)`, which is the
 * only wait available here that a hidden tab does not throttle to a second.
 *
 * `apply` must be idempotent, and both of this file's are: pinning to a time
 * already reached writes the same value, `finish()` on a finished animation is a
 * no-op, and a playback rate is a plain assignment. That is also what makes
 * StrictMode's second effect run harmless.
 *
 * @returns A teardown that stops watching.
 */
function watchDocument(apply: () => void): () => void {
	let deferred: ReturnType<typeof setTimeout> | undefined;

	/** Coalesced: a burst of mutations is one recalculation and wants one pass. */
	const applyAfterRecalc = (): void => {
		if (deferred !== undefined) return;

		deferred = setTimeout(() => {
			deferred = undefined;
			apply();
		}, 0);
	};

	apply();
	applyAfterRecalc();

	const observer = new MutationObserver(() => {
		apply();
		applyAfterRecalc();
	});
	observer.observe(document.body, { childList: true, subtree: true, attributes: true });

	return () => {
		observer.disconnect();
		if (deferred !== undefined) clearTimeout(deferred);
	};
}

/**
 * Keeps a frame's animations pinned, including the ones that do not exist yet.
 *
 * @returns A teardown that stops watching.
 */
export function holdMotion(pin: MotionPin): () => void {
	if (pin === 'live') return () => undefined;

	return watchDocument(() => {
		applyMotion(pin, document.getAnimations());
	});
}

/** An animation with only the surface the slow-motion pass touches. */
export type RateableAnimation = {
	playbackRate: number;
};

/**
 * How fast a frame's clock runs, as a multiplier on real time.
 *
 * `1` is real time, `0.2` is five times slower, and `0` holds every animation
 * wherever it has got to. Slowing down is the point: a 150ms transition is over
 * before the eye can say what moved, and the same transition at a fifth of speed
 * is the same design with the answer visible in it.
 */
export function applyRate(rate: number, animations: RateableAnimation[]): void {
	for (const animation of animations) {
		animation.playbackRate = rate;
	}
}

/**
 * Keeps a frame running at `rate`, including animations that do not exist yet.
 *
 * An animation created after the last pass runs at real time until the deferred
 * pass reaches it — about 2ms, per the timing this file is built on. That is
 * under 2% of the shortest thing here and is not corrected for, because seeking
 * it back would be a second, visible jump.
 *
 * @returns A teardown that stops watching.
 */
export function holdRate(rate: number): () => void {
	if (rate === 1) return () => undefined;

	return watchDocument(() => {
		applyRate(rate, document.getAnimations());
	});
}

/**
 * Gives a hidden tab a frame clock, so libraries that schedule work behind
 * `requestAnimationFrame` still reach the commit they are waiting for.
 *
 * This is the one place the workbench lies about its environment, and it is
 * load-bearing for every screenshot taken after it: a capture tab now runs
 * frames a real background tab would not. The lie is narrow and deliberate —
 * without it, Headless UI's overlays never settle and no mirror can be
 * screenshotted at all, which is the trap that froze motion here in the first
 * place.
 *
 * `setTimeout(fn, 0)` is the queue rather than `16`, because every non-zero
 * delay is clamped to about a second in a hidden tab while zero is not. A
 * double-rAF therefore costs about 4ms instead of two seconds.
 *
 * Both halves of the pair are replaced: the shim hands out its own handles, so a
 * `cancelAnimationFrame` left pointing at the native implementation would cancel
 * an unrelated frame or nothing at all.
 *
 * @returns A teardown that restores the natives and drops anything queued.
 */
export function installFrameClock(): () => void {
	// Kept unbound: these are only ever assigned back, never called through, and a
	// bound copy would not restore the identity anything else compares against.
	// eslint-disable-next-line @typescript-eslint/unbound-method -- Reason: assigned back, never called
	const nativeRequest = window.requestAnimationFrame;
	// eslint-disable-next-line @typescript-eslint/unbound-method -- Reason: assigned back, never called
	const nativeCancel = window.cancelAnimationFrame;

	const queued = new Map<number, ReturnType<typeof setTimeout>>();
	let nextHandle = 1;
	let installed = false;

	const install = (): void => {
		if (installed) return;
		installed = true;

		window.requestAnimationFrame = (callback: FrameRequestCallback): number => {
			const handle = nextHandle;
			nextHandle += 1;

			queued.set(
				handle,
				setTimeout(() => {
					queued.delete(handle);
					callback(performance.now());
				}, 0)
			);

			return handle;
		};

		window.cancelAnimationFrame = (handle: number): void => {
			const timer = queued.get(handle);
			if (timer === undefined) return;

			clearTimeout(timer);
			queued.delete(handle);
		};
	};

	const restore = (): void => {
		if (!installed) return;
		installed = false;

		for (const timer of queued.values()) {
			clearTimeout(timer);
		}
		queued.clear();

		window.requestAnimationFrame = nativeRequest;
		window.cancelAnimationFrame = nativeCancel;
	};

	const sync = (): void => {
		if (document.visibilityState === 'hidden') {
			install();
		} else {
			restore();
		}
	};

	sync();
	document.addEventListener('visibilitychange', sync);

	return () => {
		document.removeEventListener('visibilitychange', sync);
		restore();
	};
}
