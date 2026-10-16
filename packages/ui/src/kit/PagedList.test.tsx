import { cleanup, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import type { ListColumn, ListMark, ListWords } from '@app/ui/kit/paged-list';
import { PagedList, usePagedList } from './PagedList';
import { READ_FAILED, WaitFailed } from './Waiting';

type Note = { id: string; name: string; words: number; pinned: boolean };

const NOTES: Note[] = [
	{ id: 'a', name: 'Errands', words: 720, pinned: false },
	{ id: 'b', name: 'Garden', words: 365, pinned: true },
	{ id: 'c', name: 'Groceries', words: 2300, pinned: true },
];

const COLUMNS: readonly ListColumn<Note>[] = [
	{
		id: 'words',
		head: 'Words',
		figure: row => String(row.words),
		value: row => row.words,
		narrow: true,
	},
];

const WORDS: ListWords = {
	name: 'Name',
	find: 'Find by name',
	clear: 'Clear search',
	previous: 'Previous page',
	next: 'Next page',
	empty: 'No notes.',
	count: total => `${String(total)} notes`,
	match: (found, total) => `${String(found)} of ${String(total)} match`,
	noMatch: query => `No note matches ${query}.`,
	range: (first, last) => `${String(first)}–${String(last)}`,
	pageName: (page, pages) => `Page ${String(page)} of ${String(pages)}`,
};

const PINNED: ListMark<Note> = { rows: row => row.pinned, word: 'pinned' };

/** The list as a caller draws it: the hand spread straight in. */
function Specimen({ mark }: { mark?: ListMark<Note> }) {
	const hand = usePagedList(NOTES, COLUMNS);
	return (
		<PagedList
			title="Notes"
			words={WORDS}
			columns={COLUMNS}
			href={row => `#${row.id}`}
			mark={mark}
			{...hand}
		/>
	);
}

const headerOf = (name: string) =>
	screen
		.getAllByRole('rowheader')
		.find(header => within(header).queryByRole('link', { name }) !== null);

afterEach(cleanup);

describe('PagedList — marked rows', () => {
	it('says a marked row is marked, in the row header a reader hears', () => {
		render(<Specimen mark={PINNED} />);

		expect(headerOf('Groceries')?.textContent).toBe('Groceries, pinned');
		expect(headerOf('Garden')?.textContent).toBe('Garden, pinned');
		expect(headerOf('Errands')?.textContent).toBe('Errands');
	});

	it('keeps the name the whole of the link, so the mark never renames where it goes', () => {
		render(<Specimen mark={PINNED} />);
		expect(screen.getByRole('link', { name: 'Groceries' })).toBeTruthy();
	});

	it('marks nothing where no mark is passed', () => {
		render(<Specimen />);
		expect(screen.queryByText('pinned')).toBeNull();
		expect(screen.getAllByRole('rowheader').map(header => header.textContent)).toEqual([
			'Errands',
			'Garden',
			'Groceries',
		]);
	});
});

describe('PagedList — the sort a head cycles through', () => {
	const names = () => screen.getAllByRole('link').map(link => link.textContent);
	const sorted = () =>
		screen
			.getAllByRole('columnheader')
			.filter(head => head.hasAttribute('aria-sort'))
			.map(head => `${head.textContent ?? ''} ${head.getAttribute('aria-sort') ?? ''}`);
	const press = (head: string) => {
		fireEvent.click(screen.getByRole('button', { name: head }));
	};

	it('goes low to high, high to low, then off — the records back as they were handed', () => {
		render(<Specimen />);

		press('Words');
		expect(sorted()).toEqual(['Words ascending']);
		expect(names()).toEqual(['Garden', 'Errands', 'Groceries']);

		press('Words');
		expect(sorted()).toEqual(['Words descending']);
		expect(names()).toEqual(['Groceries', 'Errands', 'Garden']);

		press('Words');
		expect(sorted()).toEqual([]);
		expect(names()).toEqual(['Errands', 'Garden', 'Groceries']);
		expect(screen.getAllByRole('columnheader').some(head => head.querySelector('svg'))).toBe(false);
	});

	it('takes the name off the same way', () => {
		render(<Specimen />);
		press('Name');
		expect(sorted()).toEqual(['Name descending']);
		press('Name');
		expect(sorted()).toEqual([]);
	});
});

describe('PagedList — names with nowhere to go', () => {
	function Still() {
		const hand = usePagedList(NOTES, COLUMNS);
		return <PagedList title="Notes" words={WORDS} columns={COLUMNS} {...hand} />;
	}

	it('draws every name still, as text, when no href is passed', () => {
		render(<Still />);

		expect(screen.queryAllByRole('link')).toHaveLength(0);
		expect(screen.queryByRole('button', { name: 'Errands' })).toBeNull();
		expect(screen.getByText('Errands')).toBeDefined();
	});
});

/** A failed list says so at its head, once, and not in the pager under its rows. */
describe('PagedList — a wait that failed', () => {
	function Waiting({ failed }: { failed: boolean }) {
		const hand = usePagedList(NOTES, COLUMNS);
		return (
			<WaitFailed onRetry={failed ? () => undefined : null}>
				<PagedList title="Notes" words={WORDS} columns={COLUMNS} {...hand} loading />
			</WaitFailed>
		);
	}

	const notes = () =>
		screen.queryAllByRole('status').filter(node => node.textContent.includes(READ_FAILED));

	it('says it once, before the placeholder rows', () => {
		render(<Waiting failed />);
		const [note] = notes();
		expect(notes()).toHaveLength(1);
		const firstRow = document.querySelector('[data-ui-glaze-bar]');
		expect(note?.compareDocumentPosition(firstRow as Node) ?? 0).toBe(
			Node.DOCUMENT_POSITION_FOLLOWING
		);
	});

	it('keeps its loading note in the pager while nothing has failed', () => {
		render(<Waiting failed={false} />);
		expect(notes()).toHaveLength(0);
		expect(screen.getAllByRole('status').some(node => node.textContent.includes('loading'))).toBe(
			true
		);
	});
});
