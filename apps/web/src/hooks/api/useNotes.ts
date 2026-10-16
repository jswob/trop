import { queryOptions, useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import { client } from '@/lib/api';
import { ApiError, extractErrorMessage } from '@/lib/api-utils';
import { noteKeys } from './keys';

/** Every note, the one changed last first. */
export function noteListOptions() {
	return queryOptions({
		queryKey: noteKeys.list(),
		queryFn: async () => {
			const { data, error } = await client.api.notes.get();
			if (error)
				throw new ApiError(
					extractErrorMessage(error.value),
					typeof error.status === 'number' ? error.status : undefined
				);
			return data;
		},
	});
}

export function useNotes() {
	return useQuery(noteListOptions());
}

export type CreateNoteInput = { title: string; body?: string };

/** Adds a note and reads the list again. */
export function useCreateNote() {
	const queryClient = useQueryClient();

	return useMutation({
		mutationFn: async (input: CreateNoteInput) => {
			const { data, error } = await client.api.notes.post(input);
			if (error) throw new ApiError(extractErrorMessage(error.value), error.status);
			return data;
		},
		onSettled: () => queryClient.invalidateQueries({ queryKey: noteKeys.lists() }),
	});
}

/** Deletes a note by id and reads the list again. */
export function useDeleteNote() {
	const queryClient = useQueryClient();

	return useMutation({
		mutationFn: async (id: string) => {
			const { data, error } = await client.api.notes({ id }).delete();
			if (error) throw new ApiError(extractErrorMessage(error.value), error.status);
			return data;
		},
		onSettled: () => queryClient.invalidateQueries({ queryKey: noteKeys.lists() }),
	});
}
