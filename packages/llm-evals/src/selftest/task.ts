// The self-test's task: the "model" is a fake OpenRouter server (selftest.ts) that upper-cases a
// word, and misbehaves on words that name a failure. Everything between the CLI and that server is
// the real thing: the client, the provider, the scorer, the hooks, and promptfoo itself.
import {
	chatJson,
	chatJsonBody,
	type ChatJsonRequest,
	cachedCall,
	defineTask,
	type Case,
	type TaskRegistry,
} from '../api.ts';

/** Cases by scenario (their `group`). What each word makes the fake server do is in selftest.ts. */
export const SCENARIOS: Record<string, string[]> = {
	main: ['ok-one', 'ok-two', 'wrong', 'rate3', 'flaky', 'in200', 'nochoices'],
	dead: ['dead', 'ok-three'],
	budget: ['b1', 'b2', 'b3', 'b4', 'b5', 'b6'],
	slow: ['slow1', 'slow2', 'slow3', 'slow4', 'slow5', 'slow6'],
	stuck: ['stuck', 'ok-four'],
};

/** The held-out cases: a run on them needs a reason and is written in the ledger. */
export const HELDOUT = ['held1', 'held2'];

const caseOf = (word: string, group: string, split = 'testset'): Case => ({
	id: `word:${word}`,
	description: word,
	vars: { id: `word:${word}`, word },
	split,
	group,
});

const selftestTask = defineTask<{ word: string }, { text: string; invalid: boolean }>({
	name: 'selftest',
	title: 'Self-test',
	dir: process.env.SELFTEST_ROOT ?? '/nonexistent',
	keyVersion: () => '1',
	cases: () => [
		...Object.entries(SCENARIOS).flatMap(([group, words]) => words.map(w => caseOf(w, group))),
		...HELDOUT.map(w => caseOf(w, 'held', 'heldout')),
	],
	tryCases(args) {
		// `--word hello --word there`
		const words = args.filter((_arg, i) => args[i - 1] === '--word');
		if (words.length === 0) throw new Error('Give at least one word: --word hello');
		return words.map(w => ({ ...caseOf(w, 'try', 'try'), id: `try:${w}` }));
	},
	input: vars => (typeof vars.word === 'string' ? { word: vars.word } : null),
	strategies: {
		echo: {
			calls: 1,
			async run(input, model, context) {
				const request: ChatJsonRequest = {
					model: model.model,
					messages: [{ role: 'user', content: `WORD: ${input.word}` }],
					schema: {
						name: 'echo',
						schema: {
							type: 'object',
							additionalProperties: false,
							required: ['text'],
							properties: { text: { type: 'string' } },
						},
					},
				};
				const { value, cached } = await cachedCall(
					context,
					chatJsonBody(request),
					() => chatJson(request),
					result => result.cost
				);
				const text = (value.json as { text?: unknown } | undefined)?.text;
				return {
					decision: {
						text: typeof text === 'string' ? text : '',
						invalid: typeof text !== 'string',
					},
					steps: [
						{
							name: 'echo',
							model: model.model,
							cached,
							cost: value.cost,
							ms: value.ms,
							promptTokens: value.promptTokens,
							completionTokens: value.completionTokens,
							reasoningTokens: 0,
							answer: value.json,
							byok: value.byok,
							...(typeof text === 'string' ? {} : { invalid: 'no text' }),
						},
					],
				};
			},
		},
	},
	models: {
		fake: {
			model: 'fake/upper-1',
			expectByok: true,
			estCostPerCall: 0.001,
			requestsPerMinute: 600,
			tokensPerMinute: 60_000,
			tokensPerCall: 200,
			secondsPerCall: 0.5,
		},
	},
	champion: 'echo/fake',
	samples: 3,
	canary: { id: 'canary:word', word: 'canary' },
	show: decision => decision.text,
	judge(caseId, decision) {
		if (!caseId.startsWith('word:')) return undefined;
		const want = caseId.slice(5).toUpperCase();
		const ok = decision.text === want;
		return { outcome: ok ? 'correct' : 'wrong', metrics: {}, want, got: decision.text };
	},
});

export const SELFTEST_TASKS: TaskRegistry = { selftest: selftestTask };
