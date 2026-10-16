import { DESIGN_SOURCES } from './design-sources';
import { parseProposedShape } from './proposed-shape';

/** The shape a prototype proposes, read off its own file in the proto space. */
export function proposedShapeFor(id: string): string | null {
	const source = DESIGN_SOURCES[`proto/${id}.tsx`];
	return source === undefined ? null : parseProposedShape(source);
}
