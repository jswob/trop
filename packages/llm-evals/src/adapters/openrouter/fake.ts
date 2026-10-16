// Test helper: answers OpenRouter chat calls from a function, so no test reaches the network.
import { vi } from 'vitest';

export interface OpenRouterRequestBody {
	model: string;
	messages: { role: string; content: string }[];
	response_format: { json_schema: { name: string } };
}

export interface FakeModels {
	/** Returns the model's answer; a string is sent as-is (to fake invalid JSON). */
	openrouter?: (body: OpenRouterRequestBody) => unknown;
}

export interface FakeCall {
	url: string;
	body: Record<string, unknown>;
}

export function fakeModels(models: FakeModels): FakeCall[] {
	vi.stubEnv('OPENROUTER_API_KEY', 'test-key');
	const calls: FakeCall[] = [];
	vi.stubGlobal(
		'fetch',
		vi.fn((url: string, init: RequestInit) => {
			// The clients always send a JSON string body.
			const body = JSON.parse(init.body as string) as Record<string, unknown>;
			calls.push({ url, body });
			if (url.endsWith('/chat/completions')) {
				const answer = models.openrouter?.(body as unknown as OpenRouterRequestBody);
				const content = typeof answer === 'string' ? answer : JSON.stringify(answer);
				return Promise.resolve(
					Response.json({
						provider: 'Fake',
						choices: [{ finish_reason: 'stop', message: { content } }],
						usage: { prompt_tokens: 100, completion_tokens: 20, cost: 0.0001 },
					})
				);
			}
			return Promise.resolve(new Response('not found', { status: 404 }));
		})
	);
	return calls;
}

/** The schema name of an OpenRouter call, which tells the prompts apart. */
export const schemaOf = (body: OpenRouterRequestBody) => body.response_format.json_schema.name;
