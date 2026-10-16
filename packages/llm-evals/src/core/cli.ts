// The harness's commands. promptfoo stays the runner and the results screen; this layer picks
// the columns, guards the spending, and decides afterwards whether the run can be trusted.
//
//   doctor [task] [--model a,b]                    key, credit, one real call per model setting
//   try    <task> [selection] <the task's flags>   run variants on cases typed by hand
//   run    <task> [selection] --max-cost 0.50      run variants on the answer key
//                                                  (without --max-cost: what it can cost, and stop)
//   report [run]                                   the verdict on a finished run, again
//   key    <queue|dispute|draft|page|apply> <task> the answer key's review queue
//   view                                           promptfoo's viewer; leave it open
//
// selection: --variants strategy/model,... | all   or   --strategy a,b --model x,y
// run:       --split testset|heldout|all (default testset; heldout needs --reason "...")
//            --samples n  --first n  --group g  --note "what this run is for"
//
// A run that stopped, failed or was killed is finished by running the same command again: every
// answer already taken comes back from the cache at no cost.
import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

import { promptfooConfig, type EntryFiles } from './config.ts';
import { doctor, renderFindings, type Checks } from './doctor.ts';
import { estimate, renderEstimate } from './estimate.ts';
import { concurrencyFor, modelOf, selectVariants, type Selection, type Variant } from './grid.ts';
import { checkIntegrity, EXIT, integrityLine, type Integrity } from './integrity.ts';
import { keyCommand, logHeldoutRun } from './key.ts';
import { parseRecord, type RunRecord } from './record.ts';
import { buildReport, renderAnswers, renderReport } from './report.ts';
import { taskNamed, type Case, type Task, type TaskRegistry } from './task.ts';

export interface CliEnv {
	tasks: TaskRegistry;
	checks: Checks;
	/** the app's evals folder: `.runs/` and `.promptfoo/` live in it */
	root: string;
	entry: EntryFiles;
	/** the pinned promptfoo, e.g. `promptfoo@0.123.1`; it is run through npx, never imported */
	promptfoo: string;
}

/** A usage mistake: printed without a stack, exit code 1. */
export class UsageError extends Error {}

// --- arguments ----------------------------------------------------------------------------------

const VALUE_FLAGS = [
	'variants',
	'strategy',
	'model',
	'split',
	'samples',
	'first',
	'group',
	'max-cost',
	'note',
	'reason',
] as const;
type ValueFlag = (typeof VALUE_FLAGS)[number];

export interface Args {
	positional: string[];
	values: Partial<Record<ValueFlag, string>>;
	/** everything else, in order: a task's own flags for `try` */
	rest: string[];
}

export function parseArgs(argv: string[]): Args {
	const args: Args = { positional: [], values: {}, rest: [] };
	const queue = [...argv];
	let token: string | undefined;
	while ((token = queue.shift()) !== undefined) {
		const name = token.startsWith('--') ? token.slice(2) : undefined;
		if (name && (VALUE_FLAGS as readonly string[]).includes(name)) {
			const value = queue.shift();
			if (value === undefined) throw new UsageError(`--${name} needs a value.`);
			args.values[name as ValueFlag] = value;
		} else if (name !== undefined) {
			// An unknown flag and its value belong to the task (e.g. --word "hello" for `try`).
			args.rest.push(token);
			const next = queue[0];
			if (next !== undefined && !next.startsWith('--')) {
				args.rest.push(next);
				queue.shift();
			}
		} else args.positional.push(token);
	}
	return args;
}

function positiveNumber(value: string | undefined, flag: string): number | undefined {
	if (value === undefined) return undefined;
	const n = Number(value);
	if (!Number.isFinite(n) || n < 0)
		throw new UsageError(`--${flag} must be a number, got "${value}".`);
	return n;
}

// --- promptfoo ----------------------------------------------------------------------------------

