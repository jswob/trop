import type { Subject } from '@jswob/design-workbench';
import { Plus, Trash2 } from 'lucide-react';
import { useState } from 'react';

import {
	Button,
	Figure,
	IconButton,
	Modal,
	PageHead,
	PageSection,
	PlaceholderRows,
	Select,
	Switch,
	TextButton,
	TextField,
	Waiting,
	useSelect,
} from '../kit';
import type { SelectOption } from '../kit';

/**
 * **The kit's stress page**: every primitive on one sheet at once, which is the only way to see
 * whether they still agree with each other.
 *
 * It belongs to the kit and not to `/proto`, which holds only what is unsettled, and not to
 * `/views`, because it is not a place anybody can be. It draws from `kit/` alone, on the app's
 * `--color-ui-*` tokens, with data made up for it here, so nothing on it is a record of the app.
 */

/** One row of the sheet's list: a name, a word about it, and a number that lines up. */
type Row = { id: string; name: string; note: string; count: number };

const ROWS: readonly Row[] = [
	{ id: 'r-groceries', name: 'Groceries', note: 'Bread, eggs, coffee', count: 3 },
	{ id: 'r-call-ada', name: 'Call Ada about the move', note: 'Before Friday', count: 1 },
	{ id: 'r-reading', name: 'Reading list', note: 'Four books, one started', count: 12 },
	{ id: 'r-garden', name: 'Garden', note: 'Water the beans', count: 0 },
];

const SORTS: SelectOption[] = [
	{ id: 'newest', label: 'Newest first' },
	{ id: 'oldest', label: 'Oldest first' },
	{ id: 'name', label: 'By name' },
	{ id: 'size', label: 'By size', blockedBy: 'Nothing is sized yet' },
];

const VIEWS = [
	{ value: 'list', word: 'List' },
	{ value: 'grid', word: 'Grid' },
] as const;

/** The list with every control a row of it can carry. */
function Rows() {
	const [armed, setArmed] = useState<string | null>(null);

	return (
		<ul className="w-full max-w-[32rem]">
			{ROWS.map(row => (
				<li
					key={row.id}
					className="flex items-center gap-4 border-b border-ui-rule py-2.5 last:border-b-0"
				>
					<div className="min-w-0 flex-1">
						<TextButton ink="ink" className="truncate">
							{row.name}
						</TextButton>
						<p className="truncate text-ui-body text-ui-ink-3">{row.note}</p>
					</div>
					<span className="shrink-0 text-ui-body text-ui-ink-2">
						<Figure>{row.count}</Figure>
					</span>
					{armed === row.id ? (
						<span className="shrink-0 text-ui-body text-ui-red">Delete this one?</span>
					) : null}
					<IconButton
						label={`Delete ${row.name}`}
						onClick={() => setArmed(was => (was === row.id ? null : row.id))}
					>
						<Trash2 size={16} aria-hidden />
					</IconButton>
				</li>
			))}
		</ul>
	);
}

/** The sheet itself; `asking` raises the one overlay over it. */
function StressSheet({ asking }: { asking: boolean }) {
	const [search, setSearch] = useState('');
	const [view, setView] = useState<(typeof VIEWS)[number]['value']>('list');
	const sort = useSelect({ options: SORTS, value: SORTS[0] });
	const [name, setName] = useState('');

	return (
		<div className="relative min-h-full overflow-auto px-6 py-8">
			<div className="flex flex-col gap-8">
				<PageHead
					head={{ title: 'Stress', lede: 'Every primitive of the kit, side by side, at rest.' }}
				/>

				<PageSection title="Find and order" aside={`${String(ROWS.length)} rows`}>
					<div className="flex flex-wrap items-end gap-3 pt-3">
						<TextField
							label="Search"
							value={search}
							placeholder="By name"
							width="w-64"
							onChange={setSearch}
						/>
						<Select label="Order" width="w-48" {...sort.props} />
						<Switch label="Show as" options={VIEWS} value={view} choose={setView} />
					</div>
				</PageSection>

				<PageSection title="Rows" aside="Names are presses">
					<Rows />
				</PageSection>

				<PageSection title="Buttons">
					<div className="flex flex-wrap items-start gap-3 pt-3">
						<Button weight="primary" icon={Plus}>
							Add a row
						</Button>
						<Button>Rename</Button>
						<Button weight="quiet">Cancel</Button>
						<Button weight="primary" running="Saving">
							Save changes
						</Button>
						<Button blocked="Nothing has changed">Save changes</Button>
					</div>
				</PageSection>

				<PageSection title="Nothing yet">
					<p className="pt-3 text-ui-body text-ui-ink-3">
						No rows yet. The first one you add appears here.
					</p>
				</PageSection>

				<PageSection title="Waiting">
					<Waiting at={0.4}>
						<PlaceholderRows rows={3} />
					</Waiting>
				</PageSection>
			</div>

			{asking ? (
				<Modal
					title="Rename Groceries"
					onClose={() => undefined}
					footer={
						<>
							<Button>Keep the name</Button>
							<Button weight="primary">Rename</Button>
						</>
					}
				>
					<TextField label="Name" value={name} error="A row needs a name." onChange={setName} />
				</Modal>
			) : null}
		</div>
	);
}

export const kitStress: Subject = {
	id: 'kit-stress',
	title: 'The kit’s stress page',
	description: 'Every kit primitive on one sheet, so they are made to agree with each other.',
	chrome: 'bare',
	variants: [
		{
			id: 'sheet',
			label: 'Sheet',
			note: 'Every primitive at rest, nothing over it.',
			render: () => <StressSheet asking={false} />,
		},
		{
			id: 'modal',
			label: 'Modal',
			note: 'The same sheet under the one overlay, a field refused.',
			render: () => <StressSheet asking />,
		},
	],
};
