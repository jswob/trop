// What a failed model call becomes in promptfoo's terms. promptfoo's scheduler owns waiting and
// retrying (one layer, for the whole queue), so a failure is handed to it in the shape it acts on.
// Checked against promptfoo 0.123.1 by `bun evals:selftest`.
import type { ProviderResponse } from './promptfoo.ts';

/**
 * - rate_limit: wait as long as the server says, then retry
 * - transient: an overload, timeout or broken reply; retry with backoff
 * - fatal: the key, the credit or the model is wrong; stop the run
 * - rejected: the request itself was refused; retrying cannot help
 * - not_run: the spending limit was reached before this call
 */
export type FailureKind = 'rate_limit' | 'transient' | 'fatal' | 'rejected' | 'not_run';

/** A failure a model adapter classified. Core reads the shape, never an adapter's class. */
export interface ModelCallFailure {
	kind: FailureKind;
	status: number;
	retryAfterSeconds: number | null;
	message: string;
}

const KINDS: readonly FailureKind[] = ['rate_limit', 'transient', 'fatal', 'rejected', 'not_run'];

export function isModelCallFailure(error: unknown): error is ModelCallFailure {
	if (typeof error !== 'object' || error === null) return false;
	const failure = error as Partial<ModelCallFailure>;
	return KINDS.includes(failure.kind as FailureKind) && typeof failure.status === 'number';
}

/** The statuses that make promptfoo abort a run ("will not resolve on retry"). */
const ABORTS_THE_RUN = [401, 403, 404];
/** Waits used when the server names none. Without any, promptfoo pauses a rate-limited queue for a minute. */
const DEFAULT_WAIT_MS = { rate_limit: 2000, transient: 1000 };

/**
 * The provider result for a failed call. `metadata.failure` is ours (the integrity gate reads it);
 * the rest is what promptfoo's scheduler reads:
 * - `http.status` 429 with a `retry-after-ms` header: pause the queue that long, then retry the row
 * - `rateLimitKind: 'rate_limit'`: retry with backoff. promptfoo retries a returned error only when
 *   it counts as a rate limit, so an overload is reported as one
 * - `http.status` 401/403/404: abort the run
 */
export function failureResponse(failure: ModelCallFailure): ProviderResponse {
	const { kind, status, message } = failure;
	const wait = (fallback: number) =>
		String(
			failure.retryAfterSeconds === null ? fallback : Math.ceil(failure.retryAfterSeconds * 1000)
		);
	switch (kind) {
		case 'rate_limit':
			return {
				error: message,
				metadata: {
					failure: kind,
					http: { status: 429, headers: { 'retry-after-ms': wait(DEFAULT_WAIT_MS.rate_limit) } },
				},
			};
		case 'transient':
			return {
				error: message,
				metadata: {
					failure: kind,
					rateLimitKind: 'rate_limit',
					http: { status, headers: { 'retry-after-ms': wait(DEFAULT_WAIT_MS.transient) } },
				},
			};
		case 'fatal':
			return {
				error: message,
				metadata: {
					failure: kind,
					// 402 (out of credit) is fatal too, but promptfoo only stops on these three.
					http: { status: ABORTS_THE_RUN.includes(status) ? status : 403, headers: {} },
				},
			};
		case 'rejected':
		case 'not_run':
			return { error: message, metadata: { failure: kind } };
	}
}

/** Any error a provider caught, as a provider result. An unclassified one is a bug of ours: no retry. */
export function errorResponse(error: unknown): ProviderResponse {
	if (isModelCallFailure(error)) return failureResponse(error);
	return {
		error: error instanceof Error ? error.message : String(error),
		metadata: { failure: 'rejected' },
	};
}
