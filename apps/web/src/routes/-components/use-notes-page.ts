import { count } from '@app/ui/lib/plural';
import type { NoteRow } from '@app/ui/views/notes';
import { useMemo } from 'react';

import { useDeclare } from '@/frame/declare';
import { useCreateNote, useDeleteNote, useNotes } from '@/hooks/api';

/** The notes page's data and presses, and what it tells the frame: how many notes there are. */
export function useNotesPage() {
	const notes = useNotes();
	const create = useCreateNote();
	const remove = useDeleteNote();

	const rows = useMemo<NoteRow[] | undefined>(
		() => notes.data?.map(({ id, title, body }) => ({ id, title, body })),
		[notes.data]
	);
	const reading = useMemo(
		() => ({ place: 'Notes', fact: rows === undefined ? '' : count(rows.length, 'note') }),
		[rows]
	);
	useDeclare('notes', { reading });

	return {
		notes: rows,
		adding: create.isPending,
		add: (note: { title: string; body: string }) => create.mutate(note),
		remove: (id: string) => remove.mutate(id),
	};
}
