import { describe, expect, it } from 'vitest';

import {
	all,
	carry,
	cursorOf,
	hide,
	only,
	parseBenchSearch,
	shownOf,
	step,
	withOption,
	type BenchSearch,
} from './search';

const IDS = ['edit', 'holding', 'blocked', 'saving', 'deleting'];

describe('parseBenchSearch', () => {
	it('keeps the params a stage has always taken', () => {
		expect(parseBenchSearch({ width: 'tablet', theme: 'light', scale: '0.5', rate: 0 })).toEqual({
			width: 'tablet',
			theme: 'light',
			scale: 0.5,
			rate: 0,
		});
	});

	it('reads the mode and the cursor', () => {
		expect(parseBenchSearch({ show: 'widths', at: 'holding' })).toEqual({
			show: 'widths',
			at: 'holding',
		});
	});

	it('falls back to an empty search on junk rather than throwing', () => {
		expect(parseBenchSearch({ show: 'matrix' })).toEqual({});
	});

	it('drops a value it cannot read alone, and keeps the rest of the address', () => {
		expect(parseBenchSearch({ width: 'wide', theme: 'light', rate: 7, at: 'edit' })).toEqual({
			theme: 'light',
			at: 'edit',
		});
	});
});

describe('carry', () => {
	it('carries every option of the bar to another place', () => {
		const search: BenchSearch = {
			show: 'widths',
			width: 'mobile',
			theme: 'light',
			scale: 1,
			rate: 0.2,
		};

		expect(carry(search)).toEqual(search);
	});

	it('drops what names the states of one place', () => {
		expect(carry({ theme: 'light', variants: 'edit,saving', at: 'saving', frames: 'a:b' })).toEqual(
			{
				theme: 'light',
			}
		);
	});

	it('drops a pin, which is a capture of one frame and not an option', () => {
		expect(carry({ t: 120, rate: 0 })).toEqual({ rate: 0 });
	});

	it('carries whether the parts are outlined, and every value being tried', () => {
		expect(carry({ parts: 'on', try: 'color-accent.dark:%23b8860b' })).toEqual({
			parts: 'on',
			try: 'color-accent.dark:%23b8860b',
		});
	});

	it('carries whether the notes are open, from one candidate to the next', () => {
		expect(carry({ notes: 'on' })).toEqual({ notes: 'on' });
	});

	it('drops the props turned on one state, which mean nothing on another component', () => {
		expect(carry({ args: 'weight:primary', theme: 'light' })).toEqual({ theme: 'light' });
	});

	it('carries a rate of zero, which is off and not absent', () => {
		expect(carry({ rate: 0 })).toEqual({ rate: 0 });
	});

	it('ignores keys it does not own', () => {
		expect(carry({ unrelated: 'x', width: 'tablet' })).toEqual({ width: 'tablet' });
	});
});

describe('shownOf', () => {
	it('shows every state when the address names none', () => {
		expect(shownOf(IDS, {})).toEqual(IDS);
	});

	it('shows the named states in the place’s own order', () => {
		expect(shownOf(IDS, { variants: 'saving,edit' })).toEqual(['edit', 'saving']);
	});

	it('shows every state when none of the named ones exists', () => {
		expect(shownOf(IDS, { variants: 'gone' })).toEqual(IDS);
	});
});

describe('cursorOf', () => {
	it('is the named state', () => {
		expect(cursorOf(IDS, { at: 'blocked' })).toBe('blocked');
	});

	it('is the first shown state when the address names none', () => {
		expect(cursorOf(IDS, { variants: 'saving,deleting' })).toBe('saving');
	});

	it('is the first shown state when the named one is gone', () => {
		expect(cursorOf(IDS, { at: 'gone' })).toBe('edit');
	});
});

describe('step', () => {
	it('solos the cursor first, when several states are showing', () => {
		expect(step(IDS, { at: 'holding' }, 1)).toEqual({ variants: 'holding', at: 'holding' });
	});

	it('moves to the next state once one is showing alone', () => {
		expect(step(IDS, { variants: 'holding', at: 'holding' }, 1)).toEqual({
			variants: 'blocked',
			at: 'blocked',
		});
	});

	it('wraps past the last state and before the first', () => {
		expect(step(IDS, { variants: 'deleting', at: 'deleting' }, 1).at).toBe('edit');
		expect(step(IDS, { variants: 'edit', at: 'edit' }, -1).at).toBe('deleting');
	});

	it('moves the cursor alone in Widths, where one state is always showing', () => {
		expect(step(IDS, { show: 'widths', at: 'edit' }, 1)).toEqual({ show: 'widths', at: 'holding' });
	});

	it('leaves the address alone on a place with one state, where there is nowhere to step', () => {
		expect(step(['holding'], {}, 1)).toEqual({});
		expect(step(['holding'], { theme: 'light' }, -1)).toEqual({ theme: 'light' });
	});

	it('forgets the props turned on the state it leaves', () => {
		expect(step(IDS, { variants: 'edit', at: 'edit', args: 'weight:quiet' }, 1)).toEqual({
			variants: 'holding',
			at: 'holding',
		});
	});

	it('keeps the options of the bar', () => {
		expect(step(IDS, { theme: 'light', width: 'mobile' }, 1)).toMatchObject({
			theme: 'light',
			width: 'mobile',
		});
	});
});

describe('only, hide and all', () => {
	it('shows one state alone, in States', () => {
		expect(only({ show: 'widths', theme: 'light' }, 'saving')).toEqual({
			theme: 'light',
			variants: 'saving',
			at: 'saving',
		});
	});

	it('hides one state and keeps the rest in order', () => {
		expect(hide(IDS, {}, 'blocked')).toEqual({ variants: 'edit,holding,saving,deleting' });
	});

	it('moves the cursor off a state it hides', () => {
		expect(hide(IDS, { at: 'blocked' }, 'blocked')).toEqual({
			variants: 'edit,holding,saving,deleting',
		});
	});

	it('never hides the last state showing', () => {
		expect(hide(IDS, { variants: 'edit' }, 'edit')).toEqual({ variants: 'edit' });
	});

	it('forgets the turned props when it shows another state alone, or all of them', () => {
		expect(only({ args: 'weight:quiet' }, 'saving')).toEqual({ variants: 'saving', at: 'saving' });
		expect(all({ variants: 'edit', args: 'weight:quiet' })).toEqual({});
	});

	it('brings every state back, in States', () => {
		expect(all({ show: 'widths', variants: 'edit', at: 'edit', theme: 'light' })).toEqual({
			at: 'edit',
			theme: 'light',
		});
	});
});

describe('withOption', () => {
	it('writes an option into the address', () => {
		expect(withOption({ at: 'edit' }, { width: 'mobile' })).toEqual({
			at: 'edit',
			width: 'mobile',
		});
	});

	it('leaves a default off the address, so an unadorned one is the stage at rest', () => {
		expect(
			withOption(
				{ show: 'widths', width: 'mobile', theme: 'light', scale: 1, rate: 0 },
				{ show: undefined, width: 'desktop', theme: 'dark', scale: undefined, rate: 1 }
			)
		).toEqual({});
	});

	it('keeps a rate of zero, which is off', () => {
		expect(withOption({}, { rate: 0 })).toEqual({ rate: 0 });
	});
});
