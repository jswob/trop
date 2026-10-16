// The spending limit of a run. Every paid call is checked against it first and recorded after.
// The total lives in a file: the CLI is blocked while promptfoo runs, and a retry of failed rows is
// a second process that must continue from what the first one spent.
import { appendFileSync, existsSync, readFileSync } from 'node:fs';

import type { ModelCallFailure } from './outcome.ts';

/** Thrown instead of calling the model once the limit is reached. The row is reported as not run. */
export class NotRunError extends Error implements ModelCallFailure {
	readonly kind = 'not_run';
	readonly status = 0;
	readonly retryAfterSeconds = null;
}

interface Budget {
	file: string;
	max: number;
	/** what a call still in flight may cost; reserved so the limit trips before it is crossed */
	perCall: number;
	spent: number;
	inFlight: number;
}

// undefined: not looked up yet. null: this process has no limit (unit tests, `view`).
let budget: Budget | null | undefined;

function load(): Budget | null {
	const file = process.env.EVALS_SPEND_FILE;
	const max = Number(process.env.EVALS_MAX_COST);
	if (!file || !Number.isFinite(max)) return null;
	const spent = existsSync(file)
		? readFileSync(file, 'utf8')
				.split('\n')
				.reduce((sum, line) => sum + (Number(line) || 0), 0)
		: 0;
	return {
		file,
		max,
		perCall: Number(process.env.EVALS_EST_COST_PER_CALL) || 0,
		spent,
		inFlight: 0,
	};
}

/** Call before a paid model call. Throws NotRunError when the limit leaves no room for it. */
export function reserve(): void {
	if (budget === undefined) budget = load();
	if (!budget) return;
	if (budget.spent + (budget.inFlight + 1) * budget.perCall > budget.max) {
		throw new NotRunError(
			`not run: the spending limit of $${budget.max} was reached ($${budget.spent.toFixed(4)} spent)`
		);
	}
	budget.inFlight++;
}

/** Call when a reserved call ends, with what it cost (0 when it failed). */
export function settle(cost: number): void {
	if (!budget) return;
	budget.inFlight = Math.max(0, budget.inFlight - 1);
	if (cost > 0) {
		budget.spent += cost;
		appendFileSync(budget.file, `${cost}\n`);
	}
}

/** Test hook: forget the loaded limit so the next call reads the environment again. */
export function resetBudget(): void {
	budget = undefined;
}
