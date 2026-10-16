import { specimen } from '@jswob/design-workbench';
import { TextField } from '@app/ui/kit';

export default specimen(TextField, {
	kit: 'field',
	is: 'One box with an edge on all four sides, and the free text that fills it.',
	width: 320,
	base: { label: 'Name' },
	states: {
		empty: {
			label: 'Empty',
			note: 'Nothing typed; the placeholder shows.',
			args: { placeholder: 'Groceries' },
		},
		filled: {
			label: 'Filled',
			note: 'A value: 13px from 1024, 16px under it.',
			args: { value: 'Groceries' },
		},
		hint: {
			label: 'With a hint',
			note: 'One line under the box, in the third ink.',
			args: { hint: 'Without one the list prints the date.' },
		},
		error: {
			label: 'Refused',
			note: 'The edge and the line turn red, with a mark.',
			args: { value: 'Errands', error: 'Another note already has this name.' },
		},
		disabled: {
			label: 'Disabled',
			note: 'Off: it keeps its value; the edge drops to the rule.',
			args: { value: 'Groceries', disabled: true },
		},
	},
	props: {
		label: { is: 'What it asks for. Always set; it may be hidden.', text: true },
		labelHidden: { is: 'For a field whose column heading already says what it is.', toggle: true },
		value: { is: 'What it holds.', text: true },
		placeholder: { is: 'What stands in an empty box.', text: true },
		hint: { is: 'One line under the box.', text: true },
		error: { is: 'Why the value is refused. Turns the edge and the line red.', text: true },
		disabled: { is: 'Off, keeping its value.', toggle: true },
		secret: { is: 'A password box: the characters drawn as dots.', toggle: true },
		maxLength: { is: 'The length past which a keystroke does not land.' },
		width: { is: 'The room it takes. Its container’s, unless set.' },
		onChange: { is: 'What a keystroke does. Without it the box keeps its own value.' },
	},
	parts: {
		label: { select: 'div > span:first-child', is: 'What it asks for, over the box.' },
		box: { select: 'input', is: 'The target: the input fills it.' },
		line: { select: 'input + span', is: 'The hint, or why the value is refused.' },
	},
	holds: [
		'The box is the target; the input fills it.',
		'The ring belongs to the box, never to the input inside it.',
		'Disabled drops the edge to the rule and the ink to the third.',
	],
	refuses: [
		'A filled interior to say “type here”: 4.59:1 against a 5.0 floor.',
		'A label inside the box as its own cell.',
		'A chevron on anything that does not open.',
	],
});
