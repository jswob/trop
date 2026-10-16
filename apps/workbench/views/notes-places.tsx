import { count } from '@app/ui/lib/plural';
import { NotesPage, type NoteRow } from '@app/ui/views/notes';
import { useSyncExternalStore } from 'react';

import { NOTES } from '../fixtures/notes';
import type { RailReading } from '../kit';
import type { Live, Place, ViewDraw } from './types';

/**
 * **Notes**, the example resource: write a note, read every note, delete one.
 *
 * The page's presses are the bench's own. A note added stands first and a note deleted goes, kept
 * in a store the variant opens with, so the page can be used rather than only looked at. The slug
 * follows the store, as the app's reading follows its records; a cell's fact is the count the
 * variant opened on.
 *
 * Arming a delete and answering it are presses, so neither has an entry. What has one is what no
 * press reaches: the notes there are, none at all, the read still out, and the read failed.
 */

/** The bench's notes for one frame: what the page lists, and what a press does to them. */
type NoteStore = Live<readonly NoteRow[]> & {
	add: (note: { title: string; body: string }) => void;
	remove: (id: string) => void;
};

function noteStore(initial: readonly NoteRow[]): NoteStore {
	let notes = initial;
	let written = 0;
	const listeners = new Set<() => void>();
	const set = (next: readonly NoteRow[]) => {
		notes = next;
		for (const listener of listeners) listener();
	};

	return {
		subscribe: listener => {
			listeners.add(listener);
			return () => {
				listeners.delete(listener);
			};
		},
		get: () => notes,
		add: note => {
			written += 1;
			set([{ id: `n-written-${String(written)}`, ...note }, ...notes]);
		},
		remove: id => {
			set(notes.filter(note => note.id !== id));
		},
	};
}

/** What the slug prints of a store's notes: the same object until they change. */
function readingOf(store: NoteStore): Live<RailReading> {
	let read: readonly NoteRow[] | null = null;
	let reading: RailReading = { place: 'Notes', fact: '' };

	return {
		subscribe: store.subscribe,
		get: () => {
			const notes = store.get();
			if (notes !== read) {
				read = notes;
				reading = { place: 'Notes', fact: count(notes.length, 'note') };
			}
			return reading;
		},
	};
}

/** The page on its store: a press changes the list it draws. */
function NotesPlace({ store }: { store: NoteStore }) {
	const notes = useSyncExternalStore(store.subscribe, store.get);

	return <NotesPage notes={notes} onAdd={store.add} onDelete={store.remove} />;
}

/** The notes read, and the page working on them. */
function draw(initial: readonly NoteRow[]): ViewDraw {
	const store = noteStore(initial);

	return {
		page: <NotesPlace store={store} />,
		reading: readingOf(store),
		facts: { notes: String(initial.length) },
	};
}

/** The read still out: rows at the real pitch, and nothing counted yet. */
function waiting(): ViewDraw {
	return {
		page: <NotesPage notes={[]} loading onAdd={() => undefined} onDelete={() => undefined} />,
		reading: { place: 'Notes', fact: '' },
		facts: { notes: '' },
	};
}

export const notes: Place = {
	id: 'notes',
	dest: 'notes',
	title: 'Notes',
	description: 'The example resource: write a note, read every note, delete one.',
	variants: [
		{
			id: 'notes',
			label: 'Three notes',
			note: 'Three notes, one with no body, the newest first.',
			draw: () => draw(NOTES),
		},
		{
			id: 'empty',
			label: 'No notes yet',
			note: 'Nothing written yet: the line that says so.',
			draw: () => draw([]),
		},
		{
			id: 'loading',
			label: 'Loading',
			note: 'Asked for and not arrived: placeholder rows.',
			draw: waiting,
		},
		{
			id: 'failed',
			label: 'Can’t reach the server',
			note: 'The notes’ read failed; the wait says so.',
			draw: () => ({ ...waiting(), readFailed: 'page' }),
		},
	],
};

export const NOTES_PLACES: Place[] = [notes];
