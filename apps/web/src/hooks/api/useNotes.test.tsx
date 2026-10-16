import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, renderHook, waitFor } from '@testing-library/react';
import type { ReactNode } from 'react';
import { describe, expect, it, vi } from 'vitest';

const { get, post, remove, byId } = vi.hoisted(() => {
	const remove = vi.fn(() => Promise.resolve({ data: { id: 'n-1' }, error: null }));
	return {
		get: vi.fn(() => Promise.resolve({ data: [{ id: 'n-1', title: 'Groceries' }], error: null })),
		post: vi.fn((_body: unknown) => Promise.resolve({ data: { id: 'n-2' }, error: null })),
		remove,
		byId: vi.fn((_params: unknown) => ({ delete: remove })),
	};
});

vi.mock('@/lib/api', () => ({
	client: { api: { notes: Object.assign(byId, { get, post }) } },
}));

import { ApiError } from '@/lib/api-utils';
import { noteKeys } from './keys';
import { noteListOptions, useCreateNote, useDeleteNote } from './useNotes';

function wrapperWith(client: QueryClient) {
	return ({ children }: { children: ReactNode }) => (
		<QueryClientProvider client={client}>{children}</QueryClientProvider>
	);
}

describe('noteListOptions', () => {
	it('reads every note on the list key', async () => {
		const options = noteListOptions();

		expect(options.queryKey).toEqual(noteKeys.list());
		await expect(new QueryClient().fetchQuery(options)).resolves.toEqual([
			{ id: 'n-1', title: 'Groceries' },
		]);
	});

	it('throws an ApiError carrying the status when the read fails', async () => {
		get.mockResolvedValueOnce({
			data: null,
			error: { status: 500, value: { message: 'Down' } },
		} as never);

		const failed = new QueryClient({ defaultOptions: { queries: { retry: false } } }).fetchQuery(
			noteListOptions()
		);

		await expect(failed).rejects.toBeInstanceOf(ApiError);
		await expect(failed).rejects.toMatchObject({ message: 'Down', status: 500 });
	});
});

describe('note writes', () => {
	it('adds a note and reads the list again', async () => {
		const client = new QueryClient();
		const invalidate = vi.spyOn(client, 'invalidateQueries');
		const { result } = renderHook(() => useCreateNote(), { wrapper: wrapperWith(client) });

		await act(() => result.current.mutateAsync({ title: 'Errands' }));

		expect(post).toHaveBeenCalledWith({ title: 'Errands' });
		await waitFor(() => {
			expect(invalidate).toHaveBeenCalledWith({ queryKey: noteKeys.lists() });
		});
	});

	it('deletes a note by id and reads the list again', async () => {
		const client = new QueryClient();
		const invalidate = vi.spyOn(client, 'invalidateQueries');
		const { result } = renderHook(() => useDeleteNote(), { wrapper: wrapperWith(client) });

		await act(() => result.current.mutateAsync('n-1'));

		expect(byId).toHaveBeenCalledWith({ id: 'n-1' });
		expect(remove).toHaveBeenCalled();
		await waitFor(() => {
			expect(invalidate).toHaveBeenCalledWith({ queryKey: noteKeys.lists() });
		});
	});
});
