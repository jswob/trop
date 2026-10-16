import { ADVICE, keyInfo } from '../adapters/openrouter/client.ts';
import { createHook } from '../core/hooks.ts';
import { SELFTEST_TASKS } from './task.ts';

export const hook = createHook(SELFTEST_TASKS, { keyStatus: keyInfo, advice: ADVICE });
