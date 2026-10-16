#!/usr/bin/env bun
// The composition root: the app's tasks and the model adapter meet core here. Core imports
// neither, and the app's tasks come from `evals.config.ts` in the folder the CLI runs from.
//
//   llm-evals <doctor|estimate|run|try|continue|report|key|view|selftest> [task] [options]
//
// promptfoo is not a dependency: its install is ~2.2 GB and the Dockerfiles copy node_modules
// into the API image. It runs through a pinned npx and is never imported.
import { join } from 'node:path';

import { ADVICE, keyInfo } from './adapters/openrouter/client.ts';
import { loadConfig } from './config.ts';
import { main } from './core/cli.ts';
import { writeEntryFiles } from './promptfoo-entries.ts';
import { selftest } from './selftest/selftest.ts';

const PROMPTFOO = 'promptfoo@0.123.1';
const argv = process.argv.slice(2);

if (argv[0] === 'selftest') {
	process.exit(await selftest({ root: join(import.meta.dirname, '..'), promptfoo: PROMPTFOO }));
}

// Keys come from the root .env (the app's script passes --env-file). Only the commands that call
// a model need one.
if (
	['doctor', 'run', 'try', 'continue'].includes(argv[0] ?? '') &&
	!process.env.OPENROUTER_API_KEY
) {
	console.error('OPENROUTER_API_KEY is not set. Add it to the root .env.');
	process.exit(1);
}

const root = process.cwd();
const config = await loadConfig(root);
// Read by the adapter, here and in the promptfoo process the CLI starts.
if (config.title) process.env.EVALS_TITLE = config.title;

process.exit(
	await main(argv, {
		tasks: config.tasks,
		checks: { keyStatus: keyInfo, advice: ADVICE },
		root,
		promptfoo: PROMPTFOO,
		entry: writeEntryFiles(root),
	})
);
