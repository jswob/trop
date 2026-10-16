import { describe, expect, test } from 'vitest';

import { doctor, renderFindings } from './doctor.ts';
import { estimate, renderEstimate } from './estimate.ts';
import { fakeTask } from './fake-task.ts';
import { selectVariants } from './grid.ts';
import { defineTask, type Task } from './task.ts';

const key = (limitRemaining: number | null) => () =>
	Promise.resolve({ label: 'k', limitRemaining, isFreeTier: false });

describe('doctor', () => {
	test('passes when the key works and the model answers the canary', async () => {
		const findings = await doctor(fakeTask(), ['fast'], { keyStatus: key(5) }, 1);
		expect(findings.map(f => f.ok)).toEqual([true, true]);
		expect(renderFindings(findings)).toContain('✓ model fast: answered "GAMMA"');
	});

	test('a rejected key is reported once, without trying the models', async () => {
		const rejected = {
			kind: 'fatal',
			status: 401,
			retryAfterSeconds: null,
			message: 'OpenRouter 401',
		};
		const findings = await doctor(fakeTask(), ['fast', 'slow'], {
			// eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors -- Reason: the client rejects with a plain failure object, as this one
			keyStatus: () => Promise.reject(rejected),
		});
		expect(findings).toHaveLength(1);
		expect(findings[0]).toMatchObject({ ok: false, what: 'key' });
	});

	test('too little credit for the run is a failed check with a fix', async () => {
		const [finding] = await doctor(fakeTask(), [], { keyStatus: key(0.2) }, 0.5);
		expect(finding).toMatchObject({
			ok: false,
			what: 'credit',
			fix: expect.stringContaining('--max-cost') as string,
		});
	});

	test('an answer from the shared pool fails a model that expects the user’s own key', async () => {
		const base = fakeTask();
		const task: Task = defineTask({
			...base,
			models: { fast: { ...base.models.fast!, expectByok: true } },
			strategies: {
				upper: {
					calls: 1,
					run: async (input, model, context) => {
						const run = await base.strategies.upper!.run(input, model, context);
						return { ...run, steps: run.steps.map(s => ({ ...s, byok: false })) };
					},
				},
			},
		});
		const [finding] = await doctor(task, ['fast'], {});
		expect(finding).toMatchObject({
			ok: false,
			detail: expect.stringContaining('shared pool') as string,
		});
	});

	test('a model call that fails names what kind of failure it was', async () => {
		const overloaded = {
			kind: 'transient',
			status: 503,
			retryAfterSeconds: null,
			message: 'OpenRouter 503',
		};
		const task = fakeTask({
			answer() {
				// eslint-disable-next-line @typescript-eslint/only-throw-error -- Reason: a classified failure is recognised by its shape, as an adapter's error would be
				throw overloaded;
			},
		});
		const [finding] = await doctor(task, ['fast'], {});
		expect(finding).toMatchObject({
			ok: false,
			fix: expect.stringContaining('overloaded') as string,
		});
	});
});

describe('estimate', () => {
	test('counts every call as if nothing were cached', () => {
		const task = fakeTask();
		const e = estimate(task, selectVariants(task, { model: 'fast,slow' }), 65, 3, 2);
		expect(e).toMatchObject({ calls: 390, perCallMax: 0.001 });
		expect(e.atMost).toBeCloseTo(0.39);
		expect(e.seconds).toBe(195);
		expect(renderEstimate(e)).toBe(
			'390 model calls at most, about $0.39 and 4 min if nothing is cached'
		);
	});
});
