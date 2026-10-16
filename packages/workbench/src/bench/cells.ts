import type { FrameKind } from '../lib/frame-url';
import type { FrameRef } from '../lib/stage-search';
import type { ChromeKind, Subject } from '../lib/subject';

/** One frame on the stage: what its caption says, and the address it is drawn from. */
export type StageCell = {
	/** What the address's `variants` and `at` name it by. */
	id: string;
	label: string;
	note?: string;
	kind: FrameKind;
	subject: string;
	variant: string;
	chrome: ChromeKind;
};

/** A subject's states, as the cells the stage draws. */
export function stateCells(kind: FrameKind, subject: Subject): StageCell[] {
	return subject.variants.map(variant => ({
		id: variant.id,
		label: variant.label,
		...(variant.note === undefined ? {} : { note: variant.note }),
		kind,
		subject: subject.id,
		variant: variant.id,
		chrome: subject.chrome,
	}));
}

/**
 * A `frames` list, as cells: frames of more than one subject, side by side.
 *
 * `variants` can only name states of one subject. A candidate is compared with the place it would
 * replace and with the candidates beside it, and those are different subjects in different spaces.
 * A cell is named by its subject, since that is what differs across the row, with the state after
 * it. A subject nobody registered is named back rather than drawn as a hole.
 */
export function listCells(
	refs: readonly FrameRef[],
	find: (id: string) => Subject | undefined,
	kindOf: (id: string) => FrameKind
): { cells: StageCell[]; missing: string[] } {
	const cells: StageCell[] = [];
	const missing: string[] = [];

	for (const ref of refs) {
		const subject = find(ref.subject);
		const variant =
			subject?.variants.find(candidate => candidate.id === ref.variant) ?? subject?.variants[0];

		if (subject === undefined || variant === undefined) {
			if (!missing.includes(ref.subject)) missing.push(ref.subject);
			continue;
		}

		const id = `${subject.id}:${variant.id}`;
		if (cells.some(cell => cell.id === id)) continue;

		cells.push({
			id,
			label: subject.title,
			note: variant.label,
			kind: kindOf(subject.id),
			subject: subject.id,
			variant: variant.id,
			chrome: subject.chrome,
		});
	}

	return { cells, missing };
}
