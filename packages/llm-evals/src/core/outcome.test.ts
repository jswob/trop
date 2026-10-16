import { describe, expect, test } from 'vitest';

import { errorResponse, failureResponse, isModelCallFailure } from './outcome.ts';

const failure = (
	kind: Parameters<typeof failureResponse>[0]['kind'],
	status: number,
	wait: number | null = null
) => ({
	kind,
	status,
	retryAfterSeconds: wait,
	message: `OpenRouter ${status}: boom`,
});

describe('failureResponse', () => {
	test('a rate limit carries the wait the server named, in milliseconds', () => {
		expect(failureResponse(failure('rate_limit', 429, 0.192)).metadata).toEqual({
			failure: 'rate_limit',
			http: { status: 429, headers: { 'retry-after-ms': '192' } },
		});
	});

	test('a rate limit with no named wait gets a short one, not the scheduler’s minute', () => {
		expect(failureResponse(failure('rate_limit', 429)).metadata).toMatchObject({
			http: { headers: { 'retry-after-ms': '2000' } },
		});
	});

	test('an overload is reported as retryable', () => {
		expect(failureResponse(failure('transient', 503)).metadata).toMatchObject({
			failure: 'transient',
			rateLimitKind: 'rate_limit',
			http: { status: 503 },
		});
	});

	test('a fatal failure uses a status that stops the run', () => {
		expect(failureResponse(failure('fatal', 401)).metadata).toMatchObject({
			http: { status: 401 },
		});
		expect(failureResponse(failure('fatal', 402)).metadata).toMatchObject({
			http: { status: 403 },
		});
	});

	test('a rejected request and a call that was not run carry no status to act on', () => {
		expect(failureResponse(failure('rejected', 400)).metadata).toEqual({ failure: 'rejected' });
		expect(failureResponse(failure('not_run', 0)).metadata).toEqual({ failure: 'not_run' });
	});
});

describe('errorResponse', () => {
	test('an unclassified error is reported once, with no retry hint', () => {
		expect(errorResponse(new Error('bug'))).toEqual({
			error: 'bug',
			metadata: { failure: 'rejected' },
		});
	});

	test('recognises a classified failure by its shape', () => {
		expect(isModelCallFailure(failure('transient', 0))).toBe(true);
		expect(isModelCallFailure(new Error('x'))).toBe(false);
	});
});
