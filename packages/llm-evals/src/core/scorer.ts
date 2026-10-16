// The promptfoo assertion behind every row: grades each sample against the task's answer key, in
// code, and reports scores per case. Deterministic; no LLM grading.
import type { AssertionContext, GradingResult } from './promptfoo.ts';
import type { RowMetadata } from './provider.ts';
import { OUTCOMES, taskNamed, type Outcome, type TaskRegistry, type Verdict } from './task.ts';

const SCORE: Record<Outcome, number> = { correct: 1, fell_back: 0.5, invalid: 0, wrong: 0 };

const mean = (values: number[]) => values.reduce((a, b) => a + b, 0) / values.length;

/**
 * Scores for one case from the verdicts of its samples:
 * - `correct`, `fell_back`, `wrong`, `invalid`: the share of samples with that outcome
 * - `ever_wrong`: 1 when any sample was wrong. A case passes only when this is 0
 * - `all_agree`: 1 when every sample gave the same answer
 * - the task's own step scores, averaged over the samples that report them
 */
export function scoreSamples(verdicts: Verdict[], shown: string[]): GradingResult {
	const share = Object.fromEntries(
		OUTCOMES.map(o => [o, mean(verdicts.map(v => (v.outcome === o ? 1 : 0)))])
	);
	const metrics: Record<string, number> = {};
	for (const name of new Set(verdicts.flatMap(v => Object.keys(v.metrics)))) {
		metrics[name] = mean(verdicts.flatMap(v => (name in v.metrics ? [v.metrics[name] ?? 0] : [])));
	}
	const everWrong = verdicts.some(v => v.outcome === 'wrong');
	return {
		pass: !everWrong,
		score: mean(verdicts.map(v => SCORE[v.outcome])),
		reason: verdicts
			.map((v, i) => `${i + 1}. ${v.outcome}: got ${v.got}, key says ${v.want}`)
			.join('\n'),
		namedScores: {
			...share,
			ever_wrong: everWrong ? 1 : 0,
			all_agree: new Set(shown).size === 1 ? 1 : 0,
			...metrics,
		},
	};
}

/** The outcome of each sample, read back from a row's `reason` (written by scoreSamples above). */
export function outcomesIn(reason: string): Outcome[] {
	return reason
		.split('\n')
		.map(line => /^\d+\. (\w+):/.exec(line)?.[1])
		.filter((o): o is Outcome => OUTCOMES.includes(o as Outcome));
}

export function createScorer(tasks: TaskRegistry) {
	return function scorer(_output: string, context: AssertionContext): GradingResult {
		const row = context.metadata as Partial<RowMetadata> | undefined;
		if (!row?.samples?.length) {
			return { pass: false, score: 0, reason: 'The provider returned no samples.' };
		}
		const task = taskNamed(tasks, row.task);
		const id = context.vars.id;
		const verdicts = row.samples.map(s =>
			typeof id === 'string' ? task.judge(id, s.decision) : undefined
		);
		if (verdicts.some(v => v === undefined)) {
			return { pass: true, score: 1, reason: 'Not in the answer key: judge it yourself.' };
		}
		return scoreSamples(
			verdicts as Verdict[],
			row.samples.map(s => s.shown)
		);
	};
}
