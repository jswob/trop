import { WaitFailed } from '@app/ui/kit/Waiting';
import { specimen } from '@jswob/design-workbench';
import { Placeholder, PlaceholderRows, Waiting, WaitingNote } from '@app/ui/kit';

/** The generic wait: the note right-aligned over a run of footprints at the row pitch. */
const rows = (
	<>
		<div className="flex justify-end pb-1">
			<WaitingNote />
		</div>
		<PlaceholderRows rows={8} />
	</>
);

/** A wait that knows its own anatomy: a head, a lead and three lines, out of `Placeholder`. */
const ownShape = (
	<>
		<div className="flex justify-end pb-1">
			<WaitingNote />
		</div>
		<div className="space-y-3">
			<Placeholder width={144} height={20} />
			<Placeholder width="100%" height={96} />
			<Placeholder width="88%" />
			<Placeholder width="94%" />
			<Placeholder width="61%" />
		</div>
	</>
);

/** A wait whose read failed. The specimen has nothing to ask again for, so the press does nothing. */
const failed = (
	<WaitFailed onRetry={() => undefined}>
		<Waiting>{rows}</Waiting>
	</WaitFailed>
);

export default specimen(Waiting, {
	kit: 'waiting',
	is: 'What a page draws while it waits: footprints under one glaze, and a note counting seconds.',
	width: 304,
	base: { children: rows },
	states: {
		start: {
			label: 'Pinned at 0',
			note: 'The field held at the start of its pass, for a still.',
			args: { at: 0 },
		},
		quarter: {
			label: 'Pinned at 0.25',
			note: 'A quarter of a tile on. Every band has moved down.',
			args: { at: 0.25 },
		},
		half: {
			label: 'Pinned at 0.5',
			note: 'Half a tile on. Some band is always on the page.',
			args: { at: 0.5 },
		},
		late: {
			label: 'Pinned at 0.75',
			note: 'The last quarter hands back to the first: a loop.',
			args: { at: 0.75 },
		},
		running: {
			label: 'Running',
			note: 'Nothing pinned: the loader as every caller draws it.',
			args: {},
		},
		shape: {
			label: 'The caller’s shape',
			note: 'A tall footprint is graduated through its own height.',
			args: { at: 0.25, children: ownShape },
		},
		failed: {
			label: 'Failed',
			note: 'The glaze stops, the seconds go, and the note says so.',
			draw: () => failed,
		},
	},
	props: {
		children: { is: 'The footprints, in the shape the caller is waiting in.' },
		at: {
			is: 'Pins the field this far into its pass, for a still. Unset in every caller.',
			choices: { running: undefined, '0.00': 0, '0.25': 0.25, '0.50': 0.5, '0.75': 0.75 },
		},
		className: { is: 'Classes for the body, where the caller’s layout needs them.' },
	},
	parts: {
		body: {
			select: '[data-ui-glaze-body]',
			is: 'The one field. Its height sets the wave’s pitch.',
		},
		note: { select: '[role="status"]', is: 'The word and the seconds, placed by the caller.' },
		row: {
			select: '[aria-hidden] > .flex',
			is: 'A row at the row pitch, on the rule it would draw.',
		},
		footprint: {
			select: '[data-ui-glaze-bar]',
			is: 'A block, never a box: 9px of the row’s ground.',
		},
	},
	holds: [
		'The glaze is a train of bands passing down the page, top to bottom.',
		'Brightness is a function of where you are on the page, never of which element you are.',
		'The field never empties: some band is always on the page.',
		'Every offset is measured, and measured again on a fold.',
		'The note prints the word, always. Under reduced motion it is the whole of the message.',
		'A failed wait stops the glaze and says so, with Try again beside it.',
	],
	refuses: [
		'Hue. A load has no measure, so no accent meter and no coloured bar.',
		'Control ink. A footprint is not operable, and no slot is drawn empty.',
		'A shimmer: a highlight travelling inside each placeholder.',
		'A breath, which has no direction in it.',
		'A stagger from element to element, which quantises the band to whole boxes.',
		'A single band with a gap after it, which leaves the page dark for 35% of every loop.',
	],
});
