// What an app tells the harness, in `evals.config.ts` in its evals folder: the tasks it measures and
// the name its model calls go out under. The CLI runs from that folder, and keeps its runs
// (`.runs/`) and promptfoo's data (`.promptfoo/`) beside the config.
import { existsSync } from 'node:fs';
import { join } from 'node:path';

import type { TaskRegistry } from './core/task.ts';

export interface EvalsConfig {
	/** every task, by the name the commands take */
	tasks: TaskRegistry;
	/** the name OpenRouter shows for these calls (its `X-Title` header); `llm-evals` when left out */
	title?: string;
}

export const CONFIG_FILE = 'evals.config.ts';

/**
 * Type the app's config.
 * @param config - The app's tasks and title
 */
export function defineConfig(config: EvalsConfig): EvalsConfig {
	return config;
}

/**
 * The config in `cwd`.
 * @param cwd - The app's evals folder
 */
export async function loadConfig(cwd: string): Promise<EvalsConfig> {
	const path = join(cwd, CONFIG_FILE);
	if (!existsSync(path)) {
		throw new Error(`No ${CONFIG_FILE} in ${cwd}; run llm-evals from the app's evals folder.`);
	}
	const { default: config } = (await import(path)) as { default: EvalsConfig };
	return config;
}
