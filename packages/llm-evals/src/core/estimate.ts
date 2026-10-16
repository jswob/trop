// What a run can cost at most, before it starts. "At most" because answers already in the cache
// cost nothing, and the CLI cannot see promptfoo's cache.
import type { Variant } from './grid.ts';
import type { Task } from './task.ts';

export interface Estimate {
	/** model calls if nothing is cached */
	calls: number;
	atMost: number;
	seconds: number;
	/** the dearest single call in the run; the spending limit keeps this much in reserve per call in flight */
	perCallMax: number;
}

export function estimate(
	task: Task,
	variants: Variant[],
	cases: number,
	samples: number,
	concurrency: number
): Estimate {
	let calls = 0;
	let atMost = 0;
	let seconds = 0;
	let perCallMax = 0;
	for (const v of variants) {
		const strategy = task.strategies[v.strategy];
		const model = task.models[v.model];
		if (!strategy || !model) continue;
		const n = cases * samples * strategy.calls;
		calls += n;
		atMost += n * model.estCostPerCall;
		seconds += n * model.secondsPerCall;
		perCallMax = Math.max(perCallMax, model.estCostPerCall);
	}
	return { calls, atMost, seconds: seconds / Math.max(1, concurrency), perCallMax };
}

export function renderEstimate(e: Estimate): string {
	const time = e.seconds < 90 ? `${Math.ceil(e.seconds)} s` : `${Math.ceil(e.seconds / 60)} min`;
	const dollars = e.atMost < 0.1 ? e.atMost.toFixed(3) : e.atMost.toFixed(2);
	return `${e.calls} model calls at most, about $${dollars} and ${time} if nothing is cached`;
}