function promptfooEnv(env: CliEnv, extra: Record<string, string>): NodeJS.ProcessEnv {
	return {
		...process.env,
		// Results database and response cache live in the app's evals folder (gitignored), not in ~/.promptfoo.
		PROMPTFOO_CONFIG_DIR: join(env.root, '.promptfoo'),
		// promptfoo drops cached answers after 14 days by default; a finished answer stays free for a year.
		PROMPTFOO_CACHE_TTL: String(365 * 24 * 60 * 60),
		PROMPTFOO_DISABLE_TELEMETRY: '1',
		PROMPTFOO_DISABLE_SHARING: '1',
		PROMPTFOO_DISABLE_UPDATE: '1',
		// Exit codes are ours (core/integrity.ts): promptfoo's is the same for an error and a wrong answer.
		PROMPTFOO_FAILED_TEST_EXIT_CODE: '0',
		...extra,
	};
}

function promptfoo(env: CliEnv, args: string[], extra: Record<string, string> = {}): number {
	const result = spawnSync('npx', ['-y', env.promptfoo, ...args], {
		stdio: 'inherit',
		cwd: env.root,
		env: promptfooEnv(env, extra),
	});
	return result.status ?? 1;
}

function promptfooOutput(env: CliEnv, args: string[]): string {
	const result = spawnSync('npx', ['-y', env.promptfoo, ...args], {
		cwd: env.root,
		env: promptfooEnv(env, {}),
		encoding: 'utf8',
	});
	return result.stdout;
}

// --- runs on disk -------------------------------------------------------------------------------

/** What `report` needs to know about a run, saved beside its record. */
export interface RunMeta {
	runId: string;
	task: string;
	evalId?: string;
	variants: { id: string; label: string }[];
	split: string;
	/** why a held-out run was made, for the ledger */
	reason?: string;
	cases: number;
	samples: number;
	maxCost: number;
	perCallMax: number;
	startedAt: string;
}

const runsDir = (env: CliEnv) => join(env.root, '.runs');
const runDir = (env: CliEnv, runId: string) => join(runsDir(env), runId);

function saveMeta(env: CliEnv, meta: RunMeta): void {
	writeFileSync(join(runDir(env, meta.runId), 'meta.json'), JSON.stringify(meta, null, '\t'));
}

function loadMeta(env: CliEnv, run: string | undefined): RunMeta {
	const all = existsSync(runsDir(env)) ? readdirSync(runsDir(env)).sort() : [];
	const runId = run && run !== 'latest' ? run : all.at(-1);
	const file = runId ? join(runDir(env, runId), 'meta.json') : undefined;
	if (!file || !existsSync(file)) {
		throw new UsageError(
			run ? `No run "${run}". Runs: ${all.slice(-5).join(', ') || 'none yet'}` : 'No run yet.'
		);
	}
	return JSON.parse(readFileSync(file, 'utf8')) as RunMeta;
}

/** The environment a run's provider processes need: the spending limit and what the health check tests. */
function runEnv(env: CliEnv, meta: RunMeta): Record<string, string> {
	const task = taskNamed(env.tasks, meta.task);
	const models = [...new Set(meta.variants.map(v => modelOf(v.id)))];
	return {
		EVALS_RUN_ID: meta.runId,
		EVALS_SPEND_FILE: join(runDir(env, meta.runId), 'spend.log'),
		EVALS_MAX_COST: String(meta.maxCost),
		EVALS_EST_COST_PER_CALL: String(meta.perCallMax),
		// At a limit of $0 nothing can be paid, so there is nothing for the health check to protect.
		EVALS_PREFLIGHT: meta.maxCost > 0 ? '1' : '0',
		EVALS_TASK: task.name,
		EVALS_MODELS: models.join(','),
	};
}

