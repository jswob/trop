// The health check before money is spent: does the key work, is there credit, does each model
// setting answer one real question properly and through the expected route. It runs as `doctor`
// and as the first step of every paid run, where a failed check means zero paid calls.
import { isModelCallFailure } from './outcome.ts';
import type { Task } from './task.ts';

/** What the model adapter knows about the key in use. Core never calls an adapter itself. */
export interface KeyStatus {
	label: string;
	/** dollars left on the key; null when the key has no limit of its own */
	limitRemaining: number | null;
	isFreeTier: boolean;
}

/** What went wrong with a check, for `Checks.advice`. */
export type Trouble = 'fatal' | 'rate_limit' | 'transient' | 'rejected' | 'shared_pool';

export interface Checks {
	keyStatus?: () => Promise<KeyStatus>;
	/** what the user can do about each kind of trouble, in the adapter's own terms */
	advice?: Partial<Record<Trouble, string>>;
}

export interface Finding {
	ok: boolean;
	what: string;
	detail: string;
	/** what the user can do about a failed check */
	fix?: string;
}

const FIX: Record<Trouble, string> = {
	fatal: 'Check the API key, the credit on it, and the model id.',
	rate_limit:
		'The account is rate limited before the run even starts. Wait a minute and try again.',
	transient: 'The provider is overloaded or unreachable. Try again shortly.',
	rejected: 'The request itself was refused: the model may not support the strict answer format.',
	shared_pool: 'Set the provider key up so that calls cannot fall back to a shared pool.',
};

const fixFor = (checks: Checks, trouble: Trouble) => checks.advice?.[trouble] ?? FIX[trouble];

function failed(what: string, error: unknown, checks: Checks): Finding {
	const detail = error instanceof Error ? error.message : String(error);
	const kind = isModelCallFailure(error) ? error.kind : undefined;
	return {
		ok: false,
		what,
		detail,
		...(kind && kind !== 'not_run' ? { fix: fixFor(checks, kind) } : {}),
	};
}

async function checkKey(checks: Checks, needed: number): Promise<Finding[]> {
	if (!checks.keyStatus) return [];
	try {
		const key = await checks.keyStatus();
		const left = key.limitRemaining;
		if (left !== null && left < needed) {
			return [
				{
					ok: false,
					what: 'credit',
					detail: `$${left.toFixed(2)} left on the key, below this run's limit of $${needed}`,
					fix: 'Add credit to the key or lower --max-cost.',
				},
			];
		}
		return [
			{
				ok: true,
				what: 'key',
				detail: `works${left === null ? '' : `, $${left.toFixed(2)} credit left`}${key.isFreeTier ? ' (free tier: strict limits)' : ''}`,
			},
		];
	} catch (error) {
		return [failed('key', error, checks)];
	}
}

/** One real, uncached call on the task's canary case through the champion's strategy. */
async function checkModel(task: Task, modelName: string, checks: Checks): Promise<Finding> {
	const what = `model ${modelName}`;
	const model = task.models[modelName];
	const strategy = task.strategies[task.champion.split('/')[0] ?? ''];
	const input = task.input(task.canary);
	if (!model || !strategy || input === null) {
		return {
			ok: false,
			what,
			detail: 'the task has no such model setting, champion strategy or canary',
		};
	}
	try {
		const run = await strategy.run(input, model);
		const invalid = run.steps.find(s => s.invalid);
		if (invalid) {
			return {
				ok: false,
				what,
				detail: `answered, but not in the required format: ${invalid.invalid}`,
				fix: fixFor(checks, 'rejected'),
			};
		}
		if (model.expectByok && run.steps.some(s => s.byok === false)) {
			return {
				ok: false,
				what,
				detail: 'answered by a shared pool, not your own provider key',
				fix: fixFor(checks, 'shared_pool'),
			};
		}
		const cost = run.steps.reduce((sum, s) => sum + s.cost, 0);
		return {
			ok: true,
			what,
			detail: `answered "${task.show(run.decision, input)}" for $${cost.toFixed(5)}${cost === 0 ? ' (no cost reported)' : ''}`,
		};
	} catch (error) {
		return failed(what, error, checks);
	}
}

/** `needed`: the dollars the coming run may spend, checked against the key's credit. */
export async function doctor(
	task: Task,
	modelNames: string[],
	checks: Checks,
	needed = 0
): Promise<Finding[]> {
	const key = await checkKey(checks, needed);
	// A dead key fails every model the same way: report it once.
	if (key.some(f => !f.ok && f.what === 'key')) return key;
	const models: Finding[] = [];
	for (const name of modelNames) models.push(await checkModel(task, name, checks));
	return [...key, ...models];
}

export function renderFindings(findings: Finding[]): string {
	return findings
		.map(f => `${f.ok ? '✓' : '✕'} ${f.what}: ${f.detail}${f.fix ? `\n  → ${f.fix}` : ''}`)
		.join('\n');
}
