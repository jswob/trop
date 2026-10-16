// Test helper: a run record as `promptfoo export eval` writes it, built from a few lines per row.
export interface FixtureRow {
	variant: string;
	column?: number;
	caseId: string;
	/** the outcome of each sample, e.g. 'ccw' = correct, correct, wrong; 'f' = fell back */
	samples?: string;
	shown?: string[];
	want?: string;
	error?: { failure?: string; message?: string };
	cost?: number;
	latencyMs?: number;
	byok?: boolean;
	scores?: Record<string, number>;
}

const OUTCOME: Record<string, string> = { c: 'correct', w: 'wrong', f: 'fell_back', i: 'invalid' };

export function fixtureRecord(rows: FixtureRow[], tags: Record<string, string> = { run: 'r1' }) {
	return {
		evalId: 'eval-fixture',
		config: { description: 'Fixture run', tags },
		results: {
			version: 3,
			prompts: [],
			results: rows.map(row => {
				const letters = [...(row.samples ?? '')];
				const share = (letter: string) =>
					letters.filter(l => l === letter).length / (letters.length || 1);
				const shown =
					row.shown ?? letters.map(l => (l === 'c' ? 'RIGHT' : l === 'f' ? '↩' : 'other'));
				const want = row.want ?? 'RIGHT';
				const reason = letters
					.map((l, i) => `${i + 1}. ${OUTCOME[l]}: got ${shown[i]}, key says ${want}`)
					.join('\n');
				return {
					provider: { id: 'file://provider.ts', label: row.variant },
					promptIdx: row.column ?? 0,
					vars: { id: row.caseId },
					success: !row.error && !letters.includes('w'),
					failureReason: row.error ? 2 : letters.includes('w') ? 1 : 0,
					...(row.error ? { error: row.error.message ?? 'boom' } : {}),
					cost: row.cost ?? 0.0003,
					latencyMs: row.latencyMs ?? 3000,
					namedScores: row.error
						? {}
						: {
								correct: share('c'),
								fell_back: share('f'),
								wrong: share('w'),
								invalid: share('i'),
								ever_wrong: letters.includes('w') ? 1 : 0,
								all_agree: new Set(shown).size === 1 ? 1 : 0,
								...row.scores,
							},
					gradingResult: row.error
						? null
						: {
								// As promptfoo writes it: a passing row's own reason is replaced.
								reason: letters.includes('w') ? reason : 'All assertions passed',
								componentResults: [{ pass: !letters.includes('w'), reason }],
							},
					metadata: {
						split: 'testset',
						...(row.error ? { failure: row.error.failure } : {}),
						...(row.error
							? {}
							: {
									samples: shown.map(s => ({
										decision: {},
										shown: s,
										steps: [
											{
												name: 's',
												model: 'm',
												cached: false,
												cost: 0.0001,
												ms: 1000,
												promptTokens: 1,
												completionTokens: 1,
												reasoningTokens: 0,
												answer: s,
												...(row.byok === undefined ? {} : { byok: row.byok }),
											},
										],
									})),
								}),
					},
				};
			}),
		},
	};
}
