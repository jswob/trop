// The slice of promptfoo's provider and assertion contracts (0.123.1) this package relies on.
// Declared here so no file imports promptfoo: it runs from a pinned npx install (see cli.ts).
import { createHash } from 'node:crypto';

import { reserve, settle } from './budget.ts';

export interface CacheStore {
	get(key: string): Promise<unknown>;
	set(key: string, value: unknown): Promise<unknown>;
}

/** What promptfoo passes to a custom provider's callApi. */
export interface CallApiContext {
	vars: Record<string, unknown>;
	/** promptfoo's own response cache; memory-only when the run uses --no-cache. */
	getCache?: () => CacheStore | Promise<CacheStore>;
	/** Which sample of a case this is. Set by core/provider.ts, which takes the samples itself. */
	repeatIndex?: number;
}

/** What promptfoo passes to a custom provider's constructor; the YAML label is applied afterwards. */
export interface ProviderOptions {
	id?: string;
	config?: unknown;
}

export interface ProviderResponse {
	output?: string;
	error?: string;
	cost?: number;
	latencyMs?: number;
	cached?: boolean;
	tokenUsage?: { prompt: number; completion: number; total: number; numRequests: number };
	metadata?: Record<string, unknown>;
}

/** What promptfoo passes to a `javascript` assertion as its second argument. */
export interface AssertionContext {
	vars: Record<string, unknown>;
	/** providerResponse.metadata */
	metadata?: Record<string, unknown>;
}

export interface GradingResult {
	pass: boolean;
	score: number;
	reason: string;
	namedScores?: Record<string, number>;
}

/**
 * Part of every cache key. Bump it to drop saved answers a client bug got wrong
 * (v2: rate-limit errors inside HTTP 200 bodies were saved as empty answers).
 */
const CACHE_VERSION = 2;

/**
 * The only path to a paid model call. It runs `run` through promptfoo's cache and the run's
 * spending limit:
 * - the key covers the request and the sample index, so three samples are three calls and a re-run
 *   of an unchanged request costs nothing. That is also how a stopped run is finished: run it
 *   again. promptfoo's own --no-cache is never used: its cache is then memory-only and a second
 *   run would pay twice;
 * - a failure is never cached, and never counts as an answer.
 */
export async function cachedCall<T>(
	context: CallApiContext | undefined,
	request: unknown,
	run: () => Promise<T>,
	costOf: (value: T) => number = () => 0
): Promise<{ value: T; cached: boolean }> {
	const cache = context?.getCache ? await context.getCache() : undefined;
	const digest = createHash('sha256').update(JSON.stringify(request)).digest('hex');
	const key = `llm-evals:v${CACHE_VERSION}:${digest}:${context?.repeatIndex ?? 0}`;
	const hit = await cache?.get(key);
	if (hit !== undefined && hit !== null) return { value: hit as T, cached: true };

	reserve();
	let value: T;
	try {
		value = await run();
	} catch (error) {
		settle(0);
		throw error;
	}
	settle(costOf(value));
	await cache?.set(key, value);
	return { value, cached: false };
}
