import { describe, expect, test } from 'vitest';

import { fakeTasks } from './fake-task.ts';
import { createScorer, scoreSamples } from './scorer.ts';
import type { Verdict } from './task.ts';

const verdict = (outcome: Verdict['outcome'], metrics: Verdict['metrics'] = {}): Verdict => ({
	outcome,
	metrics,
	want: 'A',
	got: outcome === 'correct' ? 'A' : 'b',
});

describe('scoreSamples', () => {
	test('a case with every sample correct passes with full scores', () => {
		const result = scoreSamples([verdict('correct'), verdict('correct')], ['A', 'A']);
		expect(result).toMatchObject({ pass: true, score: 1 });
		expect(result.namedScores).toMatchObject({ correct: 1, wrong: 0, ever_wrong: 0, all_agree: 1 });
	});

	test('one wrong sample in three fails the case and shows as a share', () => {
		const result = scoreSamples(
			[verdict('correct'), verdict('wrong'), verdict('correct')],
			['A', 'b', 'A']
		);
		expect(result.pass).toBe(false);
		expect(result.namedScores?.correct).toBeCloseTo(2 / 3);
		expect(result.namedScores?.wrong).toBeCloseTo(1 / 3);
		expect(result.namedScores).toMatchObject({ ever_wrong: 1, all_agree: 0 });
	});

	test('falling back is safe: the case still passes', () => {
		const result = scoreSamples([verdict('fell_back'), verdict('correct')], ['↩', 'A']);
		expect(result).toMatchObject({ pass: true, score: 0.75 });
		expect(result.namedScores).toMatchObject({ fell_back: 0.5, ever_wrong: 0 });
	});

	test('a step score is averaged over the samples that report it', () => {
		const result = scoreSamples(
			[verdict('wrong', { leaked: 1 }), verdict('correct', { leaked: 0 }), verdict('fell_back')],
			['a', 'b', 'c']
		);
		expect(result.namedScores?.leaked).toBe(0.5);
	});

	test('the reason lists each sample', () => {
		expect(scoreSamples([verdict('correct'), verdict('wrong')], ['A', 'b']).reason).toBe(
			'1. correct: got A, key says A\n2. wrong: got b, key says A'
		);
	});
});

describe('scorer', () => {
	const scorer = createScorer(fakeTasks());
	const row = (texts: string[]) => ({
		task: 'fake',
		variant: 'upper/fast',
		samples: texts.map(text => ({ decision: { text }, shown: text, steps: [] })),
	});

	test('grades every sample against the key', () => {
		const result = scorer('', {
			vars: { id: 'word:alpha' },
			metadata: row(['ALPHA', 'alpha', 'ALPHA']),
		});
		expect(result.pass).toBe(false);
		expect(result.namedScores).toMatchObject({ ever_wrong: 1, typo: 1 / 3 });
	});

	test('passes a case that is not in the key', () => {
		expect(scorer('', { vars: { id: 'try:gamma' }, metadata: row(['GAMMA']) }).pass).toBe(true);
	});

	test('fails when the provider gave no samples', () => {
		expect(scorer('', { vars: { id: 'word:alpha' } }).pass).toBe(false);
	});
});
