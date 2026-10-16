import { act, renderHook } from '@testing-library/react';
import { useState, type ReactNode } from 'react';
import { describe, expect, it, vi } from 'vitest';

import { DeclareProvider, createDeclareStore, type DeclareStore } from '@/frame/declare';
import { useCreateNote, useDeleteNote, useNotes } from '@/hooks/api';

import { useNotesPage } from './use-notes-page';

vi.mock('@/hooks/api', () => ({
	useNotes: vi.fn(),
	useCreateNote: vi.fn(),
	useDeleteNote: vi.fn(),
}));

const create = vi.fn();
const remove = vi.fn();

function given(notes: { id: string; title: string; body: string; updatedAt: Date }[] | undefined) {
	vi.mocked(useNotes).mockReturnValue({ data: notes } as ReturnType<typeof useNotes>);
	vi.mocked(useCreateNote).mockReturnValue({
		mutate: create,
		isPending: false,
	} as unknown as ReturnType<typeof useCreateNote>);
	vi.mocked(useDeleteNote).mockReturnValue({
		mutate: remove,
	} as unknown as ReturnType<typeof useDeleteNote>);
}

function rendered() {
	let store: DeclareStore | undefined;
	function Wrapper({ children }: { children: ReactNode }) {
		const [own] = useState(createDeclareStore);
		store = own;
		return <DeclareProvider store={own}>{children}</DeclareProvider>;
	}
	const hook = renderHook(() => useNotesPage(), { wrapper: Wrapper });
	return { ...hook, declared: () => store?.get() };
}

const SAVED = new Date('2026-01-02T10:00:00Z');

describe('useNotesPage', () => {
	it('hands the page each note’s id, title and body', () => {
		given([{ id: 'n-1', title: 'Groceries', body: 'Milk', updatedAt: SAVED }]);

		const { result } = rendered();

		expect(result.current.notes).toEqual([{ id: 'n-1', title: 'Groceries', body: 'Milk' }]);
	});

	it('has no notes to hand while they are being read', () => {
		given(undefined);

		const { result, declared } = rendered();

		expect(result.current.notes).toBeUndefined();
		expect(declared()).toEqual({
			dest: 'notes',
			declared: { reading: { place: 'Notes', fact: '' } },
		});
	});

	it('tells the frame how many notes there are', () => {
		given([
			{ id: 'n-1', title: 'Groceries', body: '', updatedAt: SAVED },
			{ id: 'n-2', title: 'Errands', body: '', updatedAt: SAVED },
		]);

		const { declared } = rendered();

		expect(declared()?.declared.reading).toEqual({ place: 'Notes', fact: '2 notes' });
	});

	it('adds and deletes through the API’s hooks', () => {
		given([]);
		const { result } = rendered();

		act(() => {
			result.current.add({ title: 'Errands', body: '' });
			result.current.remove('n-1');
		});

		expect(create).toHaveBeenCalledWith({ title: 'Errands', body: '' });
		expect(remove).toHaveBeenCalledWith('n-1');
	});
});
