import { expect, test } from 'vitest';
import { createQueryCounter } from './count-queries';

test('counts only what is sent while the run is pending', async () => {
	const { debug, countQueries } = createQueryCounter();
	debug();

	const queries = await countQueries(async () => {
		debug();
		await Promise.resolve();
		debug();
	});

	expect(queries).toBe(2);
});

test('two counters keep their own counts', async () => {
	const first = createQueryCounter();
	const second = createQueryCounter();

	expect(await first.countQueries(() => Promise.resolve(second.debug()))).toBe(0);
});
