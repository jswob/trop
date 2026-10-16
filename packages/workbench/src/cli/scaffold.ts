/**
 * Registering a candidate, and un-registering it — as one transform each.
 *
 * `BUILD.md` carries a rule that exists only because this was hand surgery: *"main appends them —
 * one shared file, one append each, done once by one thread rather than raced by N."* Three agents
 * editing one `index.ts` is a race because a text edit is a race; a command that applies the same
 * transform twice is idempotent, and the rule stops being about who holds the pen.
 *
 * The transforms are pure and take the source, so they are testable without a filesystem — which
 * matters more than it sounds. The failure this replaces is a `python3` heredoc that asserted an
 * anchor appeared exactly once and then wrote the file; when the anchor had moved, it threw, and
 * when it had merely changed shape it did not.
 */
export function camelOf(id: string): string {
	return id.replace(/-([a-z0-9])/g, (_match, letter: string) => letter.toUpperCase());
}

export function pascalOf(id: string): string {
	const camel = camelOf(id);
	return camel.charAt(0).toUpperCase() + camel.slice(1);
}

const ARRAY = /(export const PROTOS: Subject\[\] = \[)([\s\S]*?)(\];)/;

function members(body: string): string[] {
	return body
		.split(',')
		.map(value => value.trim())
		.filter(Boolean);
}

/** The last top-level import's end: where a candidate's import goes when none sorts after it. */
function afterImports(source: string): number {
	const imports = [...source.matchAll(/^import [\s\S]*?;$/gm)];
	const last = imports[imports.length - 1];
	return last === undefined ? 0 : (last.index ?? 0) + last[0].length;
}

export function registerProto(source: string, id: string): string {
	const name = camelOf(id);
	const match = ARRAY.exec(source);
	if (match === null)
		throw new Error('proto/index.ts has no `export const PROTOS: Subject[] = [...]`');

	const existing = members(match[2] ?? '');
	if (existing.includes(name)) return source;

	const listed = source.replace(ARRAY, `$1${[...existing, name].join(', ')}$3`);
	const line = `import { ${name} } from './${id}';`;
	const later = [...listed.matchAll(/^import \{ \w+ \} from '\.\/([\w-]+)';$/gm)].find(
		sibling => (sibling[1] ?? '') > id
	);
	if (later?.index !== undefined)
		return `${listed.slice(0, later.index)}${line}\n${listed.slice(later.index)}`;

	const cut = afterImports(listed);
	return `${listed.slice(0, cut)}\n${line}${listed.slice(cut)}`;
}

export function unregisterProto(source: string, id: string): string {
	const name = camelOf(id);
	const match = ARRAY.exec(source);
	if (match === null)
		throw new Error('proto/index.ts has no `export const PROTOS: Subject[] = [...]`');

	const kept = members(match[2] ?? '').filter(member => member !== name);

	return source
		.replace(ARRAY, `$1${kept.join(', ')}$3`)
		.replace(new RegExp(`^import \\{ ${name} \\} from '\\./${id}';\\n`, 'm'), '');
}

/**
 * A board's drawing, checked before it becomes the candidate.
 *
 * A drawing a board approved is already a prototype in everything but its address (`workbench
 * board` resolves its imports from `proto/`), so the candidate starts as that file rather than as a
 * scaffold somebody re-types the drawing into. The two things that would make it register and then
 * not resolve are checked here: the registry imports a candidate under its camel-cased id, and the
 * frame address finds it by the `id` its `Subject` carries.
 */
export function assertDrawing(source: string, id: string): string {
	const name = camelOf(id);

	if (!new RegExp(`^export const ${name}\\b`, 'm').test(source)) {
		throw new Error(
			`the drawing has to hold \`export const ${name}\` to be registered as ${id} — it does not`
		);
	}
	if (!new RegExp(`\\bid: ['"]${id}['"]`).test(source)) {
		throw new Error(`the drawing's Subject has to carry \`id: '${id}'\` — it does not`);
	}

	return source;
}

/** Text for a single-quoted string in the written source: a `'` in a bet ends the string early. */
function quoted(text: string): string {
	return text.replaceAll('\\', '\\\\').replaceAll("'", "\\'");
}

/**
 * A candidate that compiles, registers and renders on its first run.
 *
 * It is deliberately almost empty, and it spends none of the app's classes: the frame's ground
 * already sets the ink. The thesis is in the header because `BUILD.md` requires the
 * agent building this to have it verbatim, and a file whose first line is the bet it is making is
 * one that cannot drift from it without the drift being visible in a diff.
 */
export function protoTemplate(
	id: string,
	title: string,
	thesis: string,
	on: { app: string; theme: string; engine: string }
): string {
	const name = camelOf(id);

	return `import type { Subject } from '${on.engine}';

/**
 * ${title}
 *
 * **The bet.** ${thesis}
 *
 * Axis values this commits to — state them as numbers, not adjectives, or three candidates come
 * back with three different readings of the same word:
 *
 * - TODO
 *
 * Built on the ${on.app} kit (\`../kit\`) and the tokens of \`${on.theme}\`, which are the only
 * palette there is.
 */

/** What this prototype would hand over if it settled. */
export type PROPOSED_SHAPE = {
	TODO: unknown;
};

export const ${name}: Subject = {
	id: '${id}',
	title: '${quoted(title)}',
	description: '${quoted(thesis)}',
	chrome: 'bare',
	variants: [
		{
			id: 'resting',
			label: 'Resting',
			note: 'TODO — say which state this is, and why no press reaches it.',
			render: () => <div style={{ padding: 32 }}>TODO: draw ${title}.</div>,
		},
	],
};
`;
}

/**
 * A candidate's thesis file: where the numbers the thesis committed to are written down.
 *
 * It starts with no checks on purpose. They are written by whoever holds the approved board,
 * before the build starts, and `conform` fails a thesis that has none.
 */
export function thesisTemplate(id: string, bet: string, engine: string): string {
	return `import { thesis } from '${engine}/thesis';

/**
 * The numbers this thesis committed to, written when it was approved and before it was built.
 * \`bun workbench conform ${id}\` holds the live frames against them.
 *
 * A part is found by \`data-part="<name>"\` in the frame, or by a selector named in \`parts\`.
 */
export const numbers = thesis({
	id: '${id}',
	bet: '${quoted(bet)}',
	checks: [],
});
`;
}
