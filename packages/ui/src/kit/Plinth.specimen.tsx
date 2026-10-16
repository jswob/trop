import { specimen } from '@jswob/design-workbench';
import { useId } from 'react';
import { HoldingProvider, Plinth } from '@app/ui/kit';
import type { PlinthMember } from '@app/ui/kit';

/**
 * The feet `kit/Plinth`'s page draws, as data.
 *
 * The `design` skill's `COMPONENT.md` asks a variant grid to be **enumerated from a config** rather
 * than hand-listed, so the page cannot quietly stop showing a state the component still has. Everything the grid draws
 * is here and nothing is written in the page.
 *
 * **These are specimens, not the app's vocabulary.** A record's own words — and the conditions
 * that choose between them — live in `views/commit`, which is the one place `Save changes` is
 * spelled for the app. The kit imports no view, so the strings are repeated here on purpose and
 * they are repeated as *examples*: a foot is drawn with whatever words its caller has.
 */

export type PlinthScene = {
	/** URL-safe, and the cell's key. */
	id: string;
	label: string;
	/** What this cell is for, and what to look at in it. */
	note: string;
	members: PlinthMember[];
};

const SAVE = 'Save changes';
const REVERT = 'Discard changes';
const DELETE = 'Delete note';

/** The document's three members, in the order they stand. The condition changes the finisher only. */
function document(finish: Partial<PlinthMember>): PlinthMember[] {
	return [
		{ id: 'finish', word: SAVE, weight: 'primary', ...finish },
		{ id: 'leave', word: REVERT },
		{ id: 'destroy', word: DELETE, destroys: true },
	];
}

export const PLINTH_SCENES: PlinthScene[] = [
	{
		id: 'ready',
		label: 'Ready',
		note: 'the errand can be finished — no reason, so nothing stands between the rule and the members',
		members: document({}),
	},
	{
		id: 'clean',
		label: 'Nothing to do',
		note: 'the finisher is off and says why, at ui-ink-3: not wrong, just nothing to finish',
		members: document({ reason: 'Nothing changed yet', tone: 'quiet' }),
	},
	{
		id: 'blocked',
		label: 'Cannot be finished',
		note: 'the same slot in ui-red, holding a sentence built from the record rather than a note beside the field that failed',
		members: document({
			reason: 'Needs a title and a project.',
			tone: 'wrong',
		}),
	},
	{
		id: 'saving',
		label: 'Running',
		note: "Button's own running: the plate empties to ui-beat and keeps its ink, which is not the same as off",
		members: document({ running: 'Saving' }),
	},
	{
		id: 'create',
		label: 'Nothing to destroy',
		note: 'a create errand: two members, one band, and no second band at all — the block does not reserve a line for a member that does not exist',
		members: [
			{ id: 'finish', word: 'Add note', weight: 'primary' },
			{ id: 'leave', word: 'Discard' },
		],
	},
	{
		id: 'row',
		label: 'Four members',
		note: 'an open row: three on band one and the destroying member alone below, at every width — never pushed there by a wrap',
		members: [
			{
				id: 'keep',
				word: 'Save this list',
				weight: 'primary',
				reason: 'A list needs a name.',
				tone: 'wrong',
			},
			{ id: 'extend', word: 'Add an item' },
			{ id: 'leave', word: 'Discard' },
			{ id: 'destroy', word: 'Delete', destroys: true },
		],
	},
	{
		id: 'over-cap',
		label: 'Over the cap',
		note: 'five members: BAND_CAP takes three, the fourth starts band two and the destroying member joins it. The split is arithmetic, never a layout prop',
		members: [
			{ id: 'finish', word: 'Send the list', weight: 'primary' },
			{ id: 'again', word: 'Send again' },
			{ id: 'export', word: 'Export the list' },
			{ id: 'leave', word: 'Discard the draft' },
			{ id: 'destroy', word: 'Delete the list', destroys: true },
		],
	},
];

