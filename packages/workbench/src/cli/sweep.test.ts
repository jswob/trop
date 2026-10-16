import { describe, expect, it } from 'vitest';

import type { FrameReport } from '../probe/report';

import type { Target } from './inspect';
import { grouped, hold, judgePress, only, pick, scrolled, type PressFacts } from './sweep';

function target(name: string, over: Partial<Target> = {}): Target {
	return {
		role: 'button',
		name,
		tag: 'button',
		disabled: false,
		visible: true,
		x: 0,
		y: 0,
		w: 10,
		h: 10,
		cx: 0,
		cy: 0,
		press: 'press',
		like: `button||${name}`,
		...over,
	};
}

const facts = (over: Partial<PressFacts> = {}): PressFacts => ({
	name: 'Save',
	moved: true,
	focusOnBody: false,
	sideways: 0,
	errors: [],
	opened: null,
	...over,
});

describe('pick', () => {
	it('takes what a press is a fair question of, and leaves the rest', () => {
		const picked = pick(
			[
				target('Save'),
				target('Off', { disabled: true }),
				target('Phone only', { visible: false }),
				target('Dark', { press: 'chosen' }),
				target('Name', { press: 'no' }),
				target('Discard'),
			],
			false
		);

		expect(picked.presses.map(press => [press.index, press.name])).toEqual([
			[0, 'Save'],
			[5, 'Discard'],
		]);
		expect(picked.alike).toBe(0);
	});

	it('keeps two controls of one name apart by the order they stand in', () => {
		const picked = pick([target('Configure'), target('Configure')], false);

		expect(picked.presses.map(press => press.key)).toEqual(['Configure#1', 'Configure#2']);
	});

	it('presses three of a long run of alike controls, and says how many it left', () => {
		const rows = Array.from({ length: 20 }, (_row, index) =>
			target(`Person ${String(index)}`, { like: 'a||row' })
		);

		const picked = pick([target('Add'), ...rows], false);

		expect(picked.presses.map(press => press.name)).toEqual([
			'Add',
			'Person 0',
			'Person 10',
			'Person 19',
		]);
		expect(picked.alike).toBe(17);
	});

	it('presses every one of a short run, since seven destinations are seven answers', () => {
		const cells = Array.from({ length: 7 }, (_row, index) =>
			target(`Place ${String(index)}`, { like: 'a||cell' })
		);

		expect(pick(cells, false).presses).toHaveLength(7);
	});

	it('presses the whole run when asked for every one', () => {
		const rows = Array.from({ length: 20 }, (_row, index) =>
			target(`Person ${String(index)}`, { like: 'a||row' })
		);

		expect(pick(rows, true)).toMatchObject({ alike: 0 });
		expect(pick(rows, true).presses).toHaveLength(20);
	});

	it('counts a control with no name, which nothing can press by asking for it', () => {
		expect(pick([target(''), target('Save')], false).unnamed).toBe(1);
	});
});

describe('judgePress', () => {
	it('finds nothing in a press that moved the page and kept the keyboard', () => {
		expect(judgePress(facts())).toEqual([]);
	});

	it('says so when a press moved nothing', () => {
		expect(judgePress(facts({ moved: false }))).toEqual(['"Save" moved nothing']);
	});

	it('says so when the keyboard is left on the page body', () => {
		expect(judgePress(facts({ focusOnBody: true }))).toEqual([
			'"Save" left the keyboard on the page body',
		]);
	});

	it('says so when the page scrolls sideways afterwards, and when the press threw', () => {
		expect(judgePress(facts({ sideways: 14, errors: ['x is not a function'] }))).toEqual([
			'"Save" made the page scroll sideways by 14px',
			'"Save" threw: x is not a function',
		]);
	});

	it('holds what a press opened to Escape, and to handing the keyboard back', () => {
		expect(judgePress(facts({ opened: { closed: false, focusBack: false } }))).toEqual([
			'"Save" opened something Escape does not close',
		]);
		expect(judgePress(facts({ opened: { closed: true, focusBack: false } }))).toEqual([
			'"Save" opened something, and closing it did not put the keyboard back on it',
		]);
		expect(judgePress(facts({ opened: { closed: true, focusBack: true } }))).toEqual([]);
	});

	it('does not blame the keyboard on a press whose opening still stands', () => {
		expect(
			judgePress(facts({ focusOnBody: true, opened: { closed: true, focusBack: false } }))
		).toEqual(['"Save" opened something, and closing it did not put the keyboard back on it']);
	});
});

