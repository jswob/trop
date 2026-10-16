import { matchBracket, skipAtomic } from './scan-source';

const DECLARATION = /export\s+(type|interface)\s+PROPOSED_SHAPE\b/;

/**
 * Lifts the `PROPOSED_SHAPE` declaration out of a prototype's source so the
 * workbench can show it beside the screen. Types are erased at runtime, and the
 * shape is the handoff artefact — it has to come from the file itself or it
 * would drift from what the prototype actually renders.
 *
 * Returns the declaration verbatim, or `null` when the file declares none.
 */
export function parseProposedShape(source: string): string | null {
	const match = DECLARATION.exec(source);
	if (!match) return null;

	const start = match.index;
	let cursor = match.index + match[0].length;

	if (match[1] === 'interface') {
		const brace = source.indexOf('{', cursor);
		if (brace === -1) return null;
		const end = matchBracket(source, brace);
		return end === -1 ? null : source.slice(start, end).trim();
	}

	while (cursor < source.length) {
		const skipped = skipAtomic(source, cursor);
		if (skipped !== -1) {
			cursor = skipped;
			continue;
		}

		const char = source[cursor] ?? '';
		if ('{[('.includes(char)) {
			const end = matchBracket(source, cursor);
			if (end === -1) return null;
			cursor = end;
			continue;
		}
		if (char === ';') return source.slice(start, cursor + 1).trim();
		cursor += 1;
	}

	return source.slice(start).trim();
}
