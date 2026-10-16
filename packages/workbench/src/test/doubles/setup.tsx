import type { ReactNode } from 'react';

import { defineSetup } from '../../lib/setup';
import type { Subject } from '../../lib/subject';

/**
 * A small app for the bench's own tests: two sections, three places, one candidate and a kit of
 * one. It stands where `virtual:workbench/setup` resolves to an app's setup file when served.
 */
const subject = (id: string, title: string): Subject => ({
	id,
	title,
	description: `${title}, as the double draws it.`,
	chrome: 'bare',
	variants: [
		{ id: 'resting', label: 'Resting', note: 'The state it opens in.', render: () => title },
	],
});

export default defineSetup({
	Wrap: ({ children }: { children: ReactNode }) => children,
	Ground: ({ children }: { children: ReactNode }) => <div data-ground="">{children}</div>,
	applyAppearance: appearance => {
		document.documentElement.dataset['theme'] = appearance;
	},
	sections: [
		{ id: 'ledger', label: 'Ledger' },
		{ id: 'people', label: 'People' },
	],
	groundLabel: 'Overview',
	places: [
		{ ...subject('overview', 'Overview'), dest: null },
		{ ...subject('ledger-list', 'Ledger'), dest: 'ledger' },
		{ ...subject('ledger-entry', 'Entry'), dest: 'ledger' },
	],
	protos: [subject('entry-split', 'Split entry')],
	kit: {
		entries: [
			{
				id: 'mark',
				name: 'Mark',
				files: ['kit/Mark.tsx'],
				symbols: ['Mark'],
				line: 'A mark beside a name.',
			},
		],
		specimens: [],
		stress: subject('kit-stress', 'Stress sheet'),
		subjects: [subject('kit-mark', 'Mark')],
	},
});
