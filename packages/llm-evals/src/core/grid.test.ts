import { describe, expect, test } from 'vitest';

import { promptfooConfig, wilsonUpperPct } from './config.ts';
import { fakeTask } from './fake-task.ts';
import { allVariants, concurrencyFor, selectVariants } from './grid.ts';

const task = fakeTask();
const ids = (selection: Parameters<typeof selectVariants>[1]) =>
	selectVariants(task, selection).map(v => v.id);

describe('selectVariants', () => {
	test('with nothing chosen, runs the champion alone', () => {
		expect(ids({})).toEqual(['upper/fast']);
		expect(selectVariants(task, {})[0]?.label).toBe('champion · upper · fast');
	});

	test('takes named cells, the champion first', () => {
		expect(ids({ variants: 'lower/slow, upper/fast' })).toEqual(['upper/fast', 'lower/slow']);
	});

	test('crosses strategies with model settings, filling a missing side from the champion', () => {
		expect(ids({ model: 'fast,slow' })).toEqual(['upper/fast', 'upper/slow']);
		expect(ids({ strategy: 'lower' })).toEqual(['lower/fast']);
		expect(ids({ strategy: 'upper,lower', model: 'slow' })).toEqual(['upper/slow', 'lower/slow']);
	});

	test('`all` is the whole grid', () => {
		expect(ids({ variants: 'all' })).toHaveLength(allVariants(task).length);
	});

	test('names what exists when a cell does not', () => {
		expect(() => ids({ variants: 'upper/huge' })).toThrow(/Settings: fast, slow/);
		expect(() => ids({ variants: 'upper' })).toThrow(/strategy\/model/);
	});
});

describe('concurrencyFor', () => {
	test('follows the tightest model setting in the run', () => {
		// fast: min(600, 60000/100) = 600 calls a minute → 0.75 × 600 × 1 s / 60 = 7
		expect(concurrencyFor(task, selectVariants(task, { model: 'fast' }))).toBe(7);
		// slow: 6000/100 = 60 calls a minute → 0.75, floored, but never below 1
		expect(concurrencyFor(task, selectVariants(task, { model: 'fast,slow' }))).toBe(1);
	});
});

describe('promptfooConfig', () => {
	const variants = selectVariants(task, { model: 'fast,slow' });
	const config = promptfooConfig(
		{
			runId: 'r1',
			task,
			variants,
			cases: task.cases(),
			samples: 3,
			concurrency: 2,
			description: 'fake · r1',
			tags: { run: 'r1', key: '1' },
		},
		{ provider: '/abs/provider.ts', scorer: '/abs/scorer.ts', hooks: '/abs/hooks.ts' }
	);

	test('has one column per variant and one row per case', () => {
		expect(config.providers).toEqual([
			{
				id: 'file:///abs/provider.ts',
				label: 'champion · upper · fast',
				config: { task: 'fake', strategy: 'upper', model: 'fast', samples: 3 },
			},
			{
				id: 'file:///abs/provider.ts',
				label: 'upper · slow',
				config: { task: 'fake', strategy: 'upper', model: 'slow', samples: 3 },
			},
		]);
		expect(config.tests).toHaveLength(2);
		expect(config).toMatchObject({
			tags: { run: 'r1', key: '1' },
			evaluateOptions: { maxConcurrency: 2 },
			extensions: ['file:///abs/hooks.ts:hook'],
		});
		expect(config).not.toHaveProperty('evaluateOptions.repeat');
	});
});

describe('wilsonUpperPct', () => {
	test('is the 95% Wilson upper bound, written for mathjs', () => {
		// 7 of 60 → upper bound 22.18% (checked in the viewer on 2026-10-04)
		const formula = wilsonUpperPct('ever_wrong')
			.replaceAll('ever_wrong', '7')
			.replaceAll('__count', '60')
			.replaceAll('^', '**')
			.replaceAll('sqrt', 'Math.sqrt');
		// eslint-disable-next-line @typescript-eslint/no-implied-eval, @typescript-eslint/no-unsafe-call -- Reason: evaluates our own arithmetic string the way mathjs would
		expect(new Function(`return ${formula}`)()).toBeCloseTo(22.18, 1);
	});
});
