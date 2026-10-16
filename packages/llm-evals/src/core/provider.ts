// The promptfoo provider behind every column: one variant (a strategy on a model setting) answering
// one case. It takes all the samples itself, so the viewer shows one row per case and counts cases,
// not samples. promptfoo's own --repeat is not used.
import { errorResponse } from './outcome.ts';
import type { CallApiContext, ProviderOptions, ProviderResponse } from './promptfoo.ts';
import {
	taskNamed,
	type ModelSetting,
	type Step,
	type Strategy,
	type Task,
	type TaskRegistry,
} from './task.ts';

/** A column's `config` in the generated promptfoo config. */
export interface VariantConfig {
	task: string;
	strategy: string;
	model: string;
	samples: number;
}

/** What the scorer and the report read back from a row. */
export interface SampleRecord {
	decision: unknown;
	shown: string;
	steps: Step[];
}

export interface RowMetadata {
	task: string;
	variant: string;
	samples: SampleRecord[];
}

function lookUp<T>(map: Record<string, T>, key: unknown, what: string): T {
	const value = typeof key === 'string' ? map[key] : undefined;
	if (value === undefined) {
		throw new Error(`Unknown ${what} "${String(key)}". Known: ${Object.keys(map).join(', ')}`);
	}
	return value;
}

/** One answer per line, numbered when there are several. */
export function showSamples(shown: string[]): string {
	return shown.length === 1 ? (shown[0] ?? '') : shown.map((s, i) => `${i + 1}. ${s}`).join('\n');
}

export function createProvider(tasks: TaskRegistry) {
	// No `label` member: promptfoo applies the config's label only when the provider has none.
	return class VariantProvider {
		readonly task: Task;
		readonly strategy: Strategy<unknown, unknown>;
		readonly model: ModelSetting;
		readonly samples: number;
		readonly variant: string;

		constructor(options: ProviderOptions) {
			const config = (options.config ?? {}) as Partial<VariantConfig>;
			this.task = taskNamed(tasks, config.task);
			this.strategy = lookUp(this.task.strategies, config.strategy, 'strategy');
			this.model = lookUp(this.task.models, config.model, 'model setting');
			const samples = config.samples;
			this.samples =
				typeof samples === 'number' && Number.isInteger(samples) && samples > 0 ? samples : 1;
			this.variant = `${config.strategy}/${config.model}`;
		}

		id(): string {
			return `${this.task.name}:${this.variant}`;
		}

		async callApi(_prompt: string, context?: CallApiContext): Promise<ProviderResponse> {
			const input = this.task.input(context?.vars ?? {});
			if (input === null) return { error: `${this.task.name}: this row's vars are not a case` };
			const samples: SampleRecord[] = [];
			try {
				// In sequence: a rate limit on the third sample fails the row, promptfoo retries it,
				// and the first two come back from the cache at no cost.
				for (let i = 0; i < this.samples; i++) {
					const run = await this.strategy.run(input, this.model, {
						vars: {},
						...context,
						repeatIndex: i,
					});
					samples.push({
						decision: run.decision,
						shown: this.task.show(run.decision, input),
						steps: run.steps,
					});
				}
			} catch (error) {
				return errorResponse(error);
			}
			const steps = samples.flatMap(s => s.steps);
			const sum = (pick: (s: Step) => number) => steps.reduce((t, s) => t + pick(s), 0);
			const prompt = sum(s => s.promptTokens);
			const completion = sum(s => s.completionTokens);
			const metadata: RowMetadata = { task: this.task.name, variant: this.variant, samples };
			return {
				output: showSamples(samples.map(s => s.shown)),
				// What the answers cost when they were taken, cached or not, so variants compare on price.
				// What this run really spent is in its spend file (core/budget.ts).
				cost: sum(s => s.cost),
				latencyMs: sum(s => s.ms),
				cached: steps.every(s => s.cached),
				tokenUsage: { prompt, completion, total: prompt + completion, numRequests: steps.length },
				metadata: { ...metadata },
			};
		}
	};
}
