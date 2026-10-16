import { afterEach, describe, expect, it, vi, type Mock } from 'vitest';

import {
	applyMotion,
	applyRate,
	endTimeOf,
	holdMotion,
	holdRate,
	installFrameClock,
	pinAnimation,
	pinFor,
	settleAnimation,
	type PinnableAnimation,
	type RateableAnimation,
} from './motion';

/**
 * An animation with only the surface the passes touch.
 *
 * `endTime` is the one thing that decides every branch here — it is what makes
 * `finish()` throw and what a pin is clamped against — so a stub states it and
 * nothing else about its timing.
 */
function stubAnimation(endTime: number): PinnableAnimation & {
	pause: Mock<() => void>;
	finish: Mock<() => void>;
} {
	return {
		effect: { getComputedTiming: () => ({ endTime }) },
		currentTime: 0,
		pause: vi.fn(),
		finish: vi.fn(),
	};
}

describe('endTimeOf', () => {
	it('reads the computed end time', () => {
		expect(endTimeOf(stubAnimation(400))).toBe(400);
	});

	it('calls an animation with no effect endless', () => {
		expect(endTimeOf({ effect: null, currentTime: 0, pause: vi.fn(), finish: vi.fn() })).toBe(
			Infinity
		);
	});
});

describe('settleAnimation', () => {
	it('finishes an animation that has an end', () => {
		const animation = stubAnimation(400);

		settleAnimation(animation);

		expect(animation.finish).toHaveBeenCalledTimes(1);
		expect(animation.pause).not.toHaveBeenCalled();
	});

	it('pins an endless animation at its first frame instead of finishing it', () => {
		const animation = stubAnimation(Infinity);

		settleAnimation(animation);

		expect(animation.finish).not.toHaveBeenCalled();
		expect(animation.pause).toHaveBeenCalledTimes(1);
		expect(animation.currentTime).toBe(0);
	});
});

describe('pinAnimation', () => {
	it('pauses and seeks', () => {
		const animation = stubAnimation(400);

		pinAnimation(animation, 120);

		expect(animation.pause).toHaveBeenCalledTimes(1);
		expect(animation.currentTime).toBe(120);
	});

	it('clamps a pin past the end to the end', () => {
		const animation = stubAnimation(400);

		pinAnimation(animation, 9000);

		expect(animation.currentTime).toBe(400);
	});

	it('does not clamp an endless animation', () => {
		const animation = stubAnimation(Infinity);

		pinAnimation(animation, 9000);

		expect(animation.currentTime).toBe(9000);
	});

	it('is idempotent, so StrictMode running it twice changes nothing', () => {
		const animation = stubAnimation(400);

		pinAnimation(animation, 120);
		pinAnimation(animation, 120);

		expect(animation.currentTime).toBe(120);
	});
});

describe('pinFor', () => {
	it('runs the animation when the URL names nothing', () => {
		expect(pinFor({})).toBe('live');
	});

	it('runs it at every speed but zero', () => {
		expect(pinFor({ rate: 0.2 })).toBe('live');
		expect(pinFor({ rate: 1 })).toBe('live');
	});

	it('reads a rate of zero as off rather than as a stopped clock', () => {
		expect(pinFor({ rate: 0 })).toBe('settled');
	});

	it('lets a named moment win over a speed', () => {
		expect(pinFor({ t: 120, rate: 0 })).toBe(120);
		expect(pinFor({ t: 0 })).toBe(0);
	});
});

describe('applyMotion', () => {
	it('settles every animation by default', () => {
		const animations = [stubAnimation(200), stubAnimation(400)];

		applyMotion('settled', animations);

		expect(animations.map(animation => animation.finish.mock.calls.length)).toEqual([1, 1]);
	});

	it('leaves every animation alone when live', () => {
		const animations = [stubAnimation(200), stubAnimation(Infinity)];

		applyMotion('live', animations);

		expect(animations.map(animation => animation.finish.mock.calls.length)).toEqual([0, 0]);
		expect(animations.map(animation => animation.pause.mock.calls.length)).toEqual([0, 0]);
	});

	it('pins every animation at a time', () => {
		const animations = [stubAnimation(200), stubAnimation(400)];

		applyMotion(120, animations);

		expect(animations.map(animation => animation.currentTime)).toEqual([120, 120]);
	});

	it('pins each animation against its own end', () => {
		const animations = [stubAnimation(80), stubAnimation(400)];

		applyMotion(120, animations);

		expect(animations.map(animation => animation.currentTime)).toEqual([80, 120]);
	});
});

describe('holdMotion', () => {
	/** jsdom has no `document.getAnimations`, so the page's animations are stated here. */
	function withAnimations(animations: PinnableAnimation[]): () => void {
		const host = document as unknown as { getAnimations?: () => PinnableAnimation[] };
		host.getAnimations = () => animations;
		return () => {
			delete host.getAnimations;
		};
	}

	/** One turn of the task queue — what a deferred pass waits on. */
	const tick = () => new Promise(resolve => setTimeout(resolve, 0));

	it('does nothing at all when the clock is meant to run', () => {
		const animation = stubAnimation(400);
		const restore = withAnimations([animation]);

		holdMotion('live')();

		expect(animation.pause).not.toHaveBeenCalled();
		expect(animation.finish).not.toHaveBeenCalled();
		restore();
	});

	it('pins what is already there', () => {
		const animation = stubAnimation(400);
		const restore = withAnimations([animation]);

		const stop = holdMotion(120);

		expect(animation.currentTime).toBe(120);
		stop();
		restore();
	});

	it('pins an animation created by the recalculation after a mutation', async () => {
		const page: PinnableAnimation[] = [];
		const restore = withAnimations(page);
		const stop = holdMotion(120);

		// The commit: a mutation, with the animation appearing only afterwards — which
		// is the order a real style recalculation puts them in.
		const late = stubAnimation(400);
		document.body.append(document.createElement('div'));
		page.push(late);
		await tick();

		expect(late.currentTime).toBe(120);
		stop();
		restore();
	});

	it('stops pinning once torn down', async () => {
		const page: PinnableAnimation[] = [];
		const restore = withAnimations(page);

		holdMotion(120)();

		const late = stubAnimation(400);
		document.body.append(document.createElement('div'));
		page.push(late);
		await tick();

		expect(late.currentTime).toBe(0);
		restore();
	});
});

