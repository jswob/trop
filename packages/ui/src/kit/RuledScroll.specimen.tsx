import { specimen } from '@jswob/design-workbench';
import { useLayoutEffect, useRef } from 'react';
import { RuledScroll } from '@app/ui/kit';

/** The box every state scrolls in: 240px of a column, 32px of ground before the rule. */
const BOX = 'h-60';
const INSET = 'pr-8';
const INSET_LEFT = 'pl-8';

const PARAGRAPHS = [
	'The market opens at eight and the stalls are full by nine.',
	'Walk the whole market once before buying anything: the bread at the far end, the vegetables by the gate, the cheese wherever the queue is shortest.',
	'Buy the heavy things last, so the bag is light for most of the walk, and nothing on the list twice unless it is on offer.',
	'The bakery keeps a loaf back until ten for anyone who asks the day before. Whoever asked carries the receipt.',
	'Before leaving, read the list once more, and pack the cold things together at the top of the bag.',
	'The car is parked by whoever drove, and the ticket goes back in the glovebox the same day.',
];

const prose = (count: number) => (
	<div className="space-y-3 text-ui-body leading-5 text-ui-ink-2">
		{PARAGRAPHS.slice(0, count).map(line => (
			<p key={line}>{line}</p>
		))}
	</div>
);

/** The column read to its foot, which is where the thumb ends up on the rule. */
function ReadToTheFoot() {
	const column = useRef<HTMLDivElement>(null);

	useLayoutEffect(() => {
		const scroller = column.current?.querySelector('.ui-scroll');
		if (scroller) scroller.scrollTop = scroller.scrollHeight;
	}, []);

	return (
		<div ref={column}>
			<RuledScroll className={BOX} scrollerClassName={INSET}>
				{prose(PARAGRAPHS.length)}
			</RuledScroll>
		</div>
	);
}

export default specimen(RuledScroll, {
	kit: 'ruled-scroll',
	is: 'A column that scrolls, whose scrollbar is the rule that ends its page.',
	width: 320,
	base: {
		className: BOX,
		scrollerClassName: INSET,
		edge: 'right',
		rule: 'right',
		children: prose(PARAGRAPHS.length),
	},
	states: {
		scrolls: {
			label: 'More than fits',
			note: 'The rule thickens to 2px over the share on screen.',
			args: {},
		},
		foot: {
			label: 'Read to the foot',
			note: 'The thumb sits as far down the rule as the share does.',
			draw: () => <ReadToTheFoot />,
		},
		fits: {
			label: 'All of it fits',
			note: 'Nothing to scroll: the hairline, and no thumb on it.',
			args: { children: prose(2) },
		},
		left: {
			label: 'On the left',
			note: 'A column whose neighbour is on its left.',
			args: { edge: 'left', rule: 'left', scrollerClassName: INSET_LEFT },
		},
		unruled: {
			label: 'An ancestor’s rule',
			note: 'No hairline of its own; the thumb still rides the edge.',
			args: { rule: 'none' },
		},
	},
	props: {
		className: { is: 'Classes for the column itself: its height, where a flex parent gives none.' },
		scrollerClassName: { is: 'Classes for the scrolling box. Padding belongs here.' },
		edge: {
			is: 'Which side the thumb rides. Defaults to right.',
			options: { left: true, right: true },
		},
		rule: {
			is: 'Which side the hairline is drawn on. None where an ancestor draws it.',
			options: { left: true, right: true, none: true },
		},
		children: { is: 'What scrolls.' },
	},
	parts: {
		column: { select: '.relative.flex', is: 'The box, and from 1024 the hairline on its edge.' },
		scroller: { select: '.ui-scroll', is: 'What scrolls, its own bar switched off from 1024.' },
		grip: { select: '.ui-scroll + span', is: 'The 11px a pointer takes the thumb by.' },
		thumb: { select: '.ui-scroll + span > span', is: '2px over the share on screen.' },
	},
	holds: [
		'The column’s own bar is off, and the hairline that ends it reports the scroll.',
		'The rule is 1px down the full height and 2px over the share on screen.',
		'The scroll costs the layout no width.',
		'The thumb’s length is the share on screen and its offset is how far down that share sits.',
		'Below 1024 there is no rule to carry a thumb, and the thin bar comes back.',
	],
	refuses: [
		'An unstyled native scrollbar. Any bar the design shows is drawn on the ink ramp.',
		'A scrollbar floating in the gap between two columns, which reads as a third column.',
		'A scrollbar on the outside of the glass, for a column beside the rail.',
		'A thumb shorter than 24px, which reads as a tick and not as a position.',
	],
});
