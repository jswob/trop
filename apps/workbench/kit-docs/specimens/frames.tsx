import { useKeyboardKept } from '@app/ui/kit/keyboard';
import { parseTurned, type Specimen, type Subject } from '@jswob/design-workbench';

import { Ground, Screen } from './ground';
import { SPECIMENS } from './index';

/**
 * One state of a specimen, as the frame it is opened in draws it.
 *
 * What the bench turned is on the frame's own address (`args`), so a turned state is a link like
 * any other frame, and what `Open` shows is what the stage showed.
 *
 * A component leaves the keyboard to the frame it is drawn in: a press that takes its own control
 * away is landed by the shell. A specimen has no shell, so its ground does that one job, and
 * a sweep of it reports what the component does and not what its frame would have.
 */
function SpecimenFrame({ spec, stateId }: { spec: Specimen; stateId: string }) {
	useKeyboardKept(() => document.querySelector<HTMLElement>('[data-specimen]'));

	const turned = parseTurned(new URLSearchParams(window.location.search).get('args') ?? undefined);

	if (spec.stands === 'screen') return <Screen>{spec.render(stateId, turned)}</Screen>;

	return <Ground width={spec.width}>{spec.render(stateId, turned)}</Ground>;
}

/**
 * The specimens as staged subjects, so each state has a frame address that `drive`, `probe` and a
 * board can reach. One that wraps a subject already staged adds none of its own.
 */
export const SPECIMEN_SUBJECTS: Subject[] = SPECIMENS.filter(spec => !spec.staged).map(spec => ({
	id: spec.subjectId,
	title: spec.id,
	description: spec.is,
	chrome: 'bare',
	variants: spec.states.map(state => ({
		id: state.id,
		label: state.label,
		note: state.note,
		render: () => <SpecimenFrame spec={spec} stateId={state.id} />,
	})),
}));
