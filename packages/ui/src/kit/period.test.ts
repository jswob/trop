import { describe, expect, it } from 'vitest';

import { endWords, inPeriod, periodWords, setEnd, sortPeriods } from './period';

describe('a period in words', () => {
	it('says both ends, an open one in words', () => {
		expect(periodWords({ from: '2026-05-01', to: null })).toBe('May 1 – no end');
		expect(periodWords({ from: null, to: '2026-04-30' })).toBe('no start – Apr 30');
		expect(periodWords({ from: '2026-03-01', to: '2026-03-31' })).toBe('Mar 1 – Mar 31');
	});

	it('says one open end in words', () => {
		expect(endWords({ from: '2026-05-01', to: null }, 'from')).toBe('May 1');
		expect(endWords({ from: '2026-05-01', to: null }, 'to')).toBe('no end');
		expect(endWords({ from: null, to: '2026-04-30' }, 'from')).toBe('no start');
	});
});

describe('a period changed', () => {
	it('sets the end that was pressed, and pulls the other to meet it rather than run backwards', () => {
		expect(setEnd({ from: '2026-03-01', to: '2026-03-31' }, 'to', '2026-03-20')).toEqual({
			from: '2026-03-01',
			to: '2026-03-20',
		});
		expect(setEnd({ from: '2026-03-01', to: '2026-03-31' }, 'from', '2026-04-10')).toEqual({
			from: '2026-04-10',
			to: '2026-04-10',
		});
		expect(setEnd({ from: '2026-05-01', to: '2026-06-30' }, 'to', null)).toEqual({
			from: '2026-05-01',
			to: null,
		});
	});

	it('sorts an open start first', () => {
		expect(
			sortPeriods([
				{ from: '2026-05-01', to: null },
				{ from: null, to: '2026-03-31' },
			])
		).toEqual([
			{ from: null, to: '2026-03-31' },
			{ from: '2026-05-01', to: null },
		]);
	});

	it('runs an open end on for ever', () => {
		expect(inPeriod({ from: null, to: '2026-04-30' }, '2020-01-01')).toBe(true);
		expect(inPeriod({ from: '2026-05-01', to: null }, '2026-04-30')).toBe(false);
		expect(inPeriod({ from: '2026-03-01', to: '2026-03-31' }, '2026-03-31')).toBe(true);
	});
});
