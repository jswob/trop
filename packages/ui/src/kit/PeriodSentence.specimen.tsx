import { specimen } from '@jswob/design-workbench';
import { Minus } from 'lucide-react';
import { useState } from 'react';
import { CONTROL_H, ICON_BOX, PeriodSentence } from '@app/ui/kit';
import type { Period, PeriodOpen } from '@app/ui/kit';

const TODAY = '2026-10-01';

const SUBJECT = 'Garden project';

/** When a seasonal project runs. */
const SEASONS: Period[] = [
	{ from: '2026-10-06', to: '2026-11-14' },
	{ from: '2027-01-05', to: null },
];

const booked = (iso: string): string | null => (iso === '2026-10-17' ? 'Errands' : null);

/**
 * The sentence under its row, as a list stands it: the marker column, `gap-2`, the name. It owns
 * the periods, so a day pressed re-sorts them as it does in a record.
 */
function Row({ periods, opened = null }: { periods: Period[]; opened?: PeriodOpen | null }) {
	const [held, setHeld] = useState(periods);

	return (
		<div className="flex gap-2">
			<span aria-hidden className={`${ICON_BOX} text-ui-ink-2`}>
				<Minus size={16} />
			</span>
			<div className="min-w-0 flex-1">
				<p className={`flex ${CONTROL_H} items-center text-ui-lead text-ui-ink`}>{SUBJECT}</p>
				<PeriodSentence
					id="garden"
					subject={SUBJECT}
					periods={held}
					today={TODAY}
					booked={booked}
					opened={opened}
					onChange={setHeld}
				/>
			</div>
		</div>
	);
}

export default specimen(PeriodSentence, {
	kit: 'period-sentence',
	is: 'A list of periods, stated as the sentence that is also its control.',
	width: 512,
	states: {
		day: {
			label: 'An end open on a day',
			note: 'The grid under the whole sentence, on that end’s month.',
			draw: () => <Row periods={SEASONS} opened={{ index: 0, edge: 'to', turn: 0 }} />,
		},
		open: {
			label: 'An open end pressed',
			note: 'No end is held while that end is open.',
			draw: () => <Row periods={SEASONS} opened={{ index: 1, edge: 'to', turn: 0 }} />,
		},
		stated: {
			label: 'Stated',
			note: 'Nothing pressed: each end is its own press.',
			draw: () => <Row periods={SEASONS} />,
		},
		any: {
			label: 'No periods',
			note: 'Any time, and the press that adds the first.',
			draw: () => <Row periods={[]} />,
		},
	},
	props: {
		id: { is: 'What the press that opened the sentence points aria-controls at.' },
		subject: { is: 'Whose periods these are, for a reader.' },
		periods: { is: 'The periods, in date order, an open start first.' },
		today: { is: 'The day a new period starts on, and where a grid with no day opens.' },
		booked: { is: 'What a day already holds, named by the grid.' },
		opened: { is: 'The end it opens on, for a state a stage seeds rather than presses to.' },
		onChange: { is: 'Takes the periods after a day, an open word or a removal.' },
	},
	parts: {
		sentence: { select: '.flex-wrap', is: 'The statement: 13px on a 26px line that wraps.' },
		date: { select: '[data-focus="0-from"] button', is: 'An end with a day, in full ink.' },
		word: {
			select: '.flex-wrap span.text-ui-ink-2',
			is: 'A word between ends, in the second ink.',
		},
		held: { select: '[aria-expanded="true"]', is: 'The pressed end, on the pressed ground.' },
		grid: { select: '[role="group"]', is: 'The month grid and the presses, 8px under.' },
		presses: { select: '[role="group"] > .border-t', is: 'The pressed end’s two presses.' },
		thread: { select: 'div.relative > span[aria-hidden]', is: 'A 1px bracket to the grid’s foot.' },
		add: { select: '[data-focus="add"] button', is: 'Adds a period from today with no end.' },
	},
	holds: [
		'The statement is the control: a date in full ink, every word between in the second ink.',
		'It is 13px at weight 400 on a 26px line that wraps wherever the measure ends.',
		'A pressed end takes the control’s own pressed ground, as does the open word while open.',
		'The grid opens under the whole sentence at the full measure, with the two presses under it.',
		'The thread runs from the sentence to the grid’s foot, only while an end is open.',
		'A day sets the end that was pressed; a day past the other end pulls that end to meet it.',
	],
	refuses: [
		'A period a row, with a Remove at each row’s end.',
		'The ink plate on the pressed end or the open word.',
		'Every other date stepping back while one is open.',
		'The grid inside the sentence after its phrase, or the presses over the grid.',
		'Add a period on a line of its own, or a period a line.',
		'Dates at weight 500, or the sentence at 15px.',
	],
});
