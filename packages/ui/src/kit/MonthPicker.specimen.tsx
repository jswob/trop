import { specimen } from '@jswob/design-workbench';
import type { DayForce, MonthCell, MonthPage } from '@app/ui/kit';
import { isoDay, monthPage, MonthDay, MonthPicker, inPeriod } from '@app/ui/kit';

/**
 * What `kit/MonthPicker`'s specimen is drawn on.
 *
 * The fixtures are the picker's own, built straight on `kit/month`. They deliberately do **not**
 * come from any screen's records: a kit component's page that needs one screen's data to render is
 * a screen's component wearing a kit's name, and the first thing its page has to prove is that the
 * picker is free of the errand it was drawn for.
 */

const TODAY = isoDay(2026, 3, 4);

/** The world the drawn months are read against: what each booked day already holds. */
const BOOKED: Record<string, string> = {
	'2026-03-08': 'Mar 8',
	'2026-03-22': 'Groceries',
	'2026-04-12': 'Errands',
	'2026-04-13': 'Errands',
	'2026-04-14': 'Errands',
};

function bookedOn(iso: string): string | null {
	return BOOKED[iso] ?? null;
}

function page(year: number, month: number, start: string | null, end: string | null): MonthPage {
	return monthPage(year, month, { start, end }, { today: TODAY, booked: bookedOn });
}

/** Three months, picked so that between them every cell state is on screen at real size. */
export const MONTH_PAGES: { title: string; note: string; page: MonthPage }[] = [
	{
		title: 'A run picked',
		note: 'April 2026, Apr 5 – 7, with Errands booked the week after',
		page: page(2026, 4, '2026-04-05', '2026-04-07'),
	},
	{
		title: 'Nothing picked',
		note: 'the month we are in — today, the days behind it, and what is already booked',
		page: page(2026, 3, null, null),
	},
	{
		title: 'All three marks in one week',
		note: 'March 2026, Mar 5 – 8, so the run crosses today and a booking at once',
		page: page(2026, 3, '2026-03-05', '2026-03-08'),
	},
];

function day(date: number, over: Partial<MonthCell> = {}): MonthCell {
	return {
		iso: isoDay(2026, 4, date),
		date,
		month: 4,
		outside: false,
		past: false,
		today: false,
		booked: null,
		also: false,
		picked: null,
		...over,
	};
}

/**
 * Every state one cell can be in.
 *
 * Hover, focus and pressed cannot be produced by a screenshot, so they are forced. The
 * last two rows are the co-occurrences: a day is not picked *or* booked *or* today, it is
 * any mix of the three.
 */
export const DAY_STATES: { label: string; cell: MonthCell; force: DayForce }[] = [
	{ label: 'Default', cell: day(2), force: null },
	{ label: 'Hover', cell: day(2), force: 'hover' },
	{ label: 'Focus', cell: day(2), force: 'focus' },
	{ label: 'Pressed', cell: day(2), force: 'active' },
	{ label: 'Today', cell: day(4, { today: true }), force: null },
	{ label: 'Booked', cell: day(13, { booked: 'Errands' }), force: null },
	{ label: 'Past', cell: day(1, { past: true }), force: null },
	{ label: 'Outside the month', cell: day(30, { month: 3, outside: true }), force: null },
	{ label: 'First day', cell: day(5, { picked: 'start' }), force: null },
	{ label: 'Between', cell: day(6, { picked: 'mid' }), force: null },
	{ label: 'Last day', cell: day(7, { picked: 'end' }), force: null },
	{ label: 'One-day run', cell: day(8, { picked: 'only' }), force: null },
	{
		label: 'Picked, and booked',
		cell: day(13, { picked: 'mid', booked: 'Errands' }),
		force: null,
	},
	{
		label: 'All three at once',
		cell: day(4, { today: true, picked: 'mid', booked: 'Groceries' }),
		force: null,
	},
];

type Words = { id: string; label: string; note: string };

/**
 * The fixture's three months by their titles, in the order they are drawn here: the month
 * that draws every mark first, since the parts are measured on the first state.
 */
const PAGE_WORDS: Record<string, Words> = {
	'All three marks in one week': {
		id: 'marks',
		label: 'All three marks',
		note: 'A run, a booking and today, in one week of March.',
	},
	'A run picked': {
		id: 'run',
		label: 'A run picked',
		note: 'Apr 5 to 7: one length, which breaks where the week does.',
	},
	'Nothing picked': {
		id: 'rest',
		label: 'Nothing picked',
		note: 'This month: today, the days behind it, two bookings.',
	},
};

