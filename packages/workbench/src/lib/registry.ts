import type { Specimen } from '../bench/specimen';

import { setup } from './bound';
import type { FrameKind } from './frame-url';
import { subjectsOf, type KitEntry, type PlaceSubject, type Section } from './setup';
import type { Subject } from './subject';

/**
 * Everything the app registered, looked up: the one module that reads what the setup lists.
 *
 * The spaces — tokens, kit, views, proto — are separate on purpose, so it is never ambiguous
 * what degree of settledness a page is showing. But a frame list may put a view beside a
 * proto or a kit specimen, so `stageEntries` is the one lookup that crosses them, used only by the
 * machinery that renders frames.
 *
 * A kit component with a declared page stages its states here too, so a specimen's state has
 * a frame address like any place's.
 *
 * Ids must therefore be unique across all of them. The app's own test of its setup asserts it.
 */
export function stageEntries(): Subject[] {
	return subjectsOf(setup);
}

export function stageEntryById(id: string): Subject | undefined {
	return stageEntries().find(entry => entry.id === id);
}

/** Which space an entry came from, for the label the frame URL carries. */
export function stageKindOf(id: string): FrameKind {
	if (setup.protos.some(proto => proto.id === id)) return 'proto';
	if (setup.kit.stress.id === id) return 'kit';
	if (setup.kit.subjects.some(subject => subject.id === id)) return 'kit';
	return 'view';
}

export function viewById(id: string): PlaceSubject | undefined {
	return setup.places.find(view => view.id === id);
}

/** The place every page opens over: the one that belongs to no section. */
export function groundView(): PlaceSubject | undefined {
	return setup.places.find(view => view.dest === null);
}

export function viewsOfSection(section: string): PlaceSubject[] {
	return setup.places.filter(view => view.dest === section);
}

/** The sections with no place under them: a cell in the app's navigation with no page behind it. */
export function undrawnSections(): Section[] {
	return setup.sections.filter(section => viewsOfSection(section.id).length === 0);
}

export function protoById(id: string): Subject | undefined {
	return setup.protos.find(proto => proto.id === id);
}

export function kitEntryById(id: string): KitEntry | undefined {
	return setup.kit.entries.find(entry => entry.id === id);
}

export function specimenById(id: string): Specimen | undefined {
	return setup.kit.specimens.find(spec => spec.id === id);
}
