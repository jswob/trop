import { existsSync, mkdtempSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { describe, expect, test } from 'vitest';

import { parseArgs, UsageError } from './cli.ts';
import { fakeTask } from './fake-task.ts';
import {
	applyPicks,
	buildReviewPage,
	enqueue,
	heldoutRuns,
	logHeldoutRun,
	readQueue,
	type KeyReview,
} from './key.ts';

const dir = () => mkdtempSync(join(tmpdir(), 'key-'));
const item = (id: string, agreed = false) => ({
	id,
	source: 'dispute' as const,
	note: 'looks wrong',
	addedAt: '2026-10-04',
	agreed,
});

function review(problems: string[] = []): KeyReview & { committed: number } {
	const self = {
		committed: 0,
		guide: () => 'Rule 1.',
		item: (q: { id: string }) => ({
			id: q.id,
			title: q.id,
			options: [{ value: 'yes', label: 'Yes' }],
		}),
		apply: (picks: Record<string, { value: string }>) => ({
			changes: Object.entries(picks).map(([id, p]) => `${id} → ${p.value}`),
			problems,
			commit: () => String(++self.committed + 1),
		}),
	};
	return self;
}

describe('queue', () => {
	test('holds one item per case; a newer one replaces the older', () => {
		const task = fakeTask({ dir: dir() });
		enqueue(task, [item('word:alpha'), item('word:beta')]);
		enqueue(task, [{ ...item('word:alpha'), note: 'second look' }]);
		expect(readQueue(task).map(q => [q.id, q.note])).toEqual([
			['word:beta', 'looks wrong'],
			['word:alpha', 'second look'],
		]);
	});

	test('is empty before anything was queued', () => {
		expect(readQueue(fakeTask({ dir: dir() }))).toEqual([]);
	});
});

describe('review page', () => {
	test('carries doubtful and agreed items apart, and the rules', () => {
		const task = fakeTask({ dir: dir() });
		enqueue(task, [item('word:alpha'), item('word:<b>', true)]);
		const html = buildReviewPage(task, review());
		const data = JSON.parse(
			/<script id="data" type="application\/json">(.*)<\/script>/.exec(html)![1]!
		) as {
			doubtful: { id: string }[];
			agreed: { id: string }[];
			guide: string;
		};
		expect(data.doubtful.map(i => i.id)).toEqual(['word:alpha']);
		expect(data.agreed.map(i => i.id)).toEqual(['word:<b>']);
		expect(data.guide).toBe('Rule 1.');
		expect(html).not.toContain('word:<b>');
	});
});

describe('applyPicks', () => {
	test('writes the key, logs the change and removes the picked cases from the queue', () => {
		const task = fakeTask({ dir: dir() });
		enqueue(task, [item('word:alpha'), item('word:beta')]);
		const r = review();
		const result = applyPicks(task, r, { 'word:alpha': { value: 'yes' } });
		expect(result).toMatchObject({ version: '2', problems: [] });
		expect(readQueue(task).map(q => q.id)).toEqual(['word:beta']);
		expect(readFileSync(join(task.dir, 'key', 'CHANGELOG.md'), 'utf8')).toMatch(
			/## v2 · \d{4}-\d\d-\d\d\n\n- word:alpha → yes/
		);
	});

	test('picks that confirm the key keep its version, and are still logged', () => {
		const task = fakeTask({ dir: dir() });
		enqueue(task, [item('word:alpha')]);
		const r = review();
		r.apply = () => ({ changes: [], problems: [], commit: () => String(++r.committed + 1) });
		const result = applyPicks(task, r, { 'word:alpha': { value: 'yes', note: 'fine as is' } });
		expect(result.version).toBe('1');
		expect(r.committed).toBe(0);
		expect(readQueue(task)).toEqual([]);
		expect(readFileSync(join(task.dir, 'key', 'CHANGELOG.md'), 'utf8')).toMatch(
			/## v1 · \d{4}-\d\d-\d\d · reviewed, no change\n\n- word:alpha: kept as it is \(fine as is\)/
		);
	});

	test('changes nothing when the task finds a problem, or a pick is not in the queue', () => {
		const task = fakeTask({ dir: dir() });
		enqueue(task, [item('word:alpha')]);
		const r = review(['word:alpha: contradicts rule 2']);
		expect(
			applyPicks(task, r, { 'word:alpha': { value: 'yes' }, 'word:zeta': { value: 'yes' } })
				.problems
		).toEqual(['word:zeta: not in the queue', 'word:alpha: contradicts rule 2']);
		expect(r.committed).toBe(0);
		expect(readQueue(task)).toHaveLength(1);
		expect(existsSync(join(task.dir, 'key', 'CHANGELOG.md'))).toBe(false);
	});
});

describe('held-out ledger', () => {
	test('counts the runs made on the current key version', () => {
		const task = fakeTask({ dir: dir() });
		expect(heldoutRuns(task)).toBe(0);
		logHeldoutRun(task, { runId: 'r1', variants: ['upper/fast'], reason: 'choosing' });
		logHeldoutRun(task, { runId: 'r2', variants: ['upper/fast'] });
		expect(heldoutRuns(task)).toBe(2);
	});
});

describe('parseArgs', () => {
	test('splits our flags from the task’s own', () => {
		const args = parseArgs(['some-task', '--model', 'a,b', '--word', 'hello', '--word', 'there']);
		expect(args.positional).toEqual(['some-task']);
		expect(args.values).toEqual({ model: 'a,b' });
		expect(args.rest).toEqual(['--word', 'hello', '--word', 'there']);
	});

	test('a flag without its value is a usage mistake', () => {
		expect(() => parseArgs(['--max-cost'])).toThrow(UsageError);
	});
});
