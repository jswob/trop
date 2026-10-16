import { NotesPage } from '@app/ui/views/notes';
import { createFileRoute } from '@tanstack/react-router';

import { useNotesPage } from './-components/use-notes-page';

/**
 * **Notes**, the example resource: write a note, read every note, delete one. `docs/removing-notes.md`
 * lists every file it takes, so an app removes it in one commit.
 */
export const Route = createFileRoute('/notes')({
	component: NotesRoute,
});

function NotesRoute() {
	const { notes, adding, add, remove } = useNotesPage();
	return (
		<NotesPage
			notes={notes ?? []}
			loading={notes === undefined}
			adding={adding}
			onAdd={add}
			onDelete={remove}
		/>
	);
}
