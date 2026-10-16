// OpenRouter, the one API every model call goes through: chat models answering with JSON that must
// match a strict schema. Plain fetch, no SDK: the same call shapes production code will use.

export type Effort = 'none' | 'minimal' | 'low' | 'medium' | 'high';

export interface ChatMessage {
	role: 'system' | 'user' | 'assistant';
	content: string;
}

export interface JsonSchema {
	name: string;
	schema: Record<string, unknown>;
}

export interface ChatJsonRequest {
	model: string;
	messages: ChatMessage[];
	schema: JsonSchema;
	/** Reasoning effort; omitted when the model has no reasoning switch. */
	effort?: Effort;
	/** Sent only when set: with require_parameters, a temperature some providers lack blocks routing. */
	temperature?: number;
	maxTokens?: number;
}

export interface ChatJsonResult {
	/** The model's answer, parsed but not yet validated against the schema; undefined when invalid. */
	json: unknown;
	/** Why the answer is unusable (not JSON, refused, cut off). A model failure, still billed. */
	invalid?: string;
	cost: number;
	promptTokens: number;
	completionTokens: number;
	reasoningTokens: number;
	ms: number;
	/** The upstream provider OpenRouter routed to. */
	provider: string | null;
	/** True when the call ran on the user's own provider key rather than OpenRouter's shared pool. */
	byok: boolean;
}

interface ChatResponse {
	provider?: string;
	choices?: {
		finish_reason?: string;
		message?: { content?: string | null; refusal?: string | null };
	}[];
	usage?: {
		prompt_tokens?: number;
		completion_tokens?: number;
		cost?: number;
		is_byok?: boolean;
		cost_details?: { upstream_inference_cost?: number };
		completion_tokens_details?: { reasoning_tokens?: number };
	};
}

/**
 * What a call really cost. With the user's own provider key (BYOK), `cost` is only OpenRouter's
 * fee and the provider bills `upstream_inference_cost` directly; otherwise `cost` already covers both.
 */
export function callCost(usage: ChatResponse['usage']): number {
	const fee = usage?.cost ?? 0;
	return usage?.is_byok ? fee + (usage.cost_details?.upstream_inference_cost ?? 0) : fee;
}

const REQUEST_TIMEOUT_MS = 90_000;

/**
 * Why a call failed, which decides what the runner does next:
 * - rate_limit: wait as long as the server says, then retry
 * - transient: an overload, timeout or broken reply; retry with backoff
 * - fatal: the key, the credit or the model is wrong; no call will work, so stop the run
 * - rejected: OpenRouter refused this request (e.g. a bad schema); retrying cannot help
 */
export type FailureKind = 'rate_limit' | 'transient' | 'fatal' | 'rejected';

export function failureKind(status: number): FailureKind {
	if (status === 429) return 'rate_limit';
	if ([401, 402, 403, 404].includes(status)) return 'fatal';
	if (status === 408 || status >= 500 || status === 0) return 'transient';
	return 'rejected';
}

/** A failed call. It never stands for a model answer: the runner retries it or stops. */
export class OpenRouterError extends Error {
	readonly kind: FailureKind;
	constructor(
		/** HTTP status, the code of an error inside a 200 body, or 0 when no reply arrived */
		readonly status: number,
		readonly retryAfterSeconds: number | null,
		body: string,
		kind: FailureKind = failureKind(status)
	) {
		super(`OpenRouter ${status}: ${body.slice(0, 400)}`);
		this.kind = kind;
	}
}

/**
 * Seconds to wait before retrying, or null when the response does not say. OpenRouter's own limits
 * (e.g. 20 requests a minute per model on a new account) send X-RateLimit-Reset in epoch ms;
 * providers may send Retry-After in seconds or as a date.
 */
function retryAfter(response: Response): number | null {
	const reset = Number(response.headers.get('x-ratelimit-reset'));
	if (reset > 0) return Math.max(0, (reset - Date.now()) / 1000) + 0.5;
	const value = response.headers.get('retry-after');
	if (value === null) return null;
	const seconds = Number(value);
	if (Number.isFinite(seconds)) return Math.max(0, seconds);
	const date = Date.parse(value);
	return Number.isNaN(date) ? null : Math.max(0, (date - Date.now()) / 1000);
}

/**
 * The error a 200 body carries instead of an answer: its code, and the wait from a provider message
 * such as "Please try again in 192ms" / "in 1.5s". Null for a normal answer.
 */
export function errorInBody(text: string): { status: number; wait: number | null } | null {
	let json: unknown;
	try {
		json = JSON.parse(text);
	} catch {
		return null;
	}
	const error = (json as { error?: { code?: unknown; message?: unknown } } | null)?.error;
	if (!error) return null;
	const status = typeof error.code === 'number' ? error.code : 502;
	const match = /try again in (\d+(?:\.\d+)?)(ms|s)\b/i.exec(
		typeof error.message === 'string' ? error.message : ''
	);
	const wait = match ? Number(match[1]) / (match[2] === 'ms' ? 1000 : 1) : null;
	return { status, wait };
}