function spentBy(env: CliEnv, runId: string): number {
	const file = join(runDir(env, runId), 'spend.log');
	if (!existsSync(file)) return 0;
	return readFileSync(file, 'utf8')
		.split('\n')
		.reduce((sum, line) => sum + (Number(line) || 0), 0);
}

/** Exports the run from promptfoo to `.runs/<id>/record.json` and reads it; null when promptfoo saved nothing. */
function exportRecord(env: CliEnv, meta: RunMeta): RunRecord | null {
	const evalId =
		meta.evalId ??
		promptfooOutput(env, ['list', 'evals', '-n', '1', '--ids-only'])
			.split('\n')
			.map(line => line.trim())
			.find(line => line.startsWith('eval-'));
	if (!evalId) return null;
	const file = join(runDir(env, meta.runId), 'record.json');
	promptfooOutput(env, ['export', 'eval', evalId, '-o', file]);
	if (!existsSync(file)) return null;
	const record = parseRecord(JSON.parse(readFileSync(file, 'utf8')));
	// The latest eval in the database is ours only if it carries this run's tag.
	if (record.tags.run !== meta.runId) return null;
	if (!meta.evalId) {
		meta.evalId = evalId;
		saveMeta(env, meta);
	}
	return record;
}

const AGAIN = 'the same command again: the answers already taken are free.';
const ADVICE: Record<Integrity['state'], string> = {
	ok: '',
	over_budget: `Stopped by the spending limit. With a higher --max-cost, run ${AGAIN}`,
	aborted: `The run did not finish: the key, the credit or the model was refused, or it was interrupted. Run \`doctor\`, then ${AGAIN}`,
	untrusted: `Too many rows failed for technical reasons, so no result is reported. Run \`doctor\`, then ${AGAIN}`,
};

/** After promptfoo exits: export, check, report. Returns the exit code. */
function finish(env: CliEnv, meta: RunMeta): number {
	const record = exportRecord(env, meta);
	if (!record) {
		console.error(
			'\nThe run saved no results: the health check failed or promptfoo stopped before the first row. Nothing was scored.'
		);
		return EXIT.aborted;
	}
	const integrity = checkIntegrity(record, meta.cases * meta.variants.length);
	const task = taskNamed(env.tasks, meta.task);
	const spent = spentBy(env, meta.runId);
	if (integrity.state === 'ok' && meta.split !== 'testset' && meta.split !== 'try') {
		logHeldoutRun(task, {
			runId: meta.runId,
			variants: meta.variants.map(v => v.id),
			...(meta.reason ? { reason: meta.reason } : {}),
		});
	}
	const report = buildReport(record, integrity);
	writeFileSync(join(runDir(env, meta.runId), 'report.json'), JSON.stringify(report, null, '\t'));
	const out = ['', `Run ${meta.runId} · spent $${spent.toFixed(4)} of the $${meta.maxCost} limit`];
	if (integrity.state === 'untrusted') out.push(`Integrity: ${integrityLine(integrity)}`);
	else if (meta.split === 'try') {
		// Typed cases have no key behind them: show the answers and let the reader judge.
		out.push(
			report.description,
			`Integrity: ${integrityLine(integrity)}`,
			'',
			renderAnswers(record)
		);
	} else out.push(renderReport(report));
	if (ADVICE[integrity.state]) out.push('', ADVICE[integrity.state]);
	out.push('', 'Every answer, side by side: bun evals:view');
	process.stdout.write(`${out.join('\n')}\n`);
	return EXIT[integrity.state];
}

// --- commands -----------------------------------------------------------------------------------

function taskOf(env: CliEnv, args: Args): Task {
	const name = args.positional[0];
	if (!name) {
		const [only, ...others] = Object.values(env.tasks);
		if (only && others.length === 0) return only;
		throw new UsageError(
			only
				? `Name a task. Tasks: ${Object.keys(env.tasks).join(', ')}`
				: 'No task yet. Adding one is described in add-task.md of the llm-evals skill.'
		);
	}
	try {
		return taskNamed(env.tasks, name);
	} catch (error) {
		throw new UsageError((error as Error).message);
	}
}

