/**
 * What an app's design root imports from the workbench: how a subject and a specimen are declared,
 * how a frame keeps its place in the address, and how the browser's half of the binding is typed.
 *
 * Everything here runs in a browser and under a test runner. The terminal's half is
 * `@jswob/design-workbench/config`, and a thesis's numbers are declared with
 * `@jswob/design-workbench/thesis`.
 */
export {
	parseTurned,
	specimen,
	specimenOf,
	type PropDoc,
	type Specimen,
	type SpecimenInput,
	type SpecimenPart,
	type SpecimenProp,
	type SpecimenState,
} from './bench/specimen';
export type { Appearance } from './lib/appearance';
export { readPosition, useFramePosition, writePosition } from './lib/frame-position';
export { parseProposedShape } from './lib/proposed-shape';
export {
	defineSetup,
	settledSubjectsOf,
	subjectsOf,
	type KitEntry,
	type PlaceSubject,
	type Section,
	type WorkbenchSetup,
} from './lib/setup';
export {
	DESCRIPTION_CAP,
	NOTE_CAP,
	type ChromeKind,
	type Subject,
	type SubjectVariant,
} from './lib/subject';
