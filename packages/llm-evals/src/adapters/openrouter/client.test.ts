import { describe, expect, test, vi } from 'vitest';

import {
	callCost,
	chatJson,
	chatJsonBody,
	errorInBody,
	failureKind,
	OpenRouterError,
} from './client.ts';

const request = {
	model: 'openai/gpt-6-luna',
	messages: [{ role: 'user' as const, content: 'WORD: hello' }],
	schema: { name: 'echo', schema: {} },
};

const ok = () =>
	Response.json({
		provider: 'OpenAI',
		choices: [{ finish_reason: 'stop', message: { content: '{"person":true}' } }],
		usage: { prompt_tokens: 10, completion_tokens: 2, cost: 0.00001 },
	});
const limited = (retryAfter = '0') =>
	new Response('{"error":{"code":429,"message":"Rate limit exceeded"}}', {
		status: 429,
		headers: { 'retry-after': retryAfter },
	});

function stubFetch(...responses: (() => Response)[]) {
	vi.stubEnv('OPENROUTER_API_KEY', 'test-key');
	const fetch = vi.fn(() => Promise.resolve((responses.shift() ?? ok)()));
	vi.stubGlobal('fetch', fetch);
	return fetch;
}

const failureOf = async (run: Promise<unknown>) =>
	run.then(
		() => undefined,
		(e: unknown) => e
	);

describe('chatJson', () => {
	test('makes one attempt and reports a 429 with the wait it was told', async () => {
		const fetch = stubFetch(() => limited('7'));
		const error = await failureOf(chatJson(request));
		expect(error).toBeInstanceOf(OpenRouterError);
		expect(error).toMatchObject({ status: 429, kind: 'rate_limit', retryAfterSeconds: 7 });
		expect(fetch).toHaveBeenCalledTimes(1);
	});

	test('reads the wait from the minute window OpenRouter names', async () => {
		stubFetch(
			() =>
				new Response('{"error":{"code":429}}', {
					status: 429,
					headers: { 'x-ratelimit-limit': '20', 'x-ratelimit-reset': String(Date.now() + 3000) },
				})
		);
		const error = (await failureOf(chatJson(request))) as OpenRouterError;
		expect(error.retryAfterSeconds).toBeGreaterThan(3);
		expect(error.retryAfterSeconds).toBeLessThan(4);
	});

	test('a rate limit inside an HTTP 200 is a failure, with the wait from its message', async () => {
		stubFetch(() =>
			Response.json({
				error: {
					code: 429,
					message:
						'Rate limit reached for gpt-6-luna on tokens per min (TPM). Please try again in 5ms.',
				},
			})
		);
		expect(await failureOf(chatJson(request))).toMatchObject({
			status: 429,
			kind: 'rate_limit',
			retryAfterSeconds: 0.005,
		});
	});

	test('a reply without choices is a transient failure, never an answer', async () => {
		stubFetch(() => Response.json({ provider: 'OpenAI', usage: {} }));
		expect(await failureOf(chatJson(request))).toMatchObject({ kind: 'transient' });
	});

	test('no reply at all is a transient failure', async () => {
		vi.stubEnv('OPENROUTER_API_KEY', 'test-key');
		vi.stubGlobal('fetch', () => Promise.reject(new Error('socket hang up')));
		expect(await failureOf(chatJson(request))).toMatchObject({ status: 0, kind: 'transient' });
	});

	test('a request OpenRouter rejects is not worth retrying', async () => {
		const fetch = stubFetch(() => new Response('bad schema', { status: 400 }));
		const error = await failureOf(chatJson(request));
		expect(error).toMatchObject({ kind: 'rejected', message: 'OpenRouter 400: bad schema' });
		expect(fetch).toHaveBeenCalledTimes(1);
	});

	test('says whether the call ran on the user’s own key', async () => {
		stubFetch(() =>
			Response.json({
				choices: [{ finish_reason: 'stop', message: { content: '{}' } }],
				usage: { is_byok: true, cost: 0, cost_details: { upstream_inference_cost: 0.00002 } },
			})
		);
		expect(await chatJson(request)).toMatchObject({ byok: true, cost: 0.00002 });
	});
});

describe('failureKind', () => {
	test.each([
		[429, 'rate_limit'],
		[401, 'fatal'],
		[402, 'fatal'],
		[404, 'fatal'],
		[408, 'transient'],
		[503, 'transient'],
		[529, 'transient'],
		[400, 'rejected'],
	])('%i is %s', (status, kind) => {
		expect(failureKind(status)).toBe(kind);
	});
});

describe('chatJsonBody', () => {
	test('leaves max_tokens out unless asked, so Azure stays a fallback', () => {
		expect(chatJsonBody(request)).not.toHaveProperty('max_tokens');
		expect(chatJsonBody({ ...request, maxTokens: 500 })).toMatchObject({ max_tokens: 500 });
	});
});

describe('callCost', () => {
	test("adds what the provider bills when the call used the user's own key", () => {
		const usage = { cost: 0, is_byok: true, cost_details: { upstream_inference_cost: 0.0000035 } };
		expect(callCost(usage)).toBe(0.0000035);
	});

	test("takes OpenRouter's cost as the whole price otherwise", () => {
		const usage = {
			cost: 0.0000035,
			is_byok: false,
			cost_details: { upstream_inference_cost: 0.0000035 },
		};
		expect(callCost(usage)).toBe(0.0000035);
	});
});

describe('errorInBody', () => {
	test('reads the code and the wait a provider asks for', () => {
		expect(
			errorInBody('{"error":{"code":429,"message":"Please try again in 192ms. Visit …"}}')
		).toEqual({ status: 429, wait: 0.192 });
		expect(errorInBody('{"error":{"code":429,"message":"Please try again in 1.5s."}}')).toEqual({
			status: 429,
			wait: 1.5,
		});
	});

	test('is null for an answer', () => {
		expect(errorInBody('{"choices":[]}')).toBeNull();
		expect(errorInBody('not json')).toBeNull();
	});
});
