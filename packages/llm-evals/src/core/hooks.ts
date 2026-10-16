// The promptfoo extension hook of every run. `beforeAll` is the health check: when it throws,
// promptfoo stops before any model call.
import { doctor, renderFindings, type Checks } from './doctor.ts';
import { taskNamed, type TaskRegistry } from './task.ts';

export function createHook(tasks: TaskRegistry, checks: Checks) {
	return async function hook(name: string, context: unknown): Promise<unknown> {
		if (name === 'beforeAll' && process.env.EVALS_PREFLIGHT === '1') {
			const task = taskNamed(tasks, process.env.EVALS_TASK);
			const models = (process.env.EVALS_MODELS ?? '').split(',').filter(Boolean);
			const findings = await doctor(task, models, checks, Number(process.env.EVALS_MAX_COST) || 0);
			if (findings.some(f => !f.ok)) {
				throw new Error(`health check failed, no paid call was made:\n${renderFindings(findings)}`);
			}
		}
		return context;
	};
}