describe('grouped', () => {
	const report = (variant: string, widthName: string, findings: string[]): FrameReport => ({
		id: 'account',
		variant,
		width: 0,
		widthName,
		results: [{ probe: 'press', ok: findings.length === 0, note: '', findings }],
	});

	it('says one finding once, with every state and width it was found in', () => {
		const lines = grouped(
			[
				report('not-set-up', 'mobile', ['"Sign in" left the keyboard on the page body']),
				report('not-set-up', 'desktop', ['"Sign in" left the keyboard on the page body']),
				report('unreachable', 'mobile', ['"Sign in" left the keyboard on the page body']),
				report('unreachable', 'desktop', ['"Sign in" left the keyboard on the page body']),
				report('account', 'mobile', ['"Back" moved nothing']),
				report('account', 'desktop', []),
			],
			['mobile', 'desktop'],
			false
		);

		expect(lines).toEqual([
			'account',
			'  FAIL  "Sign in" left the keyboard on the page body',
			'          not-set-up, unreachable · every width',
			'  FAIL  "Back" moved nothing',
			'          account · mobile',
		]);
	});

	it('lists each place when a finding is not in every state at every width it touches', () => {
		const lines = grouped(
			[
				report('edit', 'mobile', ['"Save" moved nothing']),
				report('holding', 'desktop', ['"Save" moved nothing']),
			],
			['mobile', 'desktop'],
			false
		);

		expect(lines[2]).toBe('          edit at mobile, holding at desktop');
	});

	it('prints a finding the baseline holds only when asked for everything', () => {
		const held: FrameReport = {
			...report('edit', 'mobile', ['"Save" moved nothing']),
			results: [{ probe: 'press', ok: true, note: '', findings: ['"Save" moved nothing'] }],
		};

		expect(grouped([held], ['mobile'], false)).toEqual([]);
		expect(grouped([held], ['mobile'], true)[1]).toBe('  held  "Save" moved nothing');
	});
});

describe('hold', () => {
	const report = (findings: string[]): FrameReport => ({
		id: 'people',
		variant: 'grid',
		width: 390,
		widthName: 'mobile',
		results: [{ probe: 'press', ok: findings.length === 0, note: '', findings }],
	});
	const key = (finding: string): string => `people·grid·mobile·press·${finding}`;

	it('holds each finding on its own, so one new fault does not reprint the held ones as failing', () => {
		const held = hold(
			[report(['"Configure" moved nothing', '"Sign in" lost the keyboard'])],
			[key('"Configure" moved nothing')]
		);

		expect(held.held).toBe(1);
		expect(held.reports[0]?.results).toEqual([
			{ probe: 'press', ok: false, note: '', findings: ['"Sign in" lost the keyboard'] },
			{ probe: 'press', ok: true, note: '', findings: ['"Configure" moved nothing'] },
		]);
	});

	it('reports a held finding that no longer occurs, on a frame this run swept', () => {
		const held = hold(
			[report([])],
			[key('"Configure" moved nothing'), 'tasks·task·mobile·press·x']
		);

		expect(held.resolved).toEqual([key('"Configure" moved nothing')]);
	});
});

describe('only', () => {
	it('keeps the findings a pattern matches and drops the rest, for a partial accept', () => {
		const kept = only(
			[
				{
					id: 'people',
					variant: 'grid',
					width: 390,
					widthName: 'mobile',
					results: [
						{
							probe: 'press',
							ok: false,
							note: '',
							findings: ['"Configure Mar 8" moved nothing', '"Sign in" lost the keyboard'],
						},
					],
				},
			],
			/^"Configure .*" moved nothing$/
		);

		expect(kept[0]?.results[0]?.findings).toEqual(['"Configure Mar 8" moved nothing']);
	});
});

describe('scrolled', () => {
	const at = (name: string, y: number, cy: number): Target => ({ ...target(name), y, cy });

	it('sees a column scrolled under a press, which moves no content at all', () => {
		expect(
			scrolled(
				[at('Mar 8', 100, 100), at('Jun 7', 900, 900)],
				[at('Mar 8', -500, 100), at('Jun 7', 300, 900)]
			)
		).toBe(true);
	});

	it('sees nothing in a frame that stood still, or shifted by less than a line', () => {
		expect(scrolled([at('Mar 8', 100, 100)], [at('Mar 8', 106, 100)])).toBe(false);
	});
});
