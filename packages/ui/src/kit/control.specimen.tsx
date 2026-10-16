import { specimen } from '@jswob/design-workbench';
import { CONTROL_H, FOCUS } from '@app/ui/kit';
import { FORCED_GROUND, FORCED_PRESS, FORCED_RING } from '@app/ui/kit/focus';

type Forced = 'rest' | 'pointer' | 'pressed' | 'keyboard';

/** The kit's own answer for each state a pointer or a keyboard would have to produce. */
const FORCED: Record<Forced, string> = {
	rest: '',
	pointer: FORCED_GROUND,
	pressed: FORCED_PRESS,
	keyboard: FORCED_RING,
};

/** A control box at the control's height, wearing the ring, with one state drawn on purpose. */
function Control({ forced = 'rest' }: { forced?: Forced }) {
	return (
		<button
			type="button"
			className={`flex ${CONTROL_H} cursor-pointer items-center border border-ui-edge px-3 text-ui-body text-ui-ink ${FOCUS} ${FORCED[forced]}`}
		>
			<span>Save</span>
		</button>
	);
}

export default specimen(Control, {
	kit: 'constants',
	is: 'The two class constants every control shares: its height, and a ring that shows.',
	states: {
		rest: {
			label: 'At rest',
			note: '40px under 1024 and 32px from it.',
			args: { forced: 'rest' },
		},
		pointer: {
			label: 'Under a pointer',
			note: 'The row ground, drawn without waiting for a pointer.',
			args: { forced: 'pointer' },
		},
		pressed: {
			label: 'Pressed',
			note: 'The beat ground, as a press lands it.',
			args: { forced: 'pressed' },
		},
		keyboard: {
			label: 'Holding the keyboard',
			note: 'The ring: 2px of ink, offset two.',
			args: { forced: 'keyboard' },
		},
	},
	props: {
		forced: {
			is: 'A state a pointer or a keyboard would have to produce, drawn on purpose.',
			options: { rest: true, pointer: true, pressed: true, keyboard: true },
		},
	},
	parts: {
		box: { select: 'button', is: 'The control, at the height every control stands at.' },
		word: { select: 'button > span', is: 'The word on it, at the body size.' },
	},
	holds: [
		'A control stands 40px where the pointer may be a finger and 32px from lg up.',
		'The ring is two pixels of ink, offset two, on every interactive thing.',
		'Focus is the only thing allowed to draw a rectangle around a control.',
		'A component that forces a state reads the kit’s constants rather than spelling them again.',
	],
	refuses: [
		'outline-none on a focus style. It cancels outline-2 and the ring never paints.',
		'A control’s height tied to the row’s. Tightening the ledger would shrink the buttons.',
		'A ring that is transitioned. It flashes the wrong colour on its way in.',
		'FOCUS on a label around a bare input. It never fires; the box rings itself.',
	],
});
