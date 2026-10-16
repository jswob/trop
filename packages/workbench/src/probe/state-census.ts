/**
 * Which components draw which state, and where each one gets its ink.
 *
 * The check behind `TRAPS.md`'s "one enum, two drawings": a progress meter and a row mark sat
 * 200px apart on the same table and drew `pending` in two different hues, because one read its
 * ink from a record written rounds earlier and the other's author wrote the mapping again from
 * memory. Both passed their own review — a candidate is checked against its thesis, and neither
 * thesis mentioned the other drawing.
 *
 * So the question is asked of the source rather than of a screenshot: name every state the system
 * has, and for each one list every file that draws it with hue of its own. More than one is a
 * mapping that has to be shared, not copied. Two constants agreeing today is the bug waiting.
 */

/** A state of the system, and every file that paints it with a literal hue. */
export type StateDrawing = {
	state: string;
	/** Files naming the state *and* a hue token — a drawing, not a mention. */
	inkSources: string[];
	/** Files naming the state with no hue of their own — they take it from somewhere else. */
	mentions: string[];
};

/** The code, without the prose. Naming a state in a doc comment is not drawing it. */
export function stripComments(source: string): string {
	return source.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^[ \t]*\/\/.*$/gm, '');
}

/**
 * The members of a string-union type, read off its declaration.
 *
 * Returns an empty list when the type is not found or is not a plain union, rather than
 * guessing — a census built on a guessed enum reports on states the system does not have.
 */
export function unionMembers(source: string, typeName: string): string[] {
	const match = new RegExp(`export type ${typeName}\\s*=\\s*([^;]+);`).exec(stripComments(source));
	if (!match?.[1]) return [];

	return [...match[1].matchAll(/'([^']+)'/g)]
		.map(hit => hit[1])
		.filter((member): member is string => Boolean(member));
}

/** Hue tokens a file names literally, e.g. `var(--color-accent)`. */
export function hueTokens(code: string, prefix: string): string[] {
	const found = [...code.matchAll(new RegExp(`${prefix}[a-z0-9-]+`, 'g'))].map(hit => hit[0]);
	return [...new Set(found)].sort();
}

/**
 * How far **after** a state name still counts as its value. A `Record<State, string>` entry is a
 * key followed by its ink, so the association runs forwards only.
 *
 * Looking backwards as well is what made `kit/Select.tsx` read as a second source for `open`: it
 * has a scrollbar colour near the top and an `open: boolean` parameter below it, and with comments
 * stripped the two fell inside one window. They are not a mapping; they are a file.
 */
const WINDOW = 160;

/**
 * Whether the file paints this state, as against merely naming it near some unrelated colour.
 *
 * Same-file proximity is not enough: a `Select` that takes an `open` prop and also draws its own
 * panel in a hue reads as a second ink source for the seat state called `open`, which is a wolf.
 * The hue has to be this state's *value* — which is to say, it follows the name.
 */
export function paintsState(code: string, state: string, huePrefix: string): boolean {
	const pattern = new RegExp(`(['"\`]${state}['"\`]|\\b${state}\\s*:)`, 'g');

	for (const hit of code.matchAll(pattern)) {
		const after = (hit.index ?? 0) + hit[0].length;
		if (code.slice(after, after + WINDOW).includes(huePrefix)) return true;
	}

	return false;
}

/**
 * The census. `files` is `[path, source]` pairs; nothing here reads the filesystem, so the
 * whole thing is testable without a fixture tree.
 */
export function stateCensus(
	files: readonly (readonly [string, string])[],
	states: string[],
	huePrefix: string
): StateDrawing[] {
	return states.map(state => {
		const inkSources: string[] = [];
		const mentions: string[] = [];

		for (const [path, source] of files) {
			const code = stripComments(source);
			/* Quoted, or an object key: a `Record<SlotState, string>` writes its states unquoted,
			   and that literal is exactly the mapping this check is looking for. */
			if (!new RegExp(`(['"\`]${state}['"\`]|\\b${state}\\s*:)`).test(code)) continue;
			if (paintsState(code, state, huePrefix)) inkSources.push(path);
			else mentions.push(path);
		}

		return { state, inkSources: inkSources.sort(), mentions: mentions.sort() };
	});
}

/** Every state painted from more than one place — the mapping that has to be shared. */
export function conflicts(census: StateDrawing[]): StateDrawing[] {
	return census.filter(drawing => drawing.inkSources.length > 1);
}

export function formatCensus(census: StateDrawing[]): string {
	const lines = census.map(drawing => {
		const ink =
			drawing.inkSources.length === 0 ? 'no file paints it' : drawing.inkSources.join(', ');
		const flag = drawing.inkSources.length > 1 ? '  ⚠ two ink sources' : '';
		return `  ${drawing.state.padEnd(12)} ${ink}${flag}`;
	});

	const bad = conflicts(census);
	const verdict =
		bad.length === 0
			? 'every state takes its ink from one place'
			: `${bad.length} state(s) painted from more than one place: ${bad.map(d => d.state).join(', ')}`;

	return [...lines, '', verdict].join('\n');
}
