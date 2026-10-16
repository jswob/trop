import { describe, expect, test } from 'vitest';

import { fakeTasks } from './fake-task.ts';
import type { CacheStore, CallApiContext } from './promptfoo.ts';
import { createProvider, showSamples, type RowMetadata } from './provider.ts';

const config = { task: 'fake', strategy: 'upper', model: 'fast', samples: 3 };

function context(): CallApiContext {
	const store = new Map<string, unknown>();
	const cache: CacheStore = {
		get: key => Promise.resolve(store.get(key)),
		set: (key, value) => Promise.resolve(store.set(key, value)),
	};
	return { vars: { id: 'word:alpha', word: 'alpha' }, getCache: () => cache };
}

describe('VariantProvider', () => {
	test('takes every sample itself and returns them as one row', async () => {
		const Provider = createProvider(fakeTasks());
		const provider = new Provider({ config });
		const response = await provider.callApi('', context());
		expect(response.output).toBe('1. ALPHA\n2. ALPHA\n3. ALPHA');
		expect(response.tokenUsage).toMatchObject({ numRequests: 3, total: 36 });
		expect((response.metadata as unknown as RowMetadata).samples).toHaveLength(3);
		expect(provider.id()).toBe('fake:upper/fast');
		expect('label' in provider).toBe(false);
	});

	test('samples are separate calls, and a second run finds them all cached', async () => {
		let calls = 0;
		const Provider = createProvider(fakeTasks({ answer: word => (calls++, word.toUpperCase()) }));
		const provider = new Provider({ config });
		const shared = context();
		await provider.callApi('', shared);
		expect(calls).toBe(3);
		const again = await provider.callApi('', shared);
		expect(calls).toBe(3);
		expect(again.cached).toBe(true);
	});

	test('a failed sample fails the row, and the retry pays only for what is missing', async () => {
		let calls = 0;
		const limited = {
			kind: 'rate_limit',
			status: 429,
			retryAfterSeconds: 1,
			message: 'OpenRouter 429',
		};
		const Provider = createProvider(
			fakeTasks({
				answer(word, sample) {
					calls++;
					// eslint-disable-next-line @typescript-eslint/only-throw-error -- Reason: a classified failure is recognised by its shape, as an adapter's error would be
					if (sample === 2 && calls === 3) throw limited;
					return word.toUpperCase();
				},
			})
		);
		const provider = new Provider({ config });
		const shared = context();
		const first = await provider.callApi('', shared);
		expect(first.output).toBeUndefined();
		expect(first.metadata).toMatchObject({ failure: 'rate_limit', http: { status: 429 } });
		const retry = await provider.callApi('', shared);
		expect(retry.output).toBe('1. ALPHA\n2. ALPHA\n3. ALPHA');
		expect(calls).toBe(4);
	});

	test('rejects a row that is not a case', async () => {
		const Provider = createProvider(fakeTasks());
		const response = await new Provider({ config }).callApi('', { vars: {} });
		expect(response.error).toMatch(/not a case/);
	});

	test('rejects an unknown task, strategy or model setting', () => {
		const Provider = createProvider(fakeTasks());
		expect(() => new Provider({ config: { ...config, task: 'nope' } })).toThrow(/Unknown task/);
		expect(() => new Provider({ config: { ...config, strategy: 'nope' } })).toThrow(/strategy/);
		expect(() => new Provider({ config: { ...config, model: 'nope' } })).toThrow(/model setting/);
	});
});

describe('showSamples', () => {
	test('numbers several answers and leaves a single one bare', () => {
		expect(showSamples(['a'])).toBe('a');
		expect(showSamples(['a', 'b'])).toBe('1. a\n2. b');
	});
});
