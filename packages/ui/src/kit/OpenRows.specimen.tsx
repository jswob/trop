import { specimen } from '@jswob/design-workbench';
import { Minus } from 'lucide-react';
import { ActRow, AddRow, CONTROL_H, ICON_BOX, OpenBlock, RowGap } from '@app/ui/kit';

const press = () => undefined;

/** A project's reminders: two copies and one written here. */
const COPIES = [
	{ id: 'a', words: 'Remind me a day before each errand', off: false },
	{ id: 'b', words: 'Mark a note done once every item is ticked', off: true },
];

const WRITTEN = { id: 'c', words: 'Keep groceries at the top of the list' };

const ADD = 'Write a reminder';

/** The copies: one in force, and unless `on` one switched off, struck in the second ink. */
function copies(on = false) {
	return COPIES.map(copy => {
		const off = copy.off && !on;
		const verb = off ? 'Switch on' : 'Switch off';

		return (
			<ActRow key={copy.id} verb={verb} label={`${verb}: ${copy.words}`} onPress={press}>
				<span className={off ? 'text-ui-ink-2 line-through' : 'text-ui-ink'}>{copy.words}</span>
			</ActRow>
		);
	});
}

const oneGroup = (
	<>
		{copies()}
		<AddRow word={ADD} onPress={press} />
	</>
);

const twoGroups = (
	<>
		{copies()}
		<RowGap />
		<ActRow verb="Rewrite" label={`Rewrite: ${WRITTEN.words}`} onPress={press}>
			<span className="text-ui-ink">{WRITTEN.words}</span>
		</ActRow>
		<AddRow word={ADD} onPress={press} />
	</>
);

/** The block under the row that opened it: the marker column, then the block at the name's x. */
function Threaded() {
	return (
		<div className="flex gap-2">
			<span aria-hidden className={`${ICON_BOX} text-ui-ink-2`}>
				<Minus size={16} />
			</span>
			<div className="min-w-0 flex-1">
				<p className={`flex ${CONTROL_H} items-center text-ui-lead text-ui-ink`}>Errands</p>
				<OpenBlock id="threaded" threaded>
					{copies(true)}
					<AddRow word={ADD} onPress={press} />
				</OpenBlock>
			</div>
		</div>
	);
}

export default specimen(OpenBlock, {
	kit: 'open-rows',
	is: 'What a row of a list opens: more rows in the list’s own grid, each ending in its one press.',
	width: 512,
	base: { id: 'reminders' },
	states: {
		groups: {
			label: 'Two groups',
			note: 'Told apart by a 12px gap; one copy is switched off.',
			args: { children: twoGroups },
		},
		one: {
			label: 'One group',
			note: 'Rows on their hairlines, and the row that adds one.',
			args: { children: oneGroup },
		},
		threaded: {
			label: 'Under its row',
			note: 'At the name’s x, the thread tying it to its row.',
			draw: () => <Threaded />,
		},
	},
	props: {
		id: { is: 'What the press that opened it points aria-controls at.' },
		threaded: {
			is: 'Draws the thread beside it, in the marker column of the row that opened it.',
			toggle: true,
		},
		children: { is: 'Its rows: ActRow, RowGap between two groups, and AddRow last.' },
	},
	parts: {
		block: { select: '.pt-2', is: 'The open block: 8px over its first row, 4px under its last.' },
		row: { select: '.grid', is: 'A row: the control floor at least, a hairline over it.' },
		words: { select: '.grid > div', is: 'The words, typeset. Never the press.' },
		verb: { select: '.grid > button', is: 'The one word at the row’s end, which is the press.' },
		gap: { select: '.h-3', is: 'The 12px between two groups, closed by a hairline.' },
		add: { select: '.border-b > div:last-child', is: 'The last row: a plus and what it adds.' },
	},
	holds: [
		'A row is the control floor at least, with a hairline over it; the block draws one under its last.',
		'The words are never the press. The one word at the row’s end is, and it says what it does.',
		'The accessible name says the verb and the words together.',
		'Two groups are told apart by a 12px gap, with no heading.',
		'The block stands at the name’s x at every width, and the thread says which row opened it.',
	],
	refuses: [
		'A centred group heading with a glyph, or a 2px bar beside each rule.',
		'The open item set on its own plate, or two columns inside the open row.',
		'A filled square or a pencil at a row’s end.',
		'Stepping out of the indent below 768. It is one layout at every width.',
		'Periods as rows. They are a sentence: PeriodSentence.',
		'A second mark that says whose block it is: a ground under the row, or its words repeated.',
	],
});
