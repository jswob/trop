import { describe, expect, it, vi } from 'vitest';

import { contrastOf, measureParts, over, readColor, toHex } from './measure';

describe('readColor', () => {
	it('reads the forms a browser computes a colour to', () => {
		expect(readColor('rgb(16, 20, 25)')).toEqual([16, 20, 25, 1]);
		expect(readColor('rgba(255, 255, 255, 0.35)')).toEqual([255, 255, 255, 0.35]);
		expect(readColor('rgb(11 18 26 / 0.474)')).toEqual([11, 18, 26, 0.474]);
		expect(readColor('color(srgb 1 0.5 0 / 0.5)')).toEqual([255, 127.5, 0, 0.5]);
	});

	it('reads a hex colour, as a stylesheet declares one', () => {
		expect(readColor('#101419')).toEqual([16, 20, 25, 1]);
		expect(readColor('#fff')).toEqual([255, 255, 255, 1]);
	});

	it('answers nothing for what is not a colour', () => {
		expect(readColor('transparent-ish')).toBeNull();
		expect(readColor('')).toBeNull();
	});
});

describe('over and contrastOf', () => {
	it('lays a see-through colour over a ground', () => {
		expect(over([255, 255, 255, 0.5], [0, 0, 0, 1])).toEqual([127.5, 127.5, 127.5, 1]);
	});

	it('measures black on white at 21 to 1', () => {
		expect(contrastOf([0, 0, 0, 1], [255, 255, 255, 1])).toBeCloseTo(21, 5);
	});

	it('measures a see-through ink against the ground it sits on', () => {
		const half = contrastOf([255, 255, 255, 0.35], [16, 20, 25, 1]);

		expect(half).toBeGreaterThan(3);
		expect(half).toBeLessThan(3.5);
	});

	it('writes a colour as hex', () => {
		expect(toHex([154, 163, 178, 1])).toBe('#9aa3b2');
	});
});

describe('measureParts', () => {
	it('answers for each part that is there, tagged in order, and skips one that is not', () => {
		document.body.innerHTML =
			'<div id="root" style="background: rgb(16, 20, 25)"><button style="padding: 0 12px; font-size: 13px; font-weight: 400; color: rgb(230, 233, 239)"><span>Assign</span></button></div>';
		const root = document.getElementById('root');
		if (root === null) throw new Error('no root');

		const read = measureParts(root, [
			{ name: 'plate', select: 'button' },
			{ name: 'glyph', select: 'svg' },
			{ name: 'word', select: 'button > span' },
		]);

		expect(read.map(part => [part.name, part.tag])).toEqual([
			['plate', 'a'],
			['word', 'b'],
		]);
		expect(read[0]).toMatchObject({
			padding: [0, 12, 0, 12],
			type: '13px, 400',
			ink: '#e6e9ef',
			ground: '#101419',
		});
		expect(read[0]?.contrast).toBeCloseTo(15.2, 0);
	});

	it('reads a drawn shape in what paints it: its fill, or its stroke where it has none', () => {
		document.body.innerHTML =
			'<div id="root" style="background: rgb(16, 20, 25)"><svg><rect id="filled" /><path id="edged" /></svg></div>';
		const root = document.getElementById('root');
		if (root === null) throw new Error('no root');

		const paint: Record<string, { fill: string; stroke: string }> = {
			filled: { fill: 'rgb(201, 159, 36)', stroke: 'none' },
			edged: { fill: 'none', stroke: 'rgb(157, 151, 248)' },
		};
		const real = window.getComputedStyle.bind(window);
		const spy = vi.spyOn(window, 'getComputedStyle').mockImplementation((element, pseudo) => {
			const style = real(element, pseudo);
			const painted = paint[element.id];
			if (painted === undefined) return style;

			return new Proxy(style, {
				get: (target, key): unknown =>
					key === 'fill' || key === 'stroke'
						? painted[key]
						: key === 'color'
							? 'rgb(230, 233, 239)'
							: Reflect.get(target, key, target),
			});
		});

		const read = measureParts(root, [
			{ name: 'filled', select: '#filled' },
			{ name: 'edged', select: '#edged' },
		]);
		spy.mockRestore();

		expect(read.map(part => part.ink)).toEqual(['#c99f24', '#9d97f8']);
		expect(read.map(part => part.type)).toEqual([null, null]);
	});

	it('reads an empty box in the ink of the placeholder it shows', () => {
		document.body.innerHTML =
			'<div id="root" style="background: rgb(16, 20, 25)"><input placeholder="Youth night" style="color: rgb(230, 233, 239)" /></div>';
		const root = document.getElementById('root');
		if (root === null) throw new Error('no root');

		const asked: (string | null | undefined)[] = [];
		const real = window.getComputedStyle.bind(window);
		const spy = vi.spyOn(window, 'getComputedStyle').mockImplementation((element, pseudo) => {
			if (element.tagName === 'INPUT') asked.push(pseudo);
			return real(element, pseudo);
		});

		measureParts(root, [{ name: 'box', select: 'input' }]);
		const box = root.querySelector('input');
		if (box !== null) box.value = 'Youth night';
		measureParts(root, [{ name: 'box', select: 'input' }]);
		spy.mockRestore();

		expect(asked).toContain('::placeholder');
		expect(asked.at(-1)).toBeUndefined();
	});
});
