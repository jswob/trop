// The CLI on the self-test's task, with its runs and promptfoo data in a throwaway folder.
// Started by selftest.ts, one process per command.
import { join } from 'node:path';

import { ADVICE, keyInfo } from '../adapters/openrouter/client.ts';
import { main } from '../core/cli.ts';
import { SELFTEST_TASKS } from './task.ts';

const root = process.env.SELFTEST_ROOT;
if (!root) throw new Error('SELFTEST_ROOT is not set: run `bun evals:selftest`.');

process.exit(
	await main(process.argv.slice(2), {
		tasks: SELFTEST_TASKS,
		checks: { keyStatus: keyInfo, advice: ADVICE },
		root,
		promptfoo: process.env.SELFTEST_PROMPTFOO ?? 'promptfoo',
		entry: {
			provider: join(import.meta.dirname, 'provider.ts'),
			scorer: join(import.meta.dirname, 'scorer.ts'),
			hooks: join(import.meta.dirname, 'hooks.ts'),
		},
	})
);
