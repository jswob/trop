import type { NoteRow } from '@app/ui/views/notes';

/**
 * **The notes the bench draws**: a few, written the way a person writes them, so the page is judged
 * on words that read like notes rather than on filler. The one changed last stands first, as the
 * page lists them, and one has no body, because a body is optional.
 */
export const NOTES: readonly NoteRow[] = [
	{
		id: 'n-groceries',
		title: 'Groceries',
		body: 'Oats, lemons, coffee beans, and something for Sunday.',
	},
	{ id: 'n-call-ada', title: 'Call Ada about the move', body: '' },
	{
		id: 'n-reading',
		title: 'Reading list',
		body: 'The lighthouse one first, then the short stories Ada lent me.',
	},
];
