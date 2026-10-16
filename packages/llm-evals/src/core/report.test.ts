import { describe, expect, test } from 'vitest';

import { checkIntegrity, EXIT, integrityLine } from './integrity.ts';
import { fixtureRecord, type FixtureRow } from './record.fixture.ts';
import { parseRecord } from './record.ts';
import { buildReport, renderAnswers, renderReport, verdictOf } from './report.ts';
import { median, pairedDifference, wilson } from './stats.ts';

const A = 'champion · a';
const B = 'b';
const rows = (variant: string, column: number, samples: string[]): FixtureRow[] =>
	samples.map((s, i) => ({ variant, column, caseId: `case:${i}`, samples: s }));

describe('parseRecord', () => {
	test('reads rows, columns in order, and the run tags', () => {
		const record = parseRecord(fixtureRecord([...rows(B, 1, ['ccc']), ...rows(A, 0, ['ccw'])]));
		expect(record.variants).toEqual([A, B]);
		expect(record.tags).toEqual({ run: 'r1' });
		expect(record.rows[1]).toMatchObject({ variant: A, caseId: 'case:0', state: 'fail' });
		expect(record.rows[1]?.samples).toHaveLength(3);
	});

	test('keeps a technical error apart from a wrong answer', () => {
		const record = parseRecord(
			fixtureRecord([
				{
					variant: A,
					caseId: 'case:0',
					error: { failure: 'transient', message: 'OpenRouter 503' },
				},
			])
		);
		expect(record.rows[0]).toMatchObject({
			state: 'error',
			failure: 'transient',
			error: 'OpenRouter 503',
		});
	});

	test('reads the scorer’s own lines on a passing row, which promptfoo overwrites at the top', () => {
		const record = parseRecord(fixtureRecord([{ variant: A, caseId: 'case:0', samples: 'ccc' }]));
		expect(record.rows[0]?.reason).toMatch(/^1\. correct/);
	});

	test('refuses anything that is not a version 3 export', () => {
		expect(() => parseRecord({ results: { version: 2 } })).toThrow(/version 3/);
	});
});

describe('checkIntegrity', () => {
	const record = (extra: FixtureRow[]) =>
		parseRecord(fixtureRecord([...rows(A, 0, ['ccc', 'ccw']), ...extra]));

	test('a run with every row scored is trusted, whatever the answers were', () => {
		expect(checkIntegrity(record([]), 2)).toMatchObject({ scored: 2, state: 'ok' });
	});

	test('technical errors above 1% make it untrusted', () => {
		const i = checkIntegrity(
			record([{ variant: A, caseId: 'case:2', error: { failure: 'transient' } }]),
			3
		);
		expect(i).toMatchObject({ retryable: 1, state: 'untrusted' });
		expect(EXIT[i.state]).toBe(3);
	});

	test('rows stopped by the spending limit are not errors', () => {
		const i = checkIntegrity(
			record([{ variant: A, caseId: 'case:2', error: { failure: 'not_run' } }]),
			3
		);
		expect(i).toMatchObject({ notRun: 1, retryable: 0, state: 'over_budget' });
	});

	test('missing rows or a fatal failure mean the run was cut short', () => {
		expect(checkIntegrity(record([]), 5).state).toBe('aborted');
		expect(
			checkIntegrity(record([{ variant: A, caseId: 'case:2', error: { failure: 'fatal' } }]), 3)
				.state
		).toBe('aborted');
	});

	test('counts calls answered outside the user’s own key', () => {
		const i = checkIntegrity(
			parseRecord(fixtureRecord([{ variant: A, caseId: 'case:0', samples: 'cc', byok: false }])),
			1
		);
		expect(i.sharedPool).toBe(2);
		expect(integrityLine(i)).toMatch(
			/1 of 1 rows scored · 0 technical errors · 2 calls answered by the shared pool/
		);
	});
});

