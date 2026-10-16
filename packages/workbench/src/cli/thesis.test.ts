import { describe, expect, it } from 'vitest';

import { asks, judge, type FrameReading, type PartReading, type Thesis } from './thesis';

function part(over: Partial<PartReading> = {}): PartReading {
	return {
		width: 320,
		height: 28,
		top: 0,
		right: 320,
		bottom: 28,
		left: 0,
		lines: 1,
		chars: 12,
		room: { top: 0, right: 0, bottom: 0, left: 0 },
		style: {},
		...over,
	};
}

function reading(
	state: string,
	width: FrameReading['width'],
	parts: Record<string, PartReading[]>
): FrameReading {
	return { state, width, parts };
}

const rows = (height: number, count = 3): PartReading[] =>
	Array.from({ length: count }, () => part({ height }));

describe('asks', () => {
	it('finds a part by its data-part name unless the thesis names a selector for it', () => {
		const thesis: Thesis = {
			id: 'panel',
			bet: 'Rows carry it.',
			parts: { plate: 'button' },
			checks: [
				{ part: 'row', expect: { height: 28 } },
				{ part: 'plate', expect: { 'font-weight': '500', height: 32 } },
			],
		};

		expect(asks(thesis)).toEqual([
			{ name: 'row', select: '[data-part~="row"]', props: [] },
			{ name: 'plate', select: 'button', props: ['font-weight'] },
		]);
	});

	it('asks for both ends of a gap, once each', () => {
		const thesis: Thesis = {
			id: 'panel',
			bet: 'Rows carry it.',
			checks: [
				{ gap: ['head', 'row'], is: 16 },
				{ part: 'row', expect: { 'box-shadow': 'none' } },
			],
		};

		expect(asks(thesis)).toEqual([
			{ name: 'head', select: '[data-part~="head"]', props: [] },
			{ name: 'row', select: '[data-part~="row"]', props: ['box-shadow'] },
		]);
	});
});

