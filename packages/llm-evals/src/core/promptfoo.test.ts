import { describe, expect, test, vi } from 'vitest';

import { cachedCall, type CacheStore, type CallApiContext } from './promptfoo.ts';

function memoryContext(repeatIndex = 0): CallApiContext & { store: Map<string, unknown> } {
	const store = new Map<string, unknown>();
	const cache: CacheStore = {
		get: key => Promise.resolve(store.get(key)),
		set: (key, value) => Promise.resolve(store.set(key, value)),
	};
	return { vars: {}, getCache: () => cache, repeatIndex, store };
}

describe('cachedCall', () => {
	test('answers a repeated request from the cache', async () => {
		const context = memoryContext();
		const run = vi.fn(() => Promise.resolve('answer'));
		await cachedCall(context, { a: 1 }, run);
		const second = await cachedCall(context, { a: 1 }, run);
		expect(second).toEqual({ value: 'answer', cached: true });
		expect(run).toHaveBeenCalledTimes(1);
	});

	test('samples again on another repeat of the same request', async () => {
		const context = memoryContext();
		const run = vi.fn(() => Promise.resolve('answer'));
		await cachedCall(context, { a: 1 }, run);
		await cachedCall({ ...context, repeatIndex: 1 }, { a: 1 }, run);
		expect(run).toHaveBeenCalledTimes(2);
	});

	test('never caches a failure', async () => {
		const context = memoryContext();
		await expect(
			cachedCall(context, { a: 1 }, () => Promise.reject(new Error('503')))
		).rejects.toThrow('503');
		expect(context.store.size).toBe(0);
	});

	test('runs every time without a cache', async () => {
		const run = vi.fn(() => Promise.resolve('answer'));
		await cachedCall(undefined, { a: 1 }, run);
		await cachedCall(undefined, { a: 1 }, run);
		expect(run).toHaveBeenCalledTimes(2);
	});
});
