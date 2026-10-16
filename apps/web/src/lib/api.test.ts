import { afterEach, describe, expect, it, vi } from 'vitest';

import { wakeServer } from './api';

afterEach(() => {
	vi.unstubAllGlobals();
});

/** The address the first request went to. */
function asked(fetcher: ReturnType<typeof vi.fn>): string {
	const [target] = fetcher.mock.calls[0] as [RequestInfo | URL];
	return target instanceof Request ? target.url : String(target);
}

describe('wakeServer', () => {
	it('asks the server’s wake', async () => {
		const fetcher = vi.fn().mockResolvedValue(Response.json({ status: 'ok' }));
		vi.stubGlobal('fetch', fetcher);

		wakeServer();

		await vi.waitFor(() => expect(fetcher).toHaveBeenCalledTimes(1));
		expect(new URL(asked(fetcher)).pathname).toBe('/api/wake');
	});

	it('leaves a server it cannot reach alone', async () => {
		const unhandled = vi.fn();
		process.on('unhandledRejection', unhandled);
		const fetcher = vi.fn().mockRejectedValue(new TypeError('Failed to fetch'));
		vi.stubGlobal('fetch', fetcher);

		expect(() => wakeServer()).not.toThrow();

		await vi.waitFor(() => expect(fetcher).toHaveBeenCalledTimes(1));
		await new Promise(resolve => setTimeout(resolve, 20));
		process.off('unhandledRejection', unhandled);
		expect(unhandled).not.toHaveBeenCalled();
	});
});
