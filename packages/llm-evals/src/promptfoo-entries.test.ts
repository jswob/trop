import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { afterEach, beforeEach, describe, expect, test } from 'vitest';

import { loadConfig } from './config.ts';
import { writeEntryFiles } from './promptfoo-entries.ts';

let root: string;

beforeEach(() => {
	root = mkdtempSync(join(tmpdir(), 'llm-evals-entries-'));
});

afterEach(() => {
	rmSync(root, { recursive: true, force: true });
});

/** An app's config listing the self-test's task. */
function writeConfig() {
	const task = join(import.meta.dirname, 'selftest', 'task.ts');
	writeFileSync(
		join(root, 'evals.config.ts'),
		`import { SELFTEST_TASKS } from '${task}';\n\nexport default { tasks: SELFTEST_TASKS };\n`
	);
}

describe('writeEntryFiles', () => {
	test('writes files that hand the app’s tasks to the harness', async () => {
		writeConfig();
		const entry = writeEntryFiles(root);

		const { default: Provider } = (await import(entry.provider)) as {
			default: new (options: { config: Record<string, unknown> }) => { id(): string };
		};
		const scorer = ((await import(entry.scorer)) as { default: unknown }).default;
		const hook = ((await import(entry.hooks)) as { hook: unknown }).hook;

		const provider = new Provider({
			config: { task: 'selftest', strategy: 'echo', model: 'fake' },
		});
		expect(provider.id()).toBe('selftest:echo/fake');
		expect(typeof scorer).toBe('function');
		expect(typeof hook).toBe('function');
	});
});

describe('loadConfig', () => {
	test('reads evals.config.ts from the folder', async () => {
		writeConfig();
		expect(Object.keys((await loadConfig(root)).tasks)).toEqual(['selftest']);
	});

	test('names the file it looked for when the folder has none', async () => {
		await expect(loadConfig(root)).rejects.toThrow(`No evals.config.ts in ${root}`);
	});
});
