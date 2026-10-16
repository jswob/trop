import { describe, expect, it } from 'vitest';

import { moved, positions, report, shifted } from './drive';
import type { FrameRead, Target } from './inspect';

function target(name: string, cx: number, cy: number): Target {
	return {
		role: 'button',
		name,
		tag: 'button',
		disabled: false,
		visible: true,
		x: cx,
		y: cy,
		w: 10,
		h: 10,
		cx,
		cy,
		press: 'press',
		like: 'button||',
	};
}

function frame(targets: Target[]): FrameRead {
	return {
		chars: 0,
		words: 0,
		boxes: 0,
		scrollY: 0,
		dialog: null,
		columns: 1,
		theme: 'dark',
		extent: { x: 0, right: 0, y: 0, bottom: 0 },
		targets,
	};
}

describe('moved', () => {
	it('says nothing moved between two reads that are the same', () => {
		expect(moved(frame([]), frame([]), false)).toBe(false);
	});

	/** `2 of 12 picked` cleared to `0 of 12 picked` is the same count of characters. */
	it('sees words swapped for others of the same length', () => {
		expect(moved(frame([]), { ...frame([]), words: 7 }, false)).toBe(true);
	});
});

describe('positions', () => {
	/**
	 * The running order draws eight buttons called `Configure Lead guitar`. Keyed by name alone
	 * they collapse to one, and a reflow that moved all eight reports as having moved nothing.
	 */
	it('keeps repeated names apart by ordinal', () => {
		const places = positions(
			[target('Configure Lead guitar', 0, 10), target('Configure Lead guitar', 0, 40)],
			false
		);

		expect([...places.keys()]).toEqual(['Configure Lead guitar#1', 'Configure Lead guitar#2']);
	});

	it('ignores targets with no accessible name', () => {
		expect(positions([target('', 0, 0), target('Named', 0, 0)], false).size).toBe(1);
	});

	it('skips what is not drawn at this width unless asked for all', () => {
		const hidden: Target = { ...target('Phone only', 0, 0), visible: false };

		expect(positions([hidden], false).size).toBe(0);
		expect(positions([hidden], true).size).toBe(1);
	});
});

describe('shifted', () => {
	it('counts a reflow and reports the largest shift', () => {
		const before = frame([target('a', 100, 0), target('b', 100, 40)]);
		const after = frame([target('a', 700, 0), target('b', 200, 40)]);

		expect(shifted(before, after, false)).toEqual({ count: 2, most: 600 });
	});

	/**
	 * **A scroll is not a reflow.** A click has to bring its target into view first, so pressing
	 * anything near the foot of a column scrolls that column. Read in viewport coordinates an inert
	 * button reported 67 targets moved and the run concluded the press reached a state — which is
	 * the exact wrong answer, on the exact control this rule turns on. `readFrame` adds the scroll
	 * offsets back, so these are content coordinates and a scroll moves nothing in them.
	 */
	it('sees nothing when only the reader moved', () => {
		const targets = [target('a', 100, 0), target('b', 100, 900)];

		expect(shifted(frame(targets), frame(targets), false)).toEqual({ count: 0, most: 0 });
	});

	/**
	 * A line's worth, the same floor the overflow probe uses. Sub-pixel rounding, a sticky row and
	 * a scrollbar appearing each move a box a few pixels; a tool that calls those a state change is
	 * one nobody believes. Measured: an inert control sat at 8px of residue.
	 */
	it('ignores a shift smaller than a line', () => {
		expect(shifted(frame([target('a', 0, 0)]), frame([target('a', 0, 8)]), false)).toEqual({
			count: 0,
			most: 0,
		});
		expect(shifted(frame([target('a', 0, 0)]), frame([target('a', 0, 20)]), false).count).toBe(1);
	});

	it('says nothing about a target that arrived or left — that is the appeared/gone lines', () => {
		expect(shifted(frame([target('a', 0, 0)]), frame([target('b', 0, 500)]), false)).toEqual({
			count: 0,
			most: 0,
		});
	});
});

describe('report', () => {
	it('counts a repaint of the whole frame as a state reached, though nothing moved', () => {
		const before = frame([]);
		const after = { ...before, theme: 'light' as const };

		const text = report(before, after, false);
		expect(text).toContain('theme      dark → light');
		expect(text).toContain('RESULT: the press reached a state.');
	});

	it('still calls a press that changed nothing inert', () => {
		expect(report(frame([]), frame([]), false)).toContain('RESULT: NOTHING MOVED');
	});
});
