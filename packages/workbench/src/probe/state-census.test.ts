import { describe, expect, it } from 'vitest';

import {
	conflicts,
	formatCensus,
	hueTokens,
	paintsState,
	stateCensus,
	stripComments,
	unionMembers,
} from './state-census';

describe('unionMembers', () => {
	it('reads a string union off its declaration', () => {
		const source = `export type SlotState = 'confirmed' | 'pending' | 'proposed' | 'open';`;

		expect(unionMembers(source, 'SlotState')).toEqual(['confirmed', 'pending', 'proposed', 'open']);
	});

	it('ignores a union named in prose', () => {
		const source = `/** export type SlotState = 'ghost'; */\nexport type SlotState = 'open';`;

		expect(unionMembers(source, 'SlotState')).toEqual(['open']);
	});

	it('returns nothing rather than guessing when the type is absent', () => {
		expect(unionMembers('export type Other = 1;', 'SlotState')).toEqual([]);
	});
});

describe('hueTokens', () => {
	it('collects each distinct token once', () => {
		const code = 'var(--color-accent) var(--color-accent) var(--color-machine)';

		expect(hueTokens(code, '--color-')).toEqual(['--color-accent', '--color-machine']);
	});
});

describe('stripComments', () => {
	it('drops block and line comments so prose is not a drawing', () => {
		expect(stripComments("/* 'pending' */\nconst a = 1; // 'open'")).not.toContain('pending');
	});
});

describe('paintsState', () => {
	it('reads the hue that follows the state as its value', () => {
		expect(paintsState("pending: 'var(--color-accent)'", 'pending', '--color-')).toBe(true);
	});

	/**
	 * The wolf this rule exists to stop: `kit/Select.tsx` names a scrollbar colour near the top
	 * and takes an `open: boolean` below it, and read as a second ink source for the seat state.
	 */
	it('does not count a hue that merely precedes the state elsewhere in the file', () => {
		const code = "const panel = 'var(--color-edge)';\nfunction usePlacement(open: boolean) {}";

		expect(paintsState(code, 'open', '--color-')).toBe(false);
	});
});

describe('stateCensus', () => {
	const seatMark = ['kit/SeatMark.tsx', "const INK = { pending: 'var(--color-accent)' };"] as const;
	const meter = ['kit/Meter.tsx', "const BAND = { pending: 'var(--color-accent)' };"] as const;
	const sheet = ['views/sheet.tsx', "const state = 'pending';"] as const;

	it('separates a file that paints a state from one that only names it', () => {
		const [drawing] = stateCensus([seatMark, sheet], ['pending'], '--color-');

		expect(drawing).toMatchObject({
			inkSources: ['kit/SeatMark.tsx'],
			mentions: ['views/sheet.tsx'],
		});
	});

	/** The measured failure this whole file exists for. */
	it('flags one state painted from two places', () => {
		const census = stateCensus([seatMark, meter], ['pending'], '--color-');

		expect(conflicts(census).map(drawing => drawing.state)).toEqual(['pending']);
	});

	it('does not flag a state with a single ink source', () => {
		expect(conflicts(stateCensus([seatMark, sheet], ['pending'], '--color-'))).toEqual([]);
	});

	it('reports a state nothing paints', () => {
		const [drawing] = stateCensus([sheet], ['pending'], '--color-');

		expect(drawing?.inkSources).toEqual([]);
	});
});

describe('formatCensus', () => {
	it('names the states that disagree', () => {
		const census = stateCensus(
			[
				['kit/SeatMark.tsx', "'pending' 'var(--color-accent)'"] as const,
				['kit/Meter.tsx', "'pending' 'var(--color-machine)'"] as const,
			],
			['pending'],
			'--color-'
		);

		expect(formatCensus(census)).toContain('two ink sources');
		expect(formatCensus(census)).toContain('painted from more than one place: pending');
	});

	it('says so plainly when every state has one source', () => {
		const census = stateCensus(
			[['kit/SeatMark.tsx', "'open' 'var(--color-ink-3)'"] as const],
			['open'],
			'--color-'
		);

		expect(formatCensus(census)).toContain('every state takes its ink from one place');
	});
});