/**
 * One request to OpenRouter: the JSON body and wall time, or an OpenRouterError. It never retries.
 * promptfoo's scheduler owns waiting and retrying, for the whole queue at once; a second retry loop
 * here would multiply its attempts and hide the rate limits it paces itself by.
 */
async function send<T>(
	method: 'GET' | 'POST',
	path: string,
	body?: Record<string, unknown>
): Promise<{ data: T; ms: number }> {
	const key = process.env.OPENROUTER_API_KEY;
	if (!key) throw new Error('OPENROUTER_API_KEY is not set. Add it to the root .env.');
	const base = process.env.OPENROUTER_BASE_URL ?? 'https://openrouter.ai/api/v1';

	const started = performance.now();
	let response: Response;
	let text: string;
	try {
		response = await fetch(`${base}${path}`, {
			method,
			headers: {
				Authorization: `Bearer ${key}`,
				'Content-Type': 'application/json',
				// The app's `title` from evals.config.ts, which the CLI puts in the environment.
				'X-Title': process.env.EVALS_TITLE ?? 'llm-evals',
			},
			...(body ? { body: JSON.stringify(body) } : {}),
			signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
		});
		text = await response.text();
	} catch (error) {
		throw new OpenRouterError(
			0,
			null,
			`no reply: ${error instanceof Error ? error.message : String(error)}`
		);
	}
	// A provider error can arrive inside an HTTP 200 (seen: OpenAI's tokens-per-minute 429 on a BYOK
	// key), so the body decides, not the status line.
	const failure = response.ok ? errorInBody(text) : { status: response.status, wait: null };
	if (failure)
		throw new OpenRouterError(failure.status, retryAfter(response) ?? failure.wait, text);
	try {
		return { data: JSON.parse(text) as T, ms: Math.round(performance.now() - started) };
	} catch {
		throw new OpenRouterError(502, null, `reply is not JSON: ${text.slice(0, 120)}`);
	}
}

// --- chat models ------------------------------------------------------------------------------

export function chatJsonBody(request: ChatJsonRequest): Record<string, unknown> {
	return {
		model: request.model,
		messages: request.messages,
		response_format: {
			type: 'json_schema',
			json_schema: { name: request.schema.name, strict: true, schema: request.schema.schema },
		},
		...(request.effort ? { reasoning: { effort: request.effort, exclude: true } } : {}),
		...(request.temperature === undefined ? {} : { temperature: request.temperature }),
		// Only when asked: Azure lists max_completion_tokens, not max_tokens, so sending it by default
		// would drop Azure from the providers require_parameters lets OpenRouter fall back to.
		...(request.maxTokens === undefined ? {} : { max_tokens: request.maxTokens }),
		// Route only to providers that honour every parameter above (strict schema above all).
		provider: { require_parameters: true },
		usage: { include: true },
	};
}

export async function chatJson(request: ChatJsonRequest): Promise<ChatJsonResult> {
	const { data, ms } = await send<ChatResponse>('POST', '/chat/completions', chatJsonBody(request));
	const choice = data.choices?.[0];
	// No choice at all is a broken reply, not a model answer: scoring or caching it would count an
	// infrastructure failure as a wrong answer.
	if (!choice) throw new OpenRouterError(502, null, 'answered without choices');
	const content = choice?.message?.content ?? '';
	let json: unknown;
	let invalid: string | undefined;
	if (choice?.message?.refusal) invalid = `refused: ${choice.message.refusal}`;
	else if (choice?.finish_reason === 'length') invalid = 'cut off at max_tokens';
	else {
		try {
			json = JSON.parse(content);
		} catch {
			invalid = `not JSON: ${content.slice(0, 120)}`;
		}
	}
	return {
		json,
		...(invalid ? { invalid } : {}),
		cost: callCost(data.usage),
		promptTokens: data.usage?.prompt_tokens ?? 0,
		completionTokens: data.usage?.completion_tokens ?? 0,
		reasoningTokens: data.usage?.completion_tokens_details?.reasoning_tokens ?? 0,
		ms,
		provider: data.provider ?? null,
		byok: data.usage?.is_byok === true,
	};
}

// --- the key itself ---------------------------------------------------------------------------

export interface KeyInfo {
	label: string;
	/** Credit left on the key in dollars; null when the key has no limit of its own. */
	limitRemaining: number | null;
	isFreeTier: boolean;
}

interface KeyResponse {
	data?: { label?: string; limit_remaining?: number | null; is_free_tier?: boolean };
}

/** What the user can do about each kind of trouble the health check finds, in OpenRouter's terms. */
export const ADVICE = {
	fatal: 'Check OPENROUTER_API_KEY in the root .env, the credit on the key, and the model id.',
	shared_pool:
		'In OpenRouter → Settings → Integrations (BYOK), add the provider key and set "never use shared capacity". The shared pool is limited to 20 requests a minute on a new account.',
};

/** What OpenRouter says about the key in use: `doctor` reads it before any paid call. */
export async function keyInfo(): Promise<KeyInfo> {
	const { data } = await send<KeyResponse>('GET', '/key');
	return {
		label: data.data?.label ?? '',
		limitRemaining: data.data?.limit_remaining ?? null,
		isFreeTier: data.data?.is_free_tier === true,
	};
}