/** Every state of one day in the fixture, by its label there. */
const DAY_WORDS: Record<string, Words> = {
	Default: { id: 'day', label: 'A day', note: 'At rest: full ink, no ground and no edge.' },
	Hover: { id: 'hover', label: 'Under the pointer', note: 'The row ground, forced for a still.' },
	Focus: {
		id: 'focus',
		label: 'Focused',
		note: 'The ring is a rectangle, so today cannot be one.',
	},
	Pressed: { id: 'pressed', label: 'Pressed', note: 'While the press is down: the beat ground.' },
	Today: {
		id: 'today',
		label: 'Today',
		note: 'A circle round the figure, the one curve there is.',
	},
	Booked: {
		id: 'booked',
		label: 'Booked',
		note: 'A short stub under the figure, in the third ink.',
	},
	Past: {
		id: 'past',
		label: 'Past',
		note: 'Fainter, never refused: it takes a press like any day.',
	},
	'Outside the month': {
		id: 'outside',
		label: 'Outside the month',
		note: 'A spill day in the second ink, still pressable.',
	},
	'First day': {
		id: 'first',
		label: 'First day',
		note: 'The run: a 2px rule across the cell’s foot.',
	},
	Between: { id: 'between', label: 'Between', note: 'The same rule, so a run is one length.' },
	'Last day': {
		id: 'last',
		label: 'Last day',
		note: 'No word at the end: its name alone says so.',
	},
	'One-day run': {
		id: 'only',
		label: 'One day',
		note: 'Both ends at once, drawn as any picked day.',
	},
	'Picked, and booked': {
		id: 'picked-booked',
		label: 'Picked and booked',
		note: 'Only the run is drawn. The stub gives way to it.',
	},
	'All three at once': {
		id: 'all',
		label: 'All three at once',
		note: 'Today, picked and booked: the circle over the run.',
	},
};

/** A fixture entry nobody wrote words for keeps its place with an empty note, which the test fails. */
const wordsFor = (words: Record<string, Words>, key: string): Words =>
	words[key] ?? { id: key.toLowerCase().replace(/[^a-z]+/g, '-'), label: key, note: '' };

const rank = (title: string): number => {
	const at = Object.keys(PAGE_WORDS).indexOf(title);
	return at === -1 ? MONTH_PAGES.length : at;
};

/** April with a second run the caller holds and is not picking, as a record's other period is. */
const ALSO = monthPage(
	2026,
	4,
	{ start: '2026-04-05', end: '2026-04-07' },
	{ also: iso => inPeriod({ from: '2026-04-20', to: '2026-04-24' }, iso) }
);

export default specimen(MonthPicker, {
	kit: 'month-picker',
	is: 'A range of days, chosen by eye against what is already taken.',
	width: 512,
	states: {
		...Object.fromEntries(
			[...MONTH_PAGES]
				.sort((a, b) => rank(a.title) - rank(b.title))
				.map(month => {
					const { id, label, note } = wordsFor(PAGE_WORDS, month.title);
					return [id, { label, note, args: { page: month.page } }];
				})
		),
		also: {
			label: 'Another run',
			note: 'A run held and not being picked: a rung fainter.',
			args: { page: ALSO },
		},
		...Object.fromEntries(
			DAY_STATES.map(state => {
				const { id, label, note } = wordsFor(DAY_WORDS, state.label);
				return [
					id,
					{
						label,
						note,
						draw: () => (
							<div className="grid grid-cols-7">
								<MonthDay cell={state.cell} force={state.force} />
							</div>
						),
					},
				];
			})
		),
	},
	props: {
		page: { is: 'The month as whole weeks of cells, each day’s state worked out by monthPage.' },
		onStep: { is: 'What the two arrows do, given one month back or one on.' },
		onPick: { is: 'What a press on a day does, given the day.' },
		className: { is: 'What is local to the caller, on the box round the head and the grid.' },
	},
	parts: {
		head: { select: '.justify-between', is: 'Which month, and the way to the ones either side.' },
		step: { select: '.justify-between > button', is: 'An icon button, at the control floor.' },
		month: { select: '.justify-between > span', is: 'The month’s name, in the lead size.' },
		initials: { select: '.border-b', is: 'The weekday initials, over the one hairline.' },
		day: {
			select: '.gap-y-1 > [aria-label="10"], .grid-cols-7:not(.gap-y-1) > button',
			is: 'A cell: 32px tall from 1024, 40px under it.',
		},
		today: { select: '.rounded-full', is: 'A 24px circle round the figure.' },
		run: { select: '[aria-pressed="true"] > span[aria-hidden]', is: 'A 2px rule, full bleed.' },
		stub: { select: 'span.w-5[aria-hidden]', is: 'A 20px stub under a booked figure.' },
	},
	holds: [
		'No edge anywhere on it. The one hairline is under the weekday initials.',
		'Every figure is full ink. Only spill days and past days step down the ramp.',
		'Picked days side by side make one length, which breaks where the week does.',
		'A day both picked and booked draws only the run.',
		'Today is a circle round the figure, the system’s one exception to radius 0.',
		'A past day is fainter, never refused: it takes the ground and the press any day takes.',
	],
	refuses: [
		'A fill on the picked days. Selection is a rule.',
		'Words at the ends of a run. A measured length needs no labels.',
		'A dashed rule or a filled ground for a booking.',
		'A mark at the head of a cell: it reads as a rule under the row above.',
		'Today as a hairline rectangle, a bar on the seam, a type weight or a mark outside the grid.',
		'Hue, in any of it.',
	],
});
