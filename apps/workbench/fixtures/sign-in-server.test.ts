import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { standInServer, WORKBENCH_PASSWORD } from './sign-in-server';

beforeEach(() => {
	vi.useFakeTimers();
});

afterEach(() => {
	vi.useRealTimers();
});

async function settle<T>(answer: Promise<T>): Promise<T | 'pending'> {
	await vi.runAllTimersAsync();
	return Promise.race([answer, Promise.resolve('pending' as const)]);
}

describe('the workbench’s stand-in server', () => {
	it('takes the workbench password and refuses anything else', async () => {
		const server = standInServer('empty');

		expect(await settle(server(WORKBENCH_PASSWORD))).toBe('ok');
		expect(await settle(server('wrong'))).toBe('wrong');
	});

	it('never answers while it is still answering', async () => {
		expect(await settle(standInServer('signing-in')(WORKBENCH_PASSWORD))).toBe('pending');
	});

	it('refuses every press with the fix when no password is set', async () => {
		expect(await settle(standInServer('not-set-up')(WORKBENCH_PASSWORD))).toBe('not-set-up');
	});
});
