import { specimen } from '@jswob/design-workbench';
import { Button, Modal, PlaceholderRows } from '@app/ui/kit';

const lines = (
	<>
		<p className="text-ui-body leading-5 text-ui-ink">
			Groceries goes from the 4 projects that hold it.
		</p>
		<p className="flex items-start gap-1.5 text-ui-body leading-5 text-ui-red">
			<span aria-hidden>!</span>
			This cannot be undone.
		</p>
	</>
);

const answers = (running: string | null) => (
	<>
		<Button disabled={running !== null}>Keep note</Button>
		<Button weight="primary" running={running}>
			Remove note
		</Button>
	</>
);

export default specimen(Modal, {
	kit: 'modal',
	is: 'The one overlay: a panel over a darkened page, for a decision that cannot be undone.',
	stands: 'screen',
	base: { title: 'Remove Groceries', children: lines, footer: answers(null) },
	states: {
		asking: {
			label: 'Asking',
			note: 'What goes, the warning in red, and the two answers.',
			args: { onClose: () => undefined },
		},
		reading: {
			label: 'Still reading',
			note: 'What goes has not arrived: a placeholder row stands in.',
			args: { children: <PlaceholderRows rows={1} />, onClose: () => undefined },
		},
		running: {
			label: 'Running',
			note: 'Its errand is going: no close, and Escape does nothing.',
			args: { footer: answers('Removing') },
		},
	},
	props: {
		title: { is: 'What is being decided. Also the dialog’s accessible name.', text: true },
		children: { is: 'What the decision costs, in lines of body type.' },
		footer: { is: 'The answers, right-aligned, the one that acts last.' },
		onClose: { is: 'What Escape and the close do. Absent, it cannot be closed.' },
	},
	parts: {
		scrim: { select: '[aria-hidden].absolute', is: 'The darkened page behind the panel.' },
		panel: { select: '[role="dialog"]', is: 'The page’s own ground, with an edge.' },
		head: { select: '[role="dialog"] > div:first-child', is: 'The title and the close, 44px.' },
		body: { select: '[role="dialog"] > div:nth-child(2)', is: 'The lines, 12px apart.' },
		foot: { select: '[role="dialog"] > div:last-child', is: 'The answers, over a rule.' },
	},
	holds: [
		'The backdrop darkens the page; the panel is the same ground with the same edge.',
		'It takes the keyboard on the panel when it opens, and hands it back to what raised it.',
		'Escape and the close do what the caller’s own keep does.',
		'The close is a real icon button, at the control floor.',
	],
	refuses: [
		'A shadow, a radius, or a lighter ground to lift the panel.',
		'A use that is not a decision that cannot be undone.',
		'Being closed while its errand runs.',
		'A second overlay on top of it.',
	],
});
