// The contract between the harness and a task. Core knows only these shapes: a task brings its
// answer key, its strategies and its model settings; core runs, scores and reports them.
import type { KeyReview } from './key.ts';
import type { CallApiContext } from './promptfoo.ts';

/**
 * How one answer compares with the key:
 * - correct
 * - fell_back: the safe answer where the key wanted more (nothing wrong is shown to a user)
 * - wrong: would show something incorrect
 * - invalid: the model's answer was unusable, so the strategy fell back
 */
export type Outcome = 'correct' | 'fell_back' | 'wrong' | 'invalid';

export const OUTCOMES: readonly Outcome[] = ['correct', 'fell_back', 'wrong', 'invalid'];

export interface Verdict {
	outcome: Outcome;
	/** the task's own step scores, each reported only on answers where it can happen */
	metrics: Record<string, 0 | 1>;
	want: string;
	got: string;
}

/** One model call, kept in the row's metadata so each step can be judged on its own in the viewer. */
export interface Step {
	name: string;
	model: string;
	cached: boolean;
	cost: number;
	ms: number;
	promptTokens: number;
	completionTokens: number;
	reasoningTokens: number;
	/** the model's whole answer as returned */
	answer: unknown;
	/** why the answer was unusable; the strategy then falls back */
	invalid?: string;
	/** false when the call was answered by a shared pool instead of the user's own provider key */
	byok?: boolean;
}

export interface StrategyRun<D> {
	decision: D;
	steps: Step[];
}

/** A model and how hard it thinks, with what the harness needs to pace and price it. */
export interface ModelSetting {
	/** the provider's model id, pinned (never a `latest` alias) */
	model: string;
	effort?: string;
	/** true when calls are expected to run on the user's own provider key */
	expectByok?: boolean;
	/** dollars for one call, for estimates before any run of this setting exists */
	estCostPerCall: number;
	/** the account's limits for this model; they set how many calls run at once */
	requestsPerMinute: number;
	tokensPerMinute: number;
	tokensPerCall: number;
	secondsPerCall: number;
}

/** A way to solve the task: a flow plus a prompt version. A challenger is a new entry beside the champion's. */
export interface Strategy<I, D> {
	/** model calls one answer takes at most, for estimates */
	calls: number;
	run(input: I, model: ModelSetting, context?: CallApiContext): Promise<StrategyRun<D>>;
}

/** One item to answer. `vars` is what the row shows and what `Task.input` reads back. */
export interface Case {
	id: string;
	description: string;
	vars: Record<string, string>;
	/** `testset` to iterate on, `heldout` for decisions, `try` for names typed by hand */
	split: string;
	group?: string;
}

export interface Task<I = unknown, D = unknown> {
	name: string;
	title: string;
	/** the task's folder; its answer key, queue and ledger live in `key/` under it */
	dir: string;
	keyVersion(): string;
	/** how the key is reviewed and changed; without it the `key` commands refuse the task */
	review?: KeyReview;
	/** every case in the answer key */
	cases(): Case[];
	/** cases for `try`, from the flags after the task name; throws with a usage line when they are wrong */
	tryCases(args: string[]): Case[];
	/** the strategy input for a row, or null when the row's vars are unusable */
	input(vars: Record<string, unknown>): I | null;
	strategies: Record<string, Strategy<I, D>>;
	models: Record<string, ModelSetting>;
	/** the current best variant, `strategy/model` */
	champion: string;
	/** answers taken per case; one row shows them all */
	samples: number;
	/** vars of one easy case outside the key, for the health check */
	canary: Record<string, string>;
	/** what the app would show for this decision */
	show(decision: D, input: I): string;
	/** the verdict against the key, or undefined for a case that is not in it */
	judge(caseId: string, decision: D): Verdict | undefined;
}

/**
 * Registers a task. Core passes a task's inputs and decisions back to that same task and never
 * looks inside them, so the registry holds tasks with their own types erased.
 */
export function defineTask<I, D>(task: Task<I, D>): Task {
	return task;
}

export type TaskRegistry = Record<string, Task>;

export function taskNamed(tasks: TaskRegistry, name: unknown): Task {
	const task = typeof name === 'string' ? tasks[name] : undefined;
	if (!task)
		throw new Error(`Unknown task "${String(name)}". Tasks: ${Object.keys(tasks).join(', ')}`);
	return task;
}