function selectionOf(task: Task, args: Args): Variant[] {
	const selection: Selection = {
		...(args.values.variants ? { variants: args.values.variants } : {}),
		...(args.values.strategy ? { strategy: args.values.strategy } : {}),
		...(args.values.model ? { model: args.values.model } : {}),
	};
	try {
		return selectVariants(task, selection);
	} catch (error) {
		throw new UsageError((error as Error).message);
	}
}

const SPLITS = ['testset', 'heldout', 'all'];

function keyCases(task: Task, args: Args): { cases: Case[]; split: string } {
	const split = args.values.split ?? 'testset';
	if (!SPLITS.includes(split)) throw new UsageError(`--split is one of ${SPLITS.join(', ')}.`);
	if (split !== 'testset' && !args.values.reason) {
		throw new UsageError(
			'The held-out cases are for decisions, and every look at them wears them out. Say why: --reason "choosing between X and Y"'
		);
	}
	let cases = task.cases().filter(c => split === 'all' || c.split === split);
	if (args.values.group) cases = cases.filter(c => c.group === args.values.group);
	const first = positiveNumber(args.values.first, 'first');
	if (first !== undefined) cases = cases.slice(0, first);
	if (cases.length === 0) throw new UsageError('No case matches that selection.');
	return { cases, split };
}

function gitCommit(env: CliEnv): string {
	const git = (args: string[]) =>
		spawnSync('git', args, { cwd: env.root, encoding: 'utf8' }).stdout.trim();
	const commit = git(['rev-parse', '--short', 'HEAD']);
	if (!commit) return 'unknown';
	return git(['status', '--porcelain', '--', '.']) ? `${commit}+changes` : commit;
}

function stamp(now: Date): string {
	const p = (n: number) => String(n).padStart(2, '0');
	return `${now.getFullYear()}${p(now.getMonth() + 1)}${p(now.getDate())}-${p(now.getHours())}${p(now.getMinutes())}${p(now.getSeconds())}`;
}

interface Launch {
	task: Task;
	variants: Variant[];
	cases: Case[];
	split: string;
	maxCost: number;
	args: Args;
}

function launch(env: CliEnv, { task, variants, cases, split, maxCost, args }: Launch): number {
	const samples = positiveNumber(args.values.samples, 'samples') ?? task.samples;
	if (!Number.isInteger(samples) || samples < 1)
		throw new UsageError('--samples is a whole number, 1 or more.');
	const concurrency = concurrencyFor(task, variants);
	const cost = estimate(task, variants, cases.length, samples, concurrency);
	const runId = `${stamp(new Date())}-${task.name}`;
	const dir = runDir(env, runId);
	mkdirSync(dir, { recursive: true });

	const note = args.values.note ?? `${variants.map(v => v.label).join(' vs ')}`;
	const config = promptfooConfig(
		{
			runId,
			task,
			variants,
			cases,
			samples,
			concurrency,
			description: `${task.title} · ${split} · ${note}`,
			tags: {
				run: runId,
				key: task.keyVersion(),
				commit: gitCommit(env),
				split,
				...(args.values.reason ? { reason: args.values.reason } : {}),
			},
		},
		env.entry
	);
	const configFile = join(dir, 'config.json');
	writeFileSync(configFile, JSON.stringify(config, null, '\t'));

	const meta: RunMeta = {
		runId,
		task: task.name,
		variants: variants.map(v => ({ id: v.id, label: v.label })),
		split,
		...(args.values.reason ? { reason: args.values.reason } : {}),
		cases: cases.length,
		samples,
		maxCost,
		perCallMax: cost.perCallMax,
		startedAt: new Date().toISOString(),
	};
	saveMeta(env, meta);

	process.stdout.write(
		[
			`Run ${runId}`,
			`${cases.length} cases × ${variants.length} variant${variants.length === 1 ? '' : 's'} × ${samples} sample${samples === 1 ? '' : 's'}: ${renderEstimate(cost)}.`,
			`${limitLine(maxCost, cost.atMost)} ${concurrency} call${concurrency === 1 ? '' : 's'} at a time.`,
			'',
		].join('\n')
	);
	promptfoo(
		env,
		['eval', '-c', configFile, '--no-table', '-j', String(concurrency)],
		runEnv(env, meta)
	);
	return finish(env, meta);
}

