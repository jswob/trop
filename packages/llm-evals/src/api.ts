// The harness's public face: everything a task and the app's `evals.config.ts` may import. The
// package exports nothing else, so the harness can change inside without touching a task. Test
// helpers are in testing.ts.
export { defineConfig, type EvalsConfig } from './config.ts';
export {
	defineTask,
	type Case,
	type ModelSetting,
	type Outcome,
	type Step,
	type Strategy,
	type StrategyRun,
	type Task,
	type TaskRegistry,
	type Verdict,
} from './core/task.ts';
export { cachedCall, type CallApiContext } from './core/promptfoo.ts';
export { allVariants, type Variant } from './core/grid.ts';
export type {
	KeyChange,
	KeyReview,
	Pick,
	QueueItem,
	ReviewItem,
	ReviewOption,
} from './core/key.ts';
export {
	chatJson,
	chatJsonBody,
	type ChatJsonRequest,
	type ChatJsonResult,
	type ChatMessage,
	type Effort,
	type JsonSchema,
} from './adapters/openrouter/client.ts';