describe('judge', () => {
	const thesis: Thesis = {
		id: 'panel',
		bet: 'Rows carry it.',
		checks: [{ part: 'row', expect: { height: 28 } }],
	};

	it('holds when every drawing of the part measures what the thesis said', () => {
		const verdicts = judge(thesis, [
			reading('rest', 'mobile', { row: rows(28) }),
			reading('rest', 'desktop', { row: rows(28.3) }),
		]);

		expect(verdicts).toEqual([
			{ what: 'row height', said: '28', ok: true, measured: 2, held: '28 to 28.3', misses: [] },
		]);
	});

	it('names the state, the width and the number where it does not hold', () => {
		const verdicts = judge(thesis, [
			reading('rest', 'mobile', { row: rows(28) }),
			reading('long', 'desktop', { row: [part({ height: 36 }), part({ height: 28 })] }),
		]);

		expect(verdicts[0]?.ok).toBe(false);
		expect(verdicts[0]?.misses).toEqual(['36 on 1 of 2, in long at desktop']);
	});

	it('takes a number for each width, and leaves a width it was not given alone', () => {
		const sized: Thesis = {
			...thesis,
			checks: [{ part: 'row', expect: { height: { mobile: 40, desktop: 32 } } }],
		};

		const verdicts = judge(sized, [
			reading('rest', 'mobile', { row: rows(40) }),
			reading('rest', 'tablet', { row: rows(99) }),
			reading('rest', 'desktop', { row: rows(40) }),
		]);

		expect(verdicts[0]?.said).toBe('40 at mobile, 32 at desktop');
		expect(verdicts[0]?.measured).toBe(2);
		expect(verdicts[0]?.misses).toEqual(['40 on 3 of 3, in rest at desktop']);
	});

	it('fails a part that no state draws, since a check on nothing passes for ever', () => {
		const verdicts = judge(thesis, [reading('rest', 'mobile', { row: [] })]);

		expect(verdicts[0]).toMatchObject({
			ok: false,
			measured: 0,
			misses: ['no state draws a part called "row"'],
		});
	});

	it('skips a state that does not draw the part, unless the check names that state', () => {
		const named: Thesis = {
			...thesis,
			checks: [{ part: 'row', in: ['empty'], expect: { height: 28 } }],
		};
		const readings = [
			reading('rest', 'mobile', { row: rows(28) }),
			reading('empty', 'mobile', { row: [] }),
		];

		expect(judge(thesis, readings)[0]?.ok).toBe(true);
		expect(judge(named, readings)[0]?.misses).toEqual(['not drawn, in empty at mobile']);
	});

	it('counts the drawings, reads lines and characters, and holds a number to a range', () => {
		const counted: Thesis = {
			...thesis,
			checks: [
				{ part: 'row', expect: { count: 3, lines: 1, chars: { max: 10 } } },
				{ part: 'row', expect: { count: { min: 4 } } },
			],
		};

		const verdicts = judge(counted, [reading('rest', 'mobile', { row: rows(28) })]);

		expect(verdicts.map(verdict => [verdict.what, verdict.ok])).toEqual([
			['row count', true],
			['row lines', true],
			['row chars', false],
			['row count', false],
		]);
		expect(verdicts[2]?.said).toBe('at most 10');
		expect(verdicts[2]?.misses).toEqual(['12 on 3 of 3, in rest at mobile']);
		expect(verdicts[3]?.misses).toEqual(['3, in rest at mobile']);
	});

	it('reads any other measure as the computed style of that name', () => {
		const styled: Thesis = {
			...thesis,
			checks: [
				{ part: 'row', expect: { 'box-shadow': 'none', 'font-size': 13, 'font-weight': 500 } },
			],
		};

		const verdicts = judge(styled, [
			reading('rest', 'mobile', {
				row: [
					part({
						style: {
							'box-shadow': 'rgba(0, 0, 0, 0.1) 0px 1px 2px 0px',
							'font-size': '13px',
							'font-weight': '500',
						},
					}),
				],
			}),
		]);

		expect(verdicts.map(verdict => verdict.ok)).toEqual([false, true, true]);
		expect(verdicts[0]?.misses).toEqual([
			'rgba(0, 0, 0, 0.1) 0px 1px 2px 0px on 1 of 1, in rest at mobile',
		]);
	});

	it('measures the gap from the foot of one part to the head of the next, or across', () => {
		const spaced: Thesis = {
			...thesis,
			checks: [
				{ gap: ['head', 'row'], is: 16 },
				{ gap: ['glyph', 'word'], across: true, is: 8 },
			],
		};

		const verdicts = judge(spaced, [
			reading('rest', 'mobile', {
				head: [part({ top: 0, bottom: 20 })],
				row: [part({ top: 44, bottom: 72 })],
				glyph: [part({ left: 12, right: 28 })],
				word: [part({ left: 36, right: 80 })],
			}),
		]);

		expect(verdicts).toEqual([
			{
				what: 'head to row',
				said: '16',
				ok: false,
				measured: 1,
				held: '',
				misses: ['24, in rest at mobile'],
			},
			{ what: 'glyph to word', said: '8', ok: true, measured: 1, held: '8', misses: [] },
		]);
	});

	it('counts a part’s own padding as room, where nothing is drawn on that side of it', () => {
		const spaced: Thesis = { ...thesis, checks: [{ gap: ['head', 'list'], is: 24 }] };

		const verdicts = judge(spaced, [
			reading('rest', 'mobile', {
				head: [part({ top: 0, bottom: 60, room: { top: 0, right: 0, bottom: 16, left: 0 } })],
				list: [part({ top: 60, bottom: 200, room: { top: 8, right: 0, bottom: 0, left: 0 } })],
			}),
		]);

		expect(verdicts[0]).toMatchObject({ ok: true, held: '24' });
	});
});