/** Why each scene is listed, at the cap. A scene added with no line here fails the words test. */
const NOTES: Record<string, string> = {
	ready: 'The errand can be finished, so no reason is drawn.',
	clean: 'The finisher is off and says why, in the third ink.',
	blocked: 'The same slot in red: a sentence built from the record.',
	saving: 'Its run is going: the plate empties and keeps its ink.',
	create: 'A create errand: two members and no second band.',
	row: 'Three on band one, the destroying member alone below.',
	'over-cap': 'Five members: the fourth starts band two.',
};

const membersOf = (id: string): PlinthMember[] =>
	PLINTH_SCENES.find(scene => scene.id === id)?.members ?? [];

/** The document's foot while the chrome holds its errand: the slug has all but the destroyer. */
const HELD = membersOf('ready').map(member =>
	member.destroys === true ? member : { ...member, chrome: true }
);

/** A foot whose members differ by width: two stand from 1024, one under it. */
const BY_WIDTH: PlinthMember[] = [
	{ id: 'finish', word: 'Finish', weight: 'primary' },
	{ id: 'all', word: 'Select all', at: 'wide' },
	{ id: 'clear', word: 'Clear all', at: 'wide' },
	{ id: 'sheet', word: 'Pick on the sheet', at: 'narrow' },
];

/** A member that is off because of a sentence the page already prints over the foot. */
function SaidAbove() {
	const said = useId();

	return (
		<div className="space-y-6">
			<p id={said} className="text-ui-body text-ui-ink">
				Every note in 5 projects is picked.
			</p>
			<Plinth
				members={[
					{ id: 'finish', word: 'Finish', weight: 'primary' },
					{ id: 'all', word: 'Select all', saidBy: said },
					{ id: 'clear', word: 'Clear all' },
				]}
			/>
		</div>
	);
}

export default specimen(Plinth, {
	kit: 'plinth',
	is: 'The foot of a thing: every control that finishes, leaves or destroys what is above it.',
	width: 512,
	states: {
		...Object.fromEntries(
			PLINTH_SCENES.map(scene => [
				scene.id,
				{ label: scene.label, note: NOTES[scene.id] ?? '', args: { members: scene.members } },
			])
		),
		widths: {
			label: 'A member a width',
			note: 'Two stand from 1024 and one under it; the cap counts each.',
			args: { members: BY_WIDTH },
		},
		said: {
			label: 'Said above',
			note: 'Off because of the line over the foot, so it prints none.',
			draw: () => <SaidAbove />,
		},
		held: {
			label: 'Held by the chrome',
			note: 'Under 1024 the slug has Save and Discard; one stays.',
			draw: () => (
				<HoldingProvider value>
					<Plinth members={HELD} />
				</HoldingProvider>
			),
		},
	},
	props: {
		members: { is: 'Every control of the foot, in reading order. The block works out the bands.' },
	},
	parts: {
		block: {
			select: '.border-t',
			is: 'The hairline at the full measure, 12px over the first line.',
		},
		band: { select: '.border-t > .flex', is: 'Band one: the finisher first, three at most.' },
		plate: { select: '.border-t > .flex > button', is: 'The member that finishes the errand.' },
		box: { select: '.border-t > .flex > button + button', is: 'Any other member: a hairline box.' },
		below: {
			select: '.border-t > .flex + .flex',
			is: 'Band two: what destroys the record, 12px under.',
		},
	},
	holds: [
		'A foot opens with a 1px hairline at the full measure, and every member is at least a box.',
		'Three members on a band, and the member that destroys the record is alone below.',
		'The bands never wrap. Below 768 the members take a line each in the same reading order.',
		'The reason stands in the block, once: red where the record is wrong, third ink otherwise.',
		'Nothing is reserved for a reason that is not there.',
		'The space above the hairline belongs to whatever it is the foot of.',
	],
	refuses: [
		'One wrapping row, which puts Delete under the others on a narrow column.',
		'A margin pushing the destroying member to the far end of the row.',
		'A text button anywhere in a foot. Four bare words under a box are four labels.',
		'A reason beside the button it is about, or a second reason slot.',
		'A fixed 32px inside the block, which puts the members 81px off an open row’s box.',
		'A member that comes and goes with the data.',
	],
});