/** What the limit means for this run, said before it starts. */
function limitLine(maxCost: number, atMost: number): string {
	if (maxCost === 0)
		return 'Spending limit $0: only answers already in the cache. The run stops at the first one it would have to pay for.';
	if (atMost > maxCost)
		return `Spending limit $${maxCost}, below the estimate: if the run stops at the limit, running it again with a higher one finishes it.`;
	return `Spending limit $${maxCost}.`;
}

const TRY_LIMIT = 0.05;

export async function main(argv: string[], env: CliEnv): Promise<number> {
	const [command, ...rest] = argv;
	try {
		const args = parseArgs(rest);
		switch (command) {
			case 'view':
				return promptfoo(env, ['view', '--yes']);

			case 'doctor': {
				const task = taskOf(env, args);
				const models = args.values.model?.split(',') ?? [modelOf(task.champion)];
				const findings = await doctor(task, models, env.checks);
				process.stdout.write(`${renderFindings(findings)}\n`);
				return findings.every(f => f.ok) ? EXIT.ok : EXIT.aborted;
			}

			case 'run': {
				const task = taskOf(env, args);
				if (args.rest.length) throw new UsageError(`Unknown option ${args.rest[0]}.`);
				const variants = selectionOf(task, args);
				const picked = keyCases(task, args);
				const maxCost = positiveNumber(args.values['max-cost'], 'max-cost');
				if (maxCost === undefined) {
					// No limit, no run: say what it can cost so the user can name one.
					const samples = positiveNumber(args.values.samples, 'samples') ?? task.samples;
					const cost = estimate(
						task,
						variants,
						picked.cases.length,
						samples,
						concurrencyFor(task, variants)
					);
					throw new UsageError(
						`${variants.map(v => v.label).join('\n')}\n${picked.cases.length} cases × ${samples} samples: ${renderEstimate(cost)}.\nNothing was run. A run needs a spending limit: --max-cost 0.50 (dollars). Answers already in the cache cost nothing.`
					);
				}
				return launch(env, { task, variants, maxCost, args, ...picked });
			}

			case 'try': {
				const task = taskOf(env, args);
				let cases: Case[];
				try {
					cases = task.tryCases(args.rest);
				} catch (error) {
					throw new UsageError((error as Error).message);
				}
				const variants = selectionOf(task, args);
				const maxCost = positiveNumber(args.values['max-cost'], 'max-cost') ?? TRY_LIMIT;
				return launch(env, { task, variants, cases, split: 'try', maxCost, args });
			}

			case 'report': {
				const meta = loadMeta(env, args.positional[0]);
				const record = exportRecord(env, meta);
				if (!record) throw new UsageError(`Run ${meta.runId} saved no results.`);
				const integrity = checkIntegrity(record, meta.cases * meta.variants.length);
				process.stdout.write(
					`Run ${meta.runId}\n${renderReport(buildReport(record, integrity))}\n`
				);
				return EXIT[integrity.state];
			}

			case 'key':
				return keyCommand(env.tasks, rest, env.root);

			default:
				throw new UsageError(
					'Usage: <doctor|try|run|report|key|view> [task] [options]. See the llm-evals skill.'
				);
		}
	} catch (error) {
		if (error instanceof UsageError) {
			console.error(error.message);
			return EXIT.usage;
		}
		throw error;
	}
}
