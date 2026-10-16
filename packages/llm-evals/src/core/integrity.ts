// The gate after a run: can its numbers be trusted? Counted from the rows of the run record.
// promptfoo's own summary is not used: it lists an aborted run as 100% passed, puts technical
// errors in the pass rate, and was off by one after a resumed run.
import type { RunRecord } from './record.ts';

export type RunState =
	/** every expected row was scored, or close enough */
	| 'ok'
	/** the spending limit stopped it; continue with a higher limit */
	| 'over_budget'
	/** the key, the credit or the model is wrong, or the run was killed; rows are missing */
	| 'aborted'
	/** too many rows failed for technical reasons, even after a retry */
	| 'untrusted';

export interface Integrity {
	expected: number;
	scored: number;
	/** rows promptfoo never saved (run killed or aborted) */
	missing: number;
	/** rate limits, overloads, broken replies: worth one more try */
	retryable: number;
	notRun: number;
	fatal: number;
	/** the request itself was refused, or our own code threw: a retry cannot help */
	rejected: number;
	/** model calls answered by a shared pool though the model setting expects the user's own key */
	sharedPool: number;
	state: RunState;
}

/** Above this share of unscored rows, a run is not reported. */
export const MAX_ERROR_SHARE = 0.01;

/** Exit codes of `run` and `continue`. Wrong answers never change them: they are the result. */
export const EXIT: Record<RunState | 'usage', number> = {
	ok: 0,
	usage: 1,
	over_budget: 2,
	untrusted: 3,
	aborted: 4,
};

export function checkIntegrity(record: RunRecord, expected: number): Integrity {
	const count = (test: (row: RunRecord['rows'][number]) => boolean) =>
		record.rows.filter(test).length;
	const scored = count(r => r.state !== 'error');
	const retryable = count(
		r => r.failure === 'rate_limit' || r.failure === 'transient' || r.failure === 'unknown'
	);
	const notRun = count(r => r.failure === 'not_run');
	const fatal = count(r => r.failure === 'fatal');
	const rejected = count(r => r.failure === 'rejected');
	const missing = Math.max(0, expected - record.rows.length);
	const sharedPool = record.rows
		.flatMap(r => r.samples.flatMap(s => s.steps))
		.filter(step => step.byok === false).length;

	let state: RunState = 'ok';
	if (fatal > 0 || missing > 0) state = 'aborted';
	else if (notRun > 0) state = 'over_budget';
	else if ((retryable + rejected) / Math.max(1, expected) > MAX_ERROR_SHARE) state = 'untrusted';
	return { expected, scored, missing, retryable, notRun, fatal, rejected, sharedPool, state };
}

export function integrityLine(i: Integrity): string {
	const parts = [`${i.scored} of ${i.expected} rows scored`];
	const unscored = i.retryable + i.rejected;
	parts.push(`${unscored} technical error${unscored === 1 ? '' : 's'}`);
	if (i.notRun) parts.push(`${i.notRun} not run (spending limit)`);
	if (i.fatal) parts.push(`${i.fatal} fatal`);
	if (i.missing) parts.push(`${i.missing} missing`);
	if (i.sharedPool)
		parts.push(`${i.sharedPool} calls answered by the shared pool, not your own key`);
	return parts.join(' · ');
}
