import { mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { beforeEach, describe, expect, test, vi } from 'vitest';

import { NotRunError, reserve, resetBudget, settle } from './budget.ts';
import { isModelCallFailure } from './outcome.ts';

function limit(max: number, perCall = 0.001) {
	const file = join(mkdtempSync(join(tmpdir(), 'budget-')), 'spend.log');
	vi.stubEnv('EVALS_SPEND_FILE', file);
	vi.stubEnv('EVALS_MAX_COST', String(max));
	vi.stubEnv('EVALS_EST_COST_PER_CALL', String(perCall));
	resetBudget();
	return file;
}

beforeEach(() => {
	resetBudget();
});

describe('spending limit', () => {
	test('lets calls through and records what they cost', () => {
		const file = limit(0.01);
		reserve();
		settle(0.002);
		reserve();
		settle(0.003);
		expect(readFileSync(file, 'utf8')).toBe('0.002\n0.003\n');
	});

	test('refuses a call that could cross the limit, counting calls still in flight', () => {
		limit(0.0025);
		reserve();
		reserve();
		expect(() => {
			reserve();
		}).toThrow(NotRunError);
	});

	test('the refusal is a classified failure the provider reports as not run', () => {
		limit(0);
		let failure: unknown;
		try {
			reserve();
		} catch (error) {
			failure = error;
		}
		expect(isModelCallFailure(failure)).toBe(true);
		expect(failure).toMatchObject({ kind: 'not_run' });
	});

	test('a second process continues from what the first one spent', () => {
		const file = limit(0.005);
		writeFileSync(file, '0.002\n0.0025\n');
		resetBudget();
		expect(() => {
			reserve();
		}).toThrow(/limit of \$0.005 was reached \(\$0.0045 spent\)/);
	});

	test('without a limit in the environment, nothing is checked', () => {
		expect(() => {
			reserve();
			settle(5);
		}).not.toThrow();
	});
});
