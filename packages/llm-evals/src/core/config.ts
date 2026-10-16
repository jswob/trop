// Builds the promptfoo config of one run from the task and the picked variants. promptfoo reads
// it as a JSON file; nothing here is written by hand per task.
import type { Variant } from './grid.ts';
import type { VariantConfig } from './provider.ts';
import type { Case, Task } from './task.ts';

/** Absolute paths of the files promptfoo loads, written by the composition root (`promptfoo-entries.ts`). */
export interface EntryFiles {
	provider: string;
	scorer: string;
	hooks: string;
}

export interface RunPlan {
	runId: string;
	task: Task;
	variants: Variant[];
	cases: Case[];
	samples: number;
	concurrency: number;
	/** what the run is for, shown as its title in the viewer */
	description: string;
	tags: Record<string, string>;
}

/**
 * The upper end of a 95% Wilson interval, in percent, for `hits` out of promptfoo's `__count` rows.
 * A mathjs string: a JSON config cannot carry a function. `hits` must be a 0/1 score per row.
 */
export function wilsonUpperPct(hits: string): string {
	const p = `(${hits} / __count)`;
	return `100 * (${p} + 1.9208 / __count + 1.96 * sqrt(${p} * (1 - ${p}) / __count + 0.9604 / __count^2)) / (1 + 3.8416 / __count)`;
}

export function promptfooConfig(plan: RunPlan, entry: EntryFiles): Record<string, unknown> {
	return {
		description: plan.description,
		tags: plan.tags,
		// The provider builds its own messages from vars; this only names the row.
		prompts: [{ raw: '{{id}}', label: 'case' }],
		providers: plan.variants.map(variant => ({
			id: `file://${entry.provider}`,
			label: variant.label,
			config: {
				task: plan.task.name,
				strategy: variant.strategy,
				model: variant.model,
				samples: plan.samples,
			} satisfies VariantConfig,
		})),
		tests: plan.cases.map(c => ({
			description: c.description,
			vars: c.vars,
			metadata: { split: c.split, ...(c.group ? { group: c.group } : {}) },
		})),
		defaultTest: { assert: [{ type: 'javascript', value: `file://${entry.scorer}` }] },
		// Shown in the viewer's column header next to `ever_wrong`: with this many cases, the share
		// of cases a variant gets wrong could be this high.
		derivedMetrics: [{ name: 'ever_wrong_at_most_pct', value: wilsonUpperPct('ever_wrong') }],
		evaluateOptions: { maxConcurrency: plan.concurrency },
		extensions: [`file://${entry.hooks}:hook`],
	};
}
