// The variant grid: every strategy of a task can run on every one of its model settings. A run
// picks cells from the grid; each picked cell becomes one column. No column is written by hand.
import type { ModelSetting, Task } from './task.ts';

export interface Variant {
	/** `strategy/model` */
	id: string;
	strategy: string;
	model: string;
	/** the column's title in the viewer */
	label: string;
	champion: boolean;
}

/** The model setting of a variant id (`strategy/model`). */
export const modelOf = (variantId: string): string => variantId.split('/')[1] ?? '';

export function variantOf(task: Task, strategy: string, model: string): Variant {
	if (!(strategy in task.strategies)) {
		throw new Error(
			`Unknown strategy "${strategy}". Strategies: ${Object.keys(task.strategies).join(', ')}`
		);
	}
	if (!(model in task.models)) {
		throw new Error(
			`Unknown model setting "${model}". Settings: ${Object.keys(task.models).join(', ')}`
		);
	}
	const id = `${strategy}/${model}`;
	const champion = id === task.champion;
	return {
		id,
		strategy,
		model,
		champion,
		label: `${champion ? 'champion · ' : ''}${strategy} · ${model}`,
	};
}

function parseId(task: Task, id: string): Variant {
	const [strategy, model, ...rest] = id.split('/');
	if (!strategy || !model || rest.length) {
		throw new Error(`A variant is written strategy/model, e.g. ${task.champion}. Got "${id}".`);
	}
	return variantOf(task, strategy, model);
}

export function allVariants(task: Task): Variant[] {
	return Object.keys(task.strategies).flatMap(strategy =>
		Object.keys(task.models).map(model => variantOf(task, strategy, model))
	);
}

export interface Selection {
	/** `a/b,c/d`, `all` or `champion` */
	variants?: string;
	/** comma lists; the cross product is run, a missing side comes from the champion */
	strategy?: string;
	model?: string;
}

const list = (value: string) =>
	value
		.split(',')
		.map(s => s.trim())
		.filter(Boolean);

/** The columns of a run. With nothing chosen: the champion alone. The champion always comes first. */
export function selectVariants(task: Task, selection: Selection): Variant[] {
	const champion = parseId(task, task.champion);
	let picked: Variant[];
	if (selection.variants === 'all') picked = allVariants(task);
	else if (selection.variants && selection.variants !== 'champion') {
		picked = list(selection.variants).map(id => parseId(task, id));
	} else if (selection.strategy || selection.model) {
		const strategies = selection.strategy ? list(selection.strategy) : [champion.strategy];
		const models = selection.model ? list(selection.model) : [champion.model];
		picked = strategies.flatMap(s => models.map(m => variantOf(task, s, m)));
	} else picked = [champion];
	const unique = [...new Map(picked.map(v => [v.id, v])).values()];
	return [...unique.filter(v => v.champion), ...unique.filter(v => !v.champion)];
}

const SAFETY = 0.75;
const MAX_CONCURRENCY = 8;

/** Calls a model setting can take per minute, by whichever of its two limits is tighter. */
function callsPerMinute(model: ModelSetting): number {
	return Math.min(model.requestsPerMinute, model.tokensPerMinute / model.tokensPerCall);
}

/**
 * How many calls may be in flight so the run stays under the account's limits instead of bouncing
 * off them. The tightest model setting in the run decides; promptfoo's scheduler still backs off
 * if a limit is hit anyway.
 */
export function concurrencyFor(task: Task, variants: Variant[]): number {
	const perSetting = variants.map(v => {
		const model = task.models[v.model];
		if (!model) return MAX_CONCURRENCY;
		return Math.floor((SAFETY * callsPerMinute(model) * model.secondsPerCall) / 60);
	});
	return Math.max(1, Math.min(MAX_CONCURRENCY, ...perSetting));
}