describe('applyRate', () => {
	it('sets every animation to the given multiple of real time', () => {
		const animations: RateableAnimation[] = [{ playbackRate: 1 }, { playbackRate: 1 }];

		applyRate(0.2, animations);

		expect(animations.map(animation => animation.playbackRate)).toEqual([0.2, 0.2]);
	});

	it('holds animations where they are at zero', () => {
		const animations: RateableAnimation[] = [{ playbackRate: 1 }];

		applyRate(0, animations);

		expect(animations[0]?.playbackRate).toBe(0);
	});
});

describe('holdRate', () => {
	/** jsdom has no `document.getAnimations`, so the page's animations are stated here. */
	function withAnimations(animations: RateableAnimation[]): () => void {
		const host = document as unknown as { getAnimations?: () => RateableAnimation[] };
		host.getAnimations = () => animations;
		return () => {
			delete host.getAnimations;
		};
	}

	const tick = () => new Promise(resolve => setTimeout(resolve, 0));

	it('does nothing at real time, so nothing is watched for nothing', () => {
		const animation: RateableAnimation = { playbackRate: 1 };
		const restore = withAnimations([animation]);
		const observe = vi.spyOn(MutationObserver.prototype, 'observe');

		holdRate(1)();

		expect(observe).not.toHaveBeenCalled();
		observe.mockRestore();
		restore();
	});

	it('slows what is already there', () => {
		const animation: RateableAnimation = { playbackRate: 1 };
		const restore = withAnimations([animation]);

		const stop = holdRate(0.5);

		expect(animation.playbackRate).toBe(0.5);
		stop();
		restore();
	});

	it('slows an animation created by the recalculation after a mutation', async () => {
		const page: RateableAnimation[] = [];
		const restore = withAnimations(page);
		const stop = holdRate(0.2);

		document.body.appendChild(document.createElement('div'));
		const late: RateableAnimation = { playbackRate: 1 };
		page.push(late);
		await tick();

		expect(late.playbackRate).toBe(0.2);
		stop();
		restore();
	});

	it('stops touching animations once torn down', async () => {
		const page: RateableAnimation[] = [];
		const restore = withAnimations(page);

		holdRate(0.2)();

		document.body.appendChild(document.createElement('div'));
		const late: RateableAnimation = { playbackRate: 1 };
		page.push(late);
		await tick();

		expect(late.playbackRate).toBe(1);
		restore();
	});
});

describe('installFrameClock', () => {
	// eslint-disable-next-line @typescript-eslint/unbound-method -- Reason: kept to put back and compare, never called
	const native = window.requestAnimationFrame;
	// eslint-disable-next-line @typescript-eslint/unbound-method -- Reason: kept to put back and compare, never called
	const nativeCancel = window.cancelAnimationFrame;

	afterEach(() => {
		window.requestAnimationFrame = native;
		window.cancelAnimationFrame = nativeCancel;
	});

	/** jsdom reports `visible`; a capture tab reports `hidden`. */
	function setVisibility(state: DocumentVisibilityState): void {
		Object.defineProperty(document, 'visibilityState', {
			configurable: true,
			get: () => state,
		});
	}

	it('leaves the natives alone while the tab is visible', () => {
		setVisibility('visible');

		const stop = installFrameClock();

		expect(window).toHaveProperty('requestAnimationFrame', native);
		stop();
	});

	it('replaces both halves of the pair while the tab is hidden', () => {
		setVisibility('hidden');

		const stop = installFrameClock();

		expect(window).not.toHaveProperty('requestAnimationFrame', native);
		expect(window).not.toHaveProperty('cancelAnimationFrame', nativeCancel);
		stop();
	});

	it('runs a queued callback, which is the whole point', async () => {
		setVisibility('hidden');
		const stop = installFrameClock();
		const onFrame = vi.fn();

		window.requestAnimationFrame(onFrame);
		await new Promise(resolve => setTimeout(resolve, 0));

		expect(onFrame).toHaveBeenCalledTimes(1);
		expect(typeof onFrame.mock.calls[0]?.[0]).toBe('number');
		stop();
	});

	it('cancels through the same queue it scheduled on', async () => {
		setVisibility('hidden');
		const stop = installFrameClock();
		const onFrame = vi.fn();

		const handle = window.requestAnimationFrame(onFrame);
		window.cancelAnimationFrame(handle);
		await new Promise(resolve => setTimeout(resolve, 0));

		expect(onFrame).not.toHaveBeenCalled();
		stop();
	});

	it('puts the natives back', () => {
		setVisibility('hidden');

		installFrameClock()();

		expect(window).toHaveProperty('requestAnimationFrame', native);
		expect(window).toHaveProperty('cancelAnimationFrame', nativeCancel);
	});
});
