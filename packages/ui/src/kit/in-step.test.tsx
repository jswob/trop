import { render } from '@testing-library/react';
import { useRef } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { useInStep } from './in-step';

type Fake = { animationName: string; playState: string; startTime: number | null };

function Field({ active }: { active: boolean }) {
	const root = useRef<HTMLDivElement>(null);
	useInStep(root, 'ui-breath', active);
	return <div ref={root} />;
}

function withAnimations(animations: Fake[]) {
	vi.spyOn(Element.prototype, 'getAnimations').mockReturnValue(
		animations as unknown as Animation[]
	);
}

describe('useInStep', () => {
	afterEach(() => {
		vi.restoreAllMocks();
	});

	it('puts every running copy of the animation on the first one’s start time', () => {
		const first = { animationName: 'ui-breath', playState: 'running', startTime: 1200 };
		const late = { animationName: 'ui-breath', playState: 'running', startTime: 5300 };
		withAnimations([first, late]);

		render(<Field active />);

		expect(late.startTime).toBe(1200);
		expect(first.startTime).toBe(1200);
	});

	it('leaves other animations, paused copies and an inactive field alone', () => {
		const first = { animationName: 'ui-breath', playState: 'running', startTime: 1200 };
		const other = { animationName: 'ui-glaze', playState: 'running', startTime: 800 };
		const pinned = { animationName: 'ui-breath', playState: 'paused', startTime: null };
		withAnimations([first, other, pinned]);

		const { rerender } = render(<Field active={false} />);
		expect(first.startTime).toBe(1200);

		rerender(<Field active />);
		expect(other.startTime).toBe(800);
		expect(pinned.startTime).toBeNull();
	});

	it('starts a pending copy on the timeline’s time when none has begun', () => {
		const pending = { animationName: 'ui-breath', playState: 'running', startTime: null };
		withAnimations([pending]);
		Object.defineProperty(document, 'timeline', {
			value: { currentTime: 4000 },
			configurable: true,
		});

		render(<Field active />);
		Reflect.deleteProperty(document, 'timeline');

		expect(pending.startTime).toBe(4000);
	});
});
