import { specimen } from '@jswob/design-workbench';
import { Plus } from 'lucide-react';
import { Button } from '@app/ui/kit';

export default specimen(Button, {
	kit: 'button',
	is: 'Every action a hand can take, at three rungs of emphasis.',
	base: { children: 'Save', weight: 'normal' },
	states: {
		rest: { label: 'At rest', note: 'The middle rung: a hairline box.', args: {} },
		primary: {
			label: 'Primary',
			note: 'The plate. The screen’s one main action.',
			args: { weight: 'primary' },
		},
		quiet: { label: 'Quiet', note: 'A bare word, for what can wait.', args: { weight: 'quiet' } },
		pair: {
			label: 'With a glyph',
			note: 'A lucide icon, 8px before the word.',
			args: { icon: Plus },
		},
		running: {
			label: 'Running',
			note: 'Its run is going: the word changes.',
			args: { weight: 'primary', running: 'Saving' },
		},
		blocked: {
			label: 'Blocked',
			note: 'Why it cannot be pressed, said beside it.',
			args: { weight: 'primary', blocked: 'Pick a project first' },
		},
	},
	props: {
		children: { is: 'The word on it.', text: true },
		weight: {
			is: 'How much surface the button owns. Defaults to normal.',
			options: { primary: true, normal: true, quiet: true },
		},
		icon: {
			is: 'A lucide icon at 16px, 8px before the word.',
			choices: { none: undefined, plus: Plus },
		},
		running: { is: 'The word to print while the run it started is going.', text: true },
		blocked: { is: 'Why it cannot be pressed. Disables it and prints the reason.', text: true },
		disabled: { is: 'Off with no reason given. Prefer blocked.', toggle: true },
		describedBy: { is: 'The id of what says why, where something else prints the reason.' },
		label: { is: 'An accessible name, where the word is not the whole of it.' },
		href: { is: 'Where it goes, where it is a link drawn as a button.' },
		pressed: { is: 'A button that stays in. The wire alone; the caller draws it.' },
		onClick: { is: 'What a press does.' },
	},
	parts: {
		plate: { select: 'button', is: 'The ground the word sits on.' },
		glyph: { select: 'button > svg', is: 'A lucide icon, before the word.' },
		word: { select: 'button > span', is: 'Centred on its type, not on its line box.' },
		reason: { select: 'button + span', is: 'Why it is blocked, printed beside it.' },
	},
	holds: [
		'Emphasis is area: a plate, a hairline box, a bare word.',
		'The plate dims under the pointer; the lower rungs fill.',
		'Running cannot be pressed, and does not dim: the plate empties, the ink stays up.',
		'A word is centred on its type, not on its line box.',
		'Hover eases over 120ms. A press is never late.',
	],
	refuses: [
		'A coloured rung, danger included. Red warns in type beside the action.',
		'A tinted ground in place of the hairline box.',
		'Two weights with no middle rung.',
		'A progress bar on a pending button.',
		'The text button as another rung.',
	],
});
