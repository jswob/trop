import { describe, expect, it } from 'vitest';

import { statesFit, widthsFit } from './fit';

const DESKTOP = { width: 1440, height: 900 };
const PHONE = { width: 390, height: 844 };

describe('statesFit', () => {
	it('lays five desktop frames three across in the room the stage has at 1440', () => {
		const layout = statesFit({
			available: { width: 1207, height: 818 },
			frame: DESKTOP,
			count: 5,
			gap: 10,
			captionHeight: 22,
			across: 'fit',
		});

		expect(layout.columns).toBe(3);
		expect(layout.frameWidth).toBe(395);
	});

	it('puts phone frames in one row when a row is what shows them largest', () => {
		const layout = statesFit({
			available: { width: 1207, height: 818 },
			frame: PHONE,
			count: 5,
			gap: 10,
			captionHeight: 22,
			across: 'fit',
		});

		expect(layout.columns).toBe(5);
	});

	it('gives one frame the whole stage', () => {
		const layout = statesFit({
			available: { width: 1207, height: 818 },
			frame: DESKTOP,
			count: 1,
			gap: 10,
			captionHeight: 22,
			across: 'fit',
		});

		expect(layout.columns).toBe(1);
		expect(layout.frameWidth).toBe(1207);
	});

	it('never draws a frame larger than it is', () => {
		const layout = statesFit({
			available: { width: 1207, height: 2000 },
			frame: PHONE,
			count: 1,
			gap: 10,
			captionHeight: 22,
			across: 'fit',
		});

		expect(layout.frameWidth).toBe(390);
	});

	it('fills two columns when asked for two across, and lets the stage scroll', () => {
		const layout = statesFit({
			available: { width: 1207, height: 818 },
			frame: DESKTOP,
			count: 5,
			gap: 10,
			captionHeight: 22,
			across: 'two',
		});

		expect(layout.columns).toBe(2);
		expect(layout.frameWidth).toBe(598);
	});

	it('fills one column when asked for one across', () => {
		const layout = statesFit({
			available: { width: 1207, height: 818 },
			frame: DESKTOP,
			count: 5,
			gap: 10,
			captionHeight: 22,
			across: 'one',
		});

		expect(layout.columns).toBe(1);
		expect(layout.frameWidth).toBe(1207);
	});

	it('stops shrinking at the narrowest a frame may be, and scrolls instead', () => {
		const layout = statesFit({
			available: { width: 400, height: 300 },
			frame: DESKTOP,
			count: 11,
			gap: 10,
			captionHeight: 22,
			across: 'fit',
		});

		expect(layout.frameWidth).toBe(160);
		expect(layout.columns).toBe(2);
	});

	it('answers with one true-size frame before the stage has been measured', () => {
		const layout = statesFit({
			available: { width: 0, height: 0 },
			frame: DESKTOP,
			count: 5,
			gap: 10,
			captionHeight: 22,
			across: 'fit',
		});

		expect(layout).toEqual({ columns: 1, frameWidth: 1440 });
	});
});

describe('widthsFit', () => {
	const frames = [PHONE, { width: 834, height: 1112 }, DESKTOP];

	it('draws the three widths at one scale, bound by the row they share', () => {
		const scale = widthsFit({
			available: { width: 1207, height: 818 },
			frames,
			gap: 10,
			captionHeight: 22,
		});

		expect(scale).toBeCloseTo((1207 - 20) / (390 + 834 + 1440), 5);
	});

	it('is bound by the tallest frame when the stage is short', () => {
		const scale = widthsFit({
			available: { width: 4000, height: 578 },
			frames,
			gap: 10,
			captionHeight: 22,
		});

		expect(scale).toBeCloseTo((578 - 22) / 1112, 5);
	});

	it('never draws a frame larger than it is', () => {
		expect(
			widthsFit({ available: { width: 9000, height: 9000 }, frames, gap: 10, captionHeight: 22 })
		).toBe(1);
	});
});
