import { specimen } from '@jswob/design-workbench';
import { Figure } from '@app/ui/kit';

/** Figures as a list prints them, with a short one to show the column holds. */
const COLUMN = ['5,000', '3,650', '1,200', '111'];

export default specimen(Figure, {
	kit: 'figure',
	is: 'Tabular figures, for numbers that have to line up down a column.',
	base: { children: '3,650' },
	states: {
		rest: {
			label: 'One figure',
			note: 'Geist Mono, at the size of the line it stands in.',
			args: {},
		},
		column: {
			label: 'Down a column',
			note: 'Four at the body size, right aligned: the digits line up.',
			draw: () => (
				<div className="flex flex-col items-end text-ui-body">
					{COLUMN.map(value => (
						<Figure key={value}>{value}</Figure>
					))}
				</div>
			),
		},
	},
	props: {
		children: { is: 'The number.', text: true },
	},
	parts: {
		figure: { select: 'span', is: 'The number, in the mono face with tabular figures.' },
	},
	holds: [
		'Geist Mono with tabular figures.',
		'A number lines up with the numbers above and below it.',
		'The monospace is here because the digits align in a column.',
	],
	refuses: [
		'Geist Mono as decoration on a small label.',
		'A second face inside one sentence. It reads as a second voice.',
		'A 13px mono clause number in a gutter. At body scale it is decoration.',
	],
});
