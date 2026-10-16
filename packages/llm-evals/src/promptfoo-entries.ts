// promptfoo loads three files per run (the provider, the scorer and the hooks), and each has to
// hand the app's tasks to the harness. The harness cannot import the app, so before every command
// the CLI writes the three into the app's `.promptfoo/entry/`: each imports the app's config and
// passes it to one of the functions here.
import { mkdirSync, writeFileSync } from 'node:fs';
import { join, relative } from 'node:path';

import { ADVICE, keyInfo } from './adapters/openrouter/client.ts';
import { CONFIG_FILE, type EvalsConfig } from './config.ts';
import type { EntryFiles } from './core/config.ts';
import { createHook } from './core/hooks.ts';
import { createProvider } from './core/provider.ts';
import { createScorer } from './core/scorer.ts';

export const provider = (config: EvalsConfig) => createProvider(config.tasks);
export const scorer = (config: EvalsConfig) => createScorer(config.tasks);
export const hook = (config: EvalsConfig) =>
	createHook(config.tasks, { keyStatus: keyInfo, advice: ADVICE });

/** Per file: what it imports from this module, and what it exports for promptfoo. */
const ENTRIES: Record<keyof EntryFiles, [string, string]> = {
	provider: ['provider', 'export default provider(config);'],
	scorer: ['scorer', 'export default scorer(config);'],
	hooks: ['hook as hookFor', 'export const hook = hookFor(config);'],
};

/** `to` as an import specifier from a file in `from`. */
function specifier(from: string, to: string): string {
	const path = relative(from, to);
	return path.startsWith('.') ? path : `./${path}`;
}

/**
 * Write the three files promptfoo loads into `<root>/.promptfoo/entry/`.
 * @param root - The app's evals folder, which holds `evals.config.ts`
 * @returns Their absolute paths
 */
export function writeEntryFiles(root: string): EntryFiles {
	const dir = join(root, '.promptfoo', 'entry');
	mkdirSync(dir, { recursive: true });
	const config = specifier(dir, join(root, CONFIG_FILE));
	const self = specifier(dir, join(import.meta.dirname, 'promptfoo-entries.ts'));

	const entry = {} as EntryFiles;
	for (const [name, [binding, body]] of Object.entries(ENTRIES) as [
		keyof EntryFiles,
		[string, string],
	][]) {
		const path = join(dir, `${name}.ts`);
		writeFileSync(
			path,
			[
				'// Written by llm-evals before every command, for promptfoo to load. Edits are lost.',
				`import config from '${config}';`,
				`import { ${binding} } from '${self}';`,
				'',
				body,
				'',
			].join('\n')
		);
		entry[name] = path;
	}
	return entry;
}
