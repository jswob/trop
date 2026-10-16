import { useEffect, type ReactNode } from 'react';

import type { Appearance } from '../lib/appearance';
import { setup } from '../lib/bound';
import { holdMotion, holdRate, pinFor } from '../lib/motion';
import { stageEntryById } from '../lib/registry';
import type { ChromeKind, Subject } from '../lib/subject';
import { SubjectFrame } from './SubjectFrame';

export type FrameDocumentProps = {
	id: string;
	/** Which of the entry's states. Absent means its first. */
	variant?: string;
	/** The frame it stands in, where the address overrides the entry's own. */
	chrome?: ChromeKind;
	/** Milliseconds to pin every animation at. Wins over `rate`. */
	t?: number;
	/** Clock speed as a multiple of real time. `0` is off: every animation at its end state. */
	rate?: number;
	theme?: Appearance;
	/**
	 * Where an id is looked up: the workbench's registry, unless the caller holds entries of its
	 * own — a board's drawings stand in front of it (`lib/board-frame`).
	 */
	find?: (id: string) => Subject | undefined;
	/** What stands in the frame when the id, or the state named on it, finds nothing. */
	missing: (what: 'entry' | 'variant', id: string, of?: string) => ReactNode;
};

/**
 * One variant, alone, owning the whole viewport — the document a frame is.
 *
 * Two things draw it: the workbench's frame route, which reads it off the address, and a board's
 * frame, which reads it off the frame's name (`lib/board-frame`). They share this so that a place
 * on a board is the place the workbench stages, by construction and not by a second copy kept in
 * step.
 *
 * The frame document is also where the appearance is set (`theme`, 76): on its root, as the
 * app's boot sets it, so a variant draws light without knowing there is a light.
 *
 * This is also where motion is decided, and it is decided for the whole frame
 * document rather than handed to the variant: `SubjectVariant.render()` takes no
 * arguments, and a pass over `document.getAnimations()` reaches everything a
 * variant draws without the contract having to know anything about time.
 */
export function FrameDocument({
	id,
	variant: variantId,
	chrome,
	t,
	rate,
	theme = 'dark',
	find = stageEntryById,
	missing,
}: FrameDocumentProps) {
	const pin = pinFor({ t, rate });
	useEffect(() => holdMotion(pin), [pin]);
	useEffect(() => (rate === undefined || rate === 0 ? undefined : holdRate(rate)), [rate]);
	/*
	 * Written while rendering, not in an effect: a variant reads the root as it renders
	 * (`useAppearance` starts from it), and children render before a parent's effects run. The
	 * write is idempotent, so a second render costs nothing.
	 */
	setup.applyAppearance(theme);

	const subject = find(id);
	if (!subject) return missing('entry', id);

	const variant = variantId
		? subject.variants.find(candidate => candidate.id === variantId)
		: subject.variants[0];
	if (!variant) return missing('variant', variantId ?? '', subject.id);

	return <SubjectFrame chrome={chrome ?? subject.chrome}>{variant.render()}</SubjectFrame>;
}
