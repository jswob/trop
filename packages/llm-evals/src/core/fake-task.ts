// Test helper: a tiny task with no model behind it. The "model" upper-cases the word, except for
// what a test scripts through `answers`.
import { cachedCall } from './promptfoo.ts';
import { defineTask, type ModelSetting, type Step, type Task, type TaskRegistry } from './task.ts';

const KEY: Record<string, string> = { 'word:alpha': 'ALPHA', 'word:beta': 'BETA' };

export const fakeModel: ModelSetting = {
	model: 'fake/upper-1',
	estCostPerCall: 0.001,
	requestsPerMinute: 600,
	tokensPerMinute: 60_000,
	tokensPerCall: 100,
	secondsPerCall: 1,
};

export interface FakeScript {
	/** called once per uncached model call; return the answer or throw a failure */
	answer?: (word: string, sample: number) => string;
	/** the task folder, for tests that write a queue or a ledger */
	dir?: string;
}

export function fakeTask(script: FakeScript = {}): Task {
	const step = (answer: string, cached: boolean): Step => ({
		name: 'upper',
		model: fakeModel.model,
		cached,
		cost: 0.001,
		ms: 5,
		promptTokens: 10,
		completionTokens: 2,
		reasoningTokens: 0,
		answer,
	});
	return defineTask<{ word: string }, { text: string }>({
		name: 'fake',
		title: 'Fake task',
		dir: script.dir ?? '/nonexistent/fake',
		keyVersion: () => '1',
		cases: () =>
			Object.keys(KEY).map(id => ({
				id,
				description: id,
				vars: { id, word: id.slice(5) },
				split: id === 'word:alpha' ? 'testset' : 'heldout',
			})),
		tryCases: args =>
			args.map(word => ({
				id: `try:${word}`,
				description: word,
				vars: { id: `try:${word}`, word },
				split: 'try',
			})),
		input: vars => (typeof vars.word === 'string' ? { word: vars.word } : null),
		strategies: {
			upper: {
				calls: 1,
				async run(input, _model, context) {
					const { value, cached } = await cachedCall(
						context,
						{ word: input.word },
						() =>
							Promise.resolve(
								script.answer?.(input.word, context?.repeatIndex ?? 0) ?? input.word.toUpperCase()
							),
						() => 0.001
					);
					return { decision: { text: value }, steps: [step(value, cached)] };
				},
			},
			lower: {
				calls: 1,
				run: input => Promise.resolve({ decision: { text: input.word }, steps: [] }),
			},
		},
		models: { fast: fakeModel, slow: { ...fakeModel, tokensPerMinute: 6000 } },
		champion: 'upper/fast',
		samples: 3,
		canary: { id: 'canary:gamma', word: 'gamma' },
		show: decision => decision.text,
		judge(caseId, decision) {
			const want = KEY[caseId];
			if (want === undefined) return undefined;
			const ok = decision.text === want;
			return {
				outcome: ok ? 'correct' : 'wrong',
				metrics: { typo: ok ? 0 : 1 },
				want,
				got: decision.text,
			};
		},
	});
}

export const fakeTasks = (script?: FakeScript): TaskRegistry => ({ fake: fakeTask(script) });