describe('stats', () => {
	test('wilson matches hand-computed intervals', () => {
		// 7 of 190 → 1.8% to 7.4%; 0 of 65 → 0% to 5.6%
		const [lo, hi] = wilson(7, 190);
		expect(lo * 100).toBeCloseTo(1.8, 1);
		expect(hi * 100).toBeCloseTo(7.4, 1);
		expect(wilson(0, 65)[1] * 100).toBeCloseTo(5.6, 1);
		expect(wilson(0, 0)).toEqual([0, 1]);
	});

	test('a paired difference that never changes sign is clear of zero', () => {
		const d = pairedDifference([...Array<number>(20).fill(1 / 3), ...Array<number>(170).fill(0)]);
		expect(d.mean).toBeCloseTo(0.0351, 3);
		expect(d.low).toBeGreaterThan(0);
		expect(verdictOf(d)).toMatch(/real difference/);
	});

	test('a few cases each way cannot be told from zero', () => {
		const d = pairedDifference([1 / 3, -1 / 3, 1 / 3, ...Array<number>(60).fill(0)]);
		expect(d.low).toBeLessThan(0);
		expect(d.high).toBeGreaterThan(0);
		expect(verdictOf(d)).toMatch(/too small/);
	});

	test('the same record always gives the same interval', () => {
		const sample = [1, 0, 0, -1 / 3, 1 / 3, 0, 0];
		expect(pairedDifference(sample)).toEqual(pairedDifference(sample));
	});

	test('median of an odd and an empty list', () => {
		expect(median([3, 1, 2])).toBe(2);
		expect(median([])).toBe(0);
	});
});

describe('buildReport', () => {
	const record = parseRecord(
		fixtureRecord([
			...rows(A, 0, ['ccc', 'ccw', 'fff', 'ccc']),
			...rows(B, 1, ['ccc', 'ccc', 'ccc', 'cwc']),
			{ variant: B, column: 1, caseId: 'case:4', error: { failure: 'transient' } },
			{ variant: A, column: 0, caseId: 'case:4', samples: 'ccc' },
		])
	);
	const report = buildReport(record, checkIntegrity(record, 10));

	test('summarises each variant per case, leaving technical errors out', () => {
		const [a, b] = report.variants;
		expect(a).toMatchObject({ variant: A, cases: 5, allAgree: 4 });
		expect(a?.everWrong.cases).toBe(1);
		expect(a?.correct).toBeCloseTo((1 + 2 / 3 + 0 + 1 + 1) / 5);
		expect(a?.fellBack).toBeCloseTo(1 / 5);
		expect(b).toMatchObject({ variant: B, cases: 4 });
		expect(a?.medianMs).toBe(1000);
	});

	test('compares the others with the first column on the cases both scored', () => {
		const [c] = report.comparisons;
		expect(c).toMatchObject({ base: A, other: B, cases: 4, otherBetter: 2, baseBetter: 1 });
		expect(c?.differing.map(d => d.caseId)).toEqual(['case:3', 'case:1', 'case:2']);
		expect(c?.differing[0]).toMatchObject({
			want: 'RIGHT',
			base: { outcomes: ['correct', 'correct', 'correct'] },
		});
	});

	test('can compare against another column, and names the columns when it does not exist', () => {
		expect(buildReport(record, report.integrity, B).comparisons[0]).toMatchObject({
			base: B,
			other: A,
		});
		expect(() => buildReport(record, report.integrity, 'nope')).toThrow(/Columns: champion · a, b/);
	});

	test('renders the verdict in words', () => {
		const text = renderReport(report);
		expect(text).toContain('b  vs  champion · a');
		expect(text).toContain('They differ on 3 of 4 cases: b better on 2, worse on 1.');
		expect(text).toMatch(/case:1\s+RIGHT\s+✓✓✕ RIGHT \| other\s+✓✓✓ RIGHT/);
	});

	test('a run without a key lists what each variant answered', () => {
		const tried = parseRecord(
			fixtureRecord([
				{ variant: A, column: 0, caseId: 'try:x', samples: 'ccc', shown: ['a', 'a', 'a'] },
				{ variant: B, column: 1, caseId: 'try:x', samples: 'ccc', shown: ['a', 'b', 'a'] },
			])
		);
		expect(renderAnswers(tried)).toMatch(/try:x\s+a\s+a \| b \| a/);
	});
});
