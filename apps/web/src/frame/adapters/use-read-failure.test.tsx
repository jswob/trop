import { QueryClient, QueryClientProvider, useQuery } from '@tanstack/react-query';
import { act, renderHook, waitFor } from '@testing-library/react';
import type { ReactNode } from 'react';
import { describe, expect, it, vi } from 'vitest';

import { ApiError } from '@/lib/api-utils';

import { useReadFailure } from './use-read-failure';

const GROUND = [['notes', 'list']] as const;

function setup() {
	const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
	const wrapper = ({ children }: { children: ReactNode }) => (
		<QueryClientProvider client={client}>{children}</QueryClientProvider>
	);
	return { client, wrapper };
}

/** A page reading one query, and the frame reading whether any read failed. */
function useReading(key: readonly unknown[], fetch: () => Promise<unknown>) {
	useQuery({ queryKey: key, queryFn: fetch });
	return useReadFailure(GROUND);
}

describe('useReadFailure', () => {
	it('is null while every read is fine', async () => {
		const { wrapper } = setup();
		const { result } = renderHook(() => useReading(['notes', 'list'], () => Promise.resolve([])), {
			wrapper,
		});
		await waitFor(() => expect(result.current).toBeNull());
	});

	it('reports a ground read that failed with a server error as the ground’s', async () => {
		const { wrapper } = setup();
		const { result } = renderHook(
			() =>
				useReading(['notes', 'list', undefined], () => Promise.reject(new ApiError('down', 503))),
			{ wrapper }
		);
		await waitFor(() => expect(result.current?.ground).toBe(true));
	});

	it('reports a network failure, which carries no status, as a failed read', async () => {
		const { wrapper } = setup();
		const { result } = renderHook(
			() => useReading(['people', 'list'], () => Promise.reject(new TypeError('Failed to fetch'))),
			{ wrapper }
		);
		await waitFor(() => expect(result.current).not.toBeNull());
		expect(result.current?.ground).toBe(false);
	});

	it.for([400, 401, 404, 409])('never counts a %s', async status => {
		const { wrapper } = setup();
		const { result } = renderHook(
			() => useReading(['notes', 'list'], () => Promise.reject(new ApiError('no', status))),
			{ wrapper }
		);
		await new Promise(resolve => setTimeout(resolve, 20));
		expect(result.current).toBeNull();
	});

	it('never counts a failed query nothing on screen is reading', async () => {
		const { client, wrapper } = setup();
		await client
			.fetchQuery({
				queryKey: ['notes', 'list'],
				queryFn: () => Promise.reject(new ApiError('down', 500)),
			})
			.catch(() => undefined);
		const { result } = renderHook(() => useReadFailure(GROUND), { wrapper });
		expect(result.current).toBeNull();
	});

	it('asks again for the failed reads only, and clears once they answer', async () => {
		const { wrapper } = setup();
		let down = true;
		const failing = vi.fn(() =>
			down ? Promise.reject(new ApiError('down', 502)) : Promise.resolve([])
		);
		const fine = vi.fn(() => Promise.resolve([]));
		const { result } = renderHook(
			() => {
				useQuery({ queryKey: ['roles', 'list'], queryFn: fine });
				return useReading(['notes', 'list'], failing);
			},
			{ wrapper }
		);
		await waitFor(() => expect(result.current?.ground).toBe(true));
		const fineCalls = fine.mock.calls.length;

		down = false;
		act(() => result.current?.retry());
		await waitFor(() => expect(result.current).toBeNull());
		expect(fine.mock.calls.length).toBe(fineCalls);
	});
});
