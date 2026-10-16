import { describe, expect, it } from 'vitest';

import { describeStep, matchTargets, parseSteps } from './steps';

describe('parseSteps', () => {
	it('reads a script of steps separated by semicolons', () => {
		expect(parseSteps('click:Open the rules; wait:400; press:Escape')).toEqual([
			{ kind: 'click', target: 'Open the rules' },
			{ kind: 'wait', ms: 400 },
			{ kind: 'press', key: 'Escape' },
		]);
	});

	it('defaults the counted steps rather than requiring a number', () => {
		expect(parseSteps('tab; scroll; wait')).toEqual([
			{ kind: 'tab', times: 1 },
			{ kind: 'scroll', by: 400 },
			{ kind: 'wait', ms: 200 },
		]);
	});

	it('keeps a scroll up negative', () => {
		expect(parseSteps('scroll:-600')).toEqual([{ kind: 'scroll', by: -600 }]);
	});

	it('splits fill at the first equals and keeps the rest of the value', () => {
		expect(parseSteps('fill:Name=Ada = Okonjo')).toEqual([
			{ kind: 'fill', target: 'Name', value: 'Ada = Okonjo' },
		]);
	});

	it('names what it expected when a step is malformed', () => {
		expect(() => parseSteps('poke:thing')).toThrow(/expected click:<name>/);
		expect(() => parseSteps('click:')).toThrow(/click takes the target name/);
		expect(() => parseSteps('fill:Name')).toThrow(/fill takes name=value/);
		expect(() => parseSteps('wait:soon')).toThrow(/is not a number/);
	});

	it('ignores empty steps so a trailing semicolon is harmless', () => {
		expect(parseSteps('click:A;;  ;')).toHaveLength(1);
	});

	it('reads a step back in the words the report prints', () => {
		expect(describeStep({ kind: 'scroll', by: 400 })).toBe('scroll +400px');
		expect(describeStep({ kind: 'tab', times: 3 })).toBe('tab ×3');
	});
});

describe('matchTargets', () => {
	const names = [
		'Configure Spring order, Mar 8',
		'Edit lines for Spring order, Mar 8',
		'Show the lines on Spring order, Front desk and Back office',
		'Configure Front desk',
		'Configure Front desk',
	];

	/**
	 * **A round's failure, as a test.** `Configure Spring order` also prefixes the side column's own
	 * control, the driver took the first match, and the run reported a press that changed nothing
	 * — which is indistinguishable from a state that does not exist. An exact name must never be
	 * outvoted by the names that merely contain it.
	 */
	it('prefers an exact name over anything that merely starts with it', () => {
		expect(matchTargets(names, 'Configure Front desk')).toEqual([3, 4]);
		expect(matchTargets(['Lines', 'Lines for Spring order'], 'Lines')).toEqual([0]);
	});

	it('falls to a glob, then a prefix, then a substring', () => {
		expect(matchTargets(names, 'Show the lines on*')).toEqual([2]);
		expect(matchTargets(names, 'Edit lines')).toEqual([1]);
		expect(matchTargets(names, 'Back office')).toEqual([2]);
	});

	/**
	 * A glob that matches nothing must return nothing. Falling through to the prefix and substring
	 * tiers runs them against a string that still holds the `*`, so `Mar 8*` matched nowhere at all
	 * and the frame was reported as having no such control — a wrong answer wearing a missing one's
	 * clothes.
	 */
	it('stops at the glob tier when the pattern is a glob', () => {
		expect(matchTargets(names, 'Mar 8*')).toEqual([]);
		expect(matchTargets(names, '*Mar 8')).toEqual([0, 1]);
		expect(matchTargets(names, '*Front desk*')).toEqual([2, 3, 4]);
	});

	it('ignores case and collapses whitespace on both sides', () => {
		expect(matchTargets(['Apr 12 – 14  rules'], 'apr 12 – 14 RULES')).toEqual([0]);
	});

	it('returns every match in a tier so the caller can refuse an ambiguous one', () => {
		expect(matchTargets(names, 'Configure Front desk')).toHaveLength(2);
	});

	it('returns nothing when nothing matches', () => {
		expect(matchTargets(names, 'Delete everything')).toEqual([]);
	});
});
