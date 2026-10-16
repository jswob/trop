import { useState } from 'react';

import { Button, PageSection, PlaceholderRows, TextButton, TextField, Waiting } from '@app/ui/kit';
import { count } from '@app/ui/lib/plural';
import { FORM_MEASURE, KEEP_TASK, REMOVE_TASK } from '@app/ui/views/page-shared';

/**
 * **The example resource's page**: write a note, read the notes, delete one. Every layer the app's
 * own resources take has a file for the notes — the table, its fixture, the route, the web's hook
 * and route, this page and the workbench's place — and `docs/removing-notes.md` lists them, so the example
 * leaves in one commit.
 */

/** One note, as the page draws it. */
export type NoteRow = { id: string; title: string; body: string };

/** The longest title the API takes (`apps/api/src/modules/notes`). */
export const NOTE_TITLE_LIMIT = 120;

export const WRITE = 'Write a note';
export const TITLE = 'Title';
export const BODY = 'Body';
export const ADD = 'Add note';
export const ADDING = 'Adding';
export const NOTES = 'Notes';
export const NO_NOTES = 'No notes yet. The first one you add appears here.';
export const DELETE = 'Delete';

/** What a note's row says once its delete is armed, and the two presses that answer it. */
export const DELETE_LINE = 'Delete this note?';

export type NotesPageProps = {
	notes: readonly NoteRow[];
	/** The notes have not been read yet: rows at the real pitch, waiting. */
	loading?: boolean;
	/** Whether a note being added is still on its way. */
	adding?: boolean;
	onAdd: (note: { title: string; body: string }) => void;
	onDelete: (id: string) => void;
};

/** The page: the note being written above, every note below, the one changed last first. */
export function NotesPage({
	notes,
	loading = false,
	adding = false,
	onAdd,
	onDelete,
}: NotesPageProps) {
	const [title, setTitle] = useState('');
	const [body, setBody] = useState('');
	const [armed, setArmed] = useState<string | null>(null);
	const ready = title.trim().length > 0;

	const add = () => {
		if (!ready) return;
		onAdd({ title: title.trim(), body });
		setTitle('');
		setBody('');
	};

	return (
		<>
			<PageSection title={WRITE}>
				<form
					className={`${FORM_MEASURE} flex flex-col gap-3 pt-3`}
					onSubmit={event => {
						event.preventDefault();
						add();
					}}
				>
					<TextField label={TITLE} value={title} maxLength={NOTE_TITLE_LIMIT} onChange={setTitle} />
					<TextField label={BODY} value={body} onChange={setBody} />
					<div>
						<Button
							weight="primary"
							disabled={!ready}
							running={adding ? ADDING : null}
							onClick={add}
						>
							{ADD}
						</Button>
					</div>
				</form>
			</PageSection>
			<PageSection title={NOTES} aside={loading ? undefined : count(notes.length, 'note')}>
				{loading ? (
					<Waiting>
						<PlaceholderRows rows={3} />
					</Waiting>
				) : notes.length === 0 ? (
					<p className="pt-3 text-ui-body text-ui-ink-3">{NO_NOTES}</p>
				) : (
					<ul className={FORM_MEASURE}>
						{notes.map(note => (
							<li
								key={note.id}
								className="flex items-start gap-4 border-b border-ui-rule py-2.5 last:border-b-0"
							>
								<div className="min-w-0 flex-1">
									<p className="truncate text-ui-body text-ui-ink">{note.title}</p>
									{note.body === '' ? null : (
										<p className="truncate text-ui-body text-ui-ink-2">{note.body}</p>
									)}
								</div>
								{armed === note.id ? (
									<div className="flex shrink-0 items-center gap-3">
										<span className="text-ui-body text-ui-red">{DELETE_LINE}</span>
										<TextButton
											ink="ink"
											onClick={() => {
												setArmed(null);
												onDelete(note.id);
											}}
										>
											{REMOVE_TASK}
										</TextButton>
										<TextButton onClick={() => setArmed(null)}>{KEEP_TASK}</TextButton>
									</div>
								) : (
									<TextButton className="shrink-0" onClick={() => setArmed(note.id)}>
										{DELETE}
									</TextButton>
								)}
							</li>
						))}
					</ul>
				)}
			</PageSection>
		</>
	);
}
