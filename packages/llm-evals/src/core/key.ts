// The life of an answer key after its first review. Everything that needs the user's judgement
// waits in one queue per task; the queue is emptied through a review page; the picks are checked
// and applied by the task, which bumps the key's version. The held-out ledger records every run on
// the held-out cases, because each look at them makes the next result a little less honest.
import { appendFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';

import { taskNamed, type Task, type TaskRegistry } from './task.ts';

/**
 * Where a queue item came from:
 * - draft: a new case proposed for the key
 * - dispute: an answer in the key looks wrong, to a model or to the user
 * - rule: a labelling rule changed and this case is touched by it
 */
export type QueueSource = 'draft' | 'dispute' | 'rule';
const SOURCES: readonly QueueSource[] = ['draft', 'dispute', 'rule'];

export interface QueueItem {
	/** the case id, already in the key or new */
	id: string;
	source: QueueSource;
	/** the doubt, in a sentence the user can judge */
	note: string;
	addedAt: string;
	/** the option proposed; shown as a suggestion, never pre-selected on a doubtful item */
	proposed?: string;
	/** true when nothing is in doubt: these go in one "confirm all" block */
	agreed?: boolean;
	/** for a case not in the key yet: what the task needs to add it */
	draft?: Record<string, unknown>;
}

/** One answer the user can pick for a case, with how it would read in the app. */
export interface ReviewOption {
	value: string;
	label: string;
	preview?: string;
	/** the option carries text the user may edit (e.g. the accepted forms), sent as `value:text` */
	text?: string;
}

export interface ReviewItem {
	id: string;
	title: string;
	group?: string;
	/** the key's current answer as an option value, when the case is already in the key */
	current?: string;
	options: ReviewOption[];
}

/** What the user chose for one case: an option value (with its text, if any) and a free comment. */
export interface Pick {
	value: string;
	note?: string;
}

export interface KeyChange {
	/** one line per case whose answer changes, for the changelog; empty when the picks confirm the key */
	changes: string[];
	/** why the picks cannot be applied as they are; when any, nothing is written */
	problems: string[];
	/** writes the key with its version bumped; returns the new version */
	commit(): string;
}

/** What a task supplies so its key can be reviewed. */
export interface KeyReview {
	/** the labelling rules, shown at the top of the review page */
	guide(): string;
	item(queued: QueueItem): ReviewItem;
	apply(picks: Record<string, Pick>, queue: QueueItem[]): KeyChange;
}

const keyFile = (task: Task, name: string) => join(task.dir, 'key', name);

export function readQueue(task: Task): QueueItem[] {
	const file = keyFile(task, 'queue.json');
	if (!existsSync(file)) return [];
	return (JSON.parse(readFileSync(file, 'utf8')) as { items: QueueItem[] }).items;
}

function writeQueue(task: Task, items: QueueItem[]): void {
	const file = keyFile(task, 'queue.json');
	mkdirSync(dirname(file), { recursive: true });
	writeFileSync(file, `${JSON.stringify({ items }, null, '\t')}\n`);
}

/** Adds items, replacing any already queued for the same case. */
export function enqueue(task: Task, items: QueueItem[]): QueueItem[] {
	const ids = new Set(items.map(i => i.id));
	const queue = [...readQueue(task).filter(i => !ids.has(i.id)), ...items];
	writeQueue(task, queue);
	return queue;
}

export interface HeldoutRun {
	runId: string;
	variants: string[];
	reason?: string;
}

export function logHeldoutRun(task: Task, run: HeldoutRun): void {
	const file = keyFile(task, 'heldout-ledger.jsonl');
	mkdirSync(dirname(file), { recursive: true });
	appendFileSync(
		file,
		`${JSON.stringify({ at: new Date().toISOString(), key: task.keyVersion(), ...run })}\n`
	);
}

/** Runs on the held-out cases so far, for the current key version. */
export function heldoutRuns(task: Task): number {
	const file = keyFile(task, 'heldout-ledger.jsonl');
	if (!existsSync(file)) return 0;
	return readFileSync(file, 'utf8')
		.split('\n')
		.filter(Boolean)
		.filter(line => (JSON.parse(line) as { key?: string }).key === task.keyVersion()).length;
}

/** The data the review page is drawn from. Doubtful items come first; agreed ones are grouped. */
export function reviewData(task: Task, review: KeyReview) {
	const queue = readQueue(task);
	const entry = (q: QueueItem) => ({
		...review.item(q),
		source: q.source,
		note: q.note,
		proposed: q.proposed,
	});
	return {
		task: task.name,
		title: task.title,
		keyVersion: task.keyVersion(),
		guide: review.guide(),
		doubtful: queue.filter(q => !q.agreed).map(entry),
		agreed: queue.filter(q => q.agreed).map(entry),
	};
}

const TEMPLATE = join(import.meta.dirname, 'review', 'template.html');

export function buildReviewPage(task: Task, review: KeyReview): string {
	const data = JSON.stringify(reviewData(task, review)).replaceAll('<', '\\u003c');
	return readFileSync(TEMPLATE, 'utf8').replace('/*__DATA__*/null', data);
}

/** Applies the user's picks. Nothing is written when the task finds a problem with them. */
export function applyPicks(
	task: Task,
	review: KeyReview,
	picks: Record<string, Pick>
): KeyChange & { version?: string } {
	const queue = readQueue(task);
	const unknown = Object.keys(picks).filter(id => !queue.some(q => q.id === id));
	const change = review.apply(picks, queue);
	const problems = [...unknown.map(id => `${id}: not in the queue`), ...change.problems];
	if (problems.length) return { ...change, problems };
	// Confirming the key as it is does not make a new version: scores stay comparable.
	const confirmed = change.changes.length === 0;
	const version = confirmed ? task.keyVersion() : change.commit();
	const today = new Date().toISOString().slice(0, 10);
	const log = keyFile(task, 'CHANGELOG.md');
	mkdirSync(dirname(log), { recursive: true });
	const head = existsSync(log) ? '' : `# ${task.title}: answer key changes\n`;
	const entry = confirmed
		? `## v${version} · ${today} · reviewed, no change\n\n${Object.keys(picks)
				.map(id => `- ${id}: kept as it is${picks[id]?.note ? ` (${picks[id].note})` : ''}`)
				.join('\n')}`
		: `## v${version} · ${today}\n\n${change.changes.map(c => `- ${c}`).join('\n')}`;
	appendFileSync(log, `${head}\n${entry}\n`);
	writeQueue(
		task,
		queue.filter(q => !(q.id in picks))
	);
	return { ...change, problems: [], version };
}

function reviewOf(task: Task): KeyReview {
	if (!task.review) throw new Error(`Task ${task.name} has no key review set up (Task.review).`);
	return task.review;
}

function option(args: string[], name: string): string | undefined {
	const at = args.indexOf(`--${name}`);
	return at === -1 ? undefined : args[at + 1];
}

const USAGE = `key queue <task>                                   what waits for the user
key dispute <task> <case id> --note "…" [--propose <option>]
key draft <task> <file.json>                       queue new cases (a JSON list of queue items)
key page <task> [--out file.html]                  build the review page to publish
key apply <task> <picks.json>                      check the user's picks and update the key`;

/** `key …` from the CLI. `root` is the app's evals folder. Returns the exit code. */
export function keyCommand(tasks: TaskRegistry, argv: string[], root: string): number {
	const [action, taskName, ...rest] = argv;
	if (!action || !taskName) {
		console.error(USAGE);
		return 1;
	}
	let task: Task;
	try {
		task = taskNamed(tasks, taskName);
	} catch (error) {
		console.error((error as Error).message);
		return 1;
	}
	const today = new Date().toISOString().slice(0, 10);
	switch (action) {
		case 'queue': {
			const queue = readQueue(task);
			const lines = queue.map(
				q => `${q.agreed ? 'agreed  ' : 'doubtful'}  ${q.source.padEnd(7)}  ${q.id}  ${q.note}`
			);
			process.stdout.write(
				`${task.title}, key v${task.keyVersion()}: ${queue.length} waiting, ${heldoutRuns(task)} held-out runs on this version\n${lines.join('\n')}\n`
			);
			return 0;
		}
		case 'dispute': {
			const [id] = rest;
			const note = option(rest, 'note');
			if (!id || id.startsWith('--') || !note) {
				console.error('key dispute <task> <case id> --note "why the key looks wrong"');
				return 1;
			}
			if (!task.cases().some(c => c.id === id)) {
				console.error(`No case "${id}" in the key. A new case goes in with \`key draft\`.`);
				return 1;
			}
			const proposed = option(rest, 'propose');
			enqueue(task, [
				{ id, source: 'dispute', note, addedAt: today, ...(proposed ? { proposed } : {}) },
			]);
			process.stdout.write(`Queued ${id}.\n`);
			return 0;
		}
		case 'draft': {
			const [file] = rest;
			if (!file || !existsSync(file)) {
				console.error('key draft <task> <file.json>: a JSON list of queue items');
				return 1;
			}
			const items = (JSON.parse(readFileSync(file, 'utf8')) as Partial<QueueItem>[]).map(i => ({
				addedAt: today,
				...i,
			}));
			const bad = items.filter(i => !i.id || !i.note || !SOURCES.includes(i.source as QueueSource));
			if (bad.length) {
				console.error(
					`Each item needs id, source (${SOURCES.join(' | ')}) and note. ${bad.length} do not.`
				);
				return 1;
			}
			const review = reviewOf(task);
			// Building each item proves the task can show it before the user is asked about it.
			for (const item of items) review.item(item as QueueItem);
			process.stdout.write(`${enqueue(task, items as QueueItem[]).length} waiting.\n`);
			return 0;
		}
		case 'page': {
			const out = option(rest, 'out') ?? join(root, '.runs', `${task.name}-review.html`);
			mkdirSync(dirname(out), { recursive: true });
			writeFileSync(out, buildReviewPage(task, reviewOf(task)));
			process.stdout.write(
				`${out}\nPublish it with the db capability; the picks are saved in the page's database as document review/picks.\n`
			);
			return 0;
		}
		case 'apply': {
			const [file] = rest;
			if (!file || !existsSync(file)) {
				console.error(
					'key apply <task> <picks.json>: {"<case id>": {"value": "<option>", "note": "…"}}'
				);
				return 1;
			}
			const picks = JSON.parse(readFileSync(file, 'utf8')) as Record<string, Pick>;
			const result = applyPicks(task, reviewOf(task), picks);
			if (result.problems.length) {
				console.error(
					`Nothing was changed. Ask the user about these again:\n${result.problems.map(p => `- ${p}`).join('\n')}`
				);
				return 1;
			}
			process.stdout.write(
				result.changes.length
					? `Key is now v${result.version}:\n${result.changes.map(c => `- ${c}`).join('\n')}\n`
					: `Reviewed, no change: the key stays v${result.version}.\n`
			);
			return 0;
		}
		default:
			console.error(USAGE);
			return 1;
	}
}
