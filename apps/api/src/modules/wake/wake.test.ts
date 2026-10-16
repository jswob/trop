import type { DatabaseConnection } from '../../db/types';
import { countQueries } from '../../test/db';
import { describe, expect, test } from '../../test/test-context';
import { createWakeRoute } from './index';

type App = ReturnType<typeof createWakeRoute>;

const wake = (app: App) => app.handle(new Request('http://localhost/api/wake'));

describe('GET /api/wake', () => {
	test('answers ok, having asked the database once', async ({ tx }) => {
		const app = createWakeRoute(tx);
		let response: Response | undefined;

		const queries = await countQueries(async () => {
			response = await wake(app);
		});

		expect(response?.status).toBe(200);
		expect(await response?.json()).toEqual({ status: 'ok' });
		expect(queries).toBe(1);
	});

	test('asked again inside the interval, leaves the database alone', async ({ tx }) => {
		const app = createWakeRoute(tx, { everyMs: 60_000 });
		await wake(app);

		const queries = await countQueries(() => wake(app));

		expect(queries).toBe(0);
	});

	test('asked again after the interval, wakes the database again', async ({ tx }) => {
		const app = createWakeRoute(tx, { everyMs: 20 });
		await wake(app);
		await new Promise(resolve => setTimeout(resolve, 40));

		const queries = await countQueries(() => wake(app));

		expect(queries).toBe(1);
	});

	test('answers ok when the database does not', async () => {
		const down = {
			execute: () => Promise.reject(new Error('connect ECONNREFUSED')),
		} as unknown as DatabaseConnection;

		const response = await wake(createWakeRoute(down));

		expect(response.status).toBe(200);
	});
});
