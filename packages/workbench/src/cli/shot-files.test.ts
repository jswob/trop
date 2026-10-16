import { describe, expect, it } from 'vitest';

import type { FrameEntry } from '../lib/frame-census';

import {
	answerName,
	assertShotsDir,
	compareSets,
	diffLines,
	packAnswer,
	shotName,
	shotsNamed,
	shotsOf,
	steadied,
	unpackAnswer,
} from './shot-files';

const entry = (id: string, variant: string, kind: FrameEntry['kind'] = 'view'): FrameEntry => ({
	id,
	kind,
	title: id,
	variant,
	variantLabel: variant,
	chrome: 'bare',
});

describe('shotName', () => {
	it('names a shot by its space, frame, state, width and appearance', () => {
		expect(shotName({ entry: entry('people', 'grid'), width: 'mobile', theme: 'light' })).toBe(
			'view.people.grid.mobile.light.png'
		);
	});
});

describe('shotsOf', () => {
	it('takes every frame at every width in every appearance, in the census’s order', () => {
		const shots = shotsOf(
			[entry('people', 'grid'), entry('button', 'rest', 'kit')],
			['mobile', 'desktop'],
			['dark', 'light']
		);

		expect(shots.map(shotName)).toEqual([
			'view.people.grid.mobile.dark.png',
			'view.people.grid.mobile.light.png',
			'view.people.grid.desktop.dark.png',
			'view.people.grid.desktop.light.png',
			'kit.button.rest.mobile.dark.png',
			'kit.button.rest.mobile.light.png',
			'kit.button.rest.desktop.dark.png',
			'kit.button.rest.desktop.light.png',
		]);
	});

	it('refuses two frames that would be written to one file', () => {
		expect(() =>
			shotsOf([entry('people.grid', 'long'), entry('people', 'grid.long')], ['mobile'], ['dark'])
		).toThrow(/view\.people\.grid\.long\.mobile\.dark\.png/);
	});

	it('refuses a name no folder can hold', () => {
		expect(() => shotsOf([entry('people', 'a/b')], ['mobile'], ['dark'])).toThrow(/people:a\/b/);
	});
});

describe('assertShotsDir', () => {
	const repo = '/work/ledger';

	it('takes a folder outside the repo', () => {
		expect(assertShotsDir('/tmp/job/before', repo)).toBe('/tmp/job/before');
	});

	it('refuses a folder inside the repo, and a missing name', () => {
		expect(() => assertShotsDir(`${repo}/apps/ledger/design/shots`, repo)).toThrow(
			/outside the repo/
		);
		expect(() => assertShotsDir(repo, repo)).toThrow(/outside the repo/);
		expect(() => assertShotsDir(undefined, repo)).toThrow(/name a folder/);
	});
});

describe('compareSets', () => {
	const before = new Map([
		['a.png', '1'],
		['b.png', '2'],
		['c.png', '3'],
	]);

	it('finds nothing between two sets that are the same', () => {
		expect(compareSets(before, new Map(before), { whole: true })).toEqual({
			same: 3,
			differs: [],
			missing: [],
			added: [],
		});
	});

	it('names what differs, what is no longer taken and what is new', () => {
		const after = new Map([
			['a.png', '1'],
			['b.png', 'changed'],
			['d.png', '4'],
		]);

		expect(compareSets(before, after, { whole: true })).toEqual({
			same: 1,
			differs: ['b.png'],
			missing: ['c.png'],
			added: ['d.png'],
		});
	});

	it('does not call a shot missing when the run was pointed at part of the set', () => {
		const after = new Map([['a.png', '1']]);

		expect(compareSets(before, after, { whole: false }).missing).toEqual([]);
	});
});

describe('diffLines', () => {
	it('prints one line per file and the counts', () => {
		const lines = diffLines({ same: 4, differs: ['b.png'], missing: ['c.png'], added: [] });

		expect(lines).toEqual([
			'  differs  b.png',
			'  missing  c.png',
			'',
			'  6 shots · 4 identical · 1 differs · 1 missing · 0 added',
		]);
	});

	it('prints the counts alone when nothing differs', () => {
		expect(diffLines({ same: 2, differs: [], missing: [], added: [] })).toEqual([
			'  2 shots · 2 identical · 0 differ · 0 missing · 0 added',
		]);
	});

	it('names a shot that only matched on a later take, without counting it as a difference', () => {
		expect(diffLines({ same: 2, differs: [], missing: [], added: [] }, ['a.png'])).toEqual([
			'  unsteady a.png (not the same on the first take, the same on a later one)',
			'',
			'  2 shots · 2 identical · 0 differ · 0 missing · 0 added',
		]);
	});
});

describe('shotsNamed', () => {
	it('picks the shots that are written to the named files, in the set’s order', () => {
		const set = shotsOf([entry('people', 'grid'), entry('people', 'empty')], ['mobile'], ['dark']);

		expect(
			shotsNamed(set, ['view.people.empty.mobile.dark.png', 'view.nobody.x.mobile.dark.png'])
		).toEqual([{ entry: entry('people', 'empty'), width: 'mobile', theme: 'dark' }]);
	});
});

describe('steadied', () => {
	it('names what differed on the first take and no longer does', () => {
		expect(
			steadied(
				{ same: 1, differs: ['a.png', 'b.png'], missing: [], added: [] },
				{ same: 2, differs: ['b.png'], missing: [], added: [] }
			)
		).toEqual(['a.png']);
	});
});

describe('answerName', () => {
	it('gives an address one file name, whatever is in it', () => {
		const css = 'https://fonts.example/css2?family=Archivo:wght@400;500&display=swap';

		expect(answerName(css)).toMatch(/^[0-9a-f]{32}\.json$/);
		expect(answerName(css)).toBe(answerName(css));
		expect(answerName(css)).not.toBe(answerName(`${css}&text=a`));
	});
});

describe('packAnswer', () => {
	it('keeps what a host answered, and comes back as it went in', () => {
		const body = Buffer.from([0, 255, 16, 77]);
		const packed = packAnswer({
			url: 'https://fonts.example/a.woff2',
			status: 200,
			headers: { 'content-type': 'font/woff2', 'access-control-allow-origin': '*' },
			body,
		});

		expect(unpackAnswer(packed)).toEqual({
			url: 'https://fonts.example/a.woff2',
			status: 200,
			headers: { 'content-type': 'font/woff2', 'access-control-allow-origin': '*' },
			body,
		});
	});

	it('drops the headers that describe a body as it travelled, not as it is kept', () => {
		const packed = packAnswer({
			url: 'https://fonts.example/css2',
			status: 200,
			headers: {
				'Content-Type': 'text/css',
				'Content-Encoding': 'br',
				'content-length': '412',
				'transfer-encoding': 'chunked',
			},
			body: Buffer.from('@font-face {}'),
		});

		expect(unpackAnswer(packed).headers).toEqual({ 'Content-Type': 'text/css' });
	});
});
