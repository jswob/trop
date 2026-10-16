import { DESIGN_SOURCES } from './design-sources';

/**
 * Who actually renders a kit component, read off the source rather than written down.
 *
 * Rule 8 says a component enters the kit on its second caller, so the caller count is the
 * one fact on a kit page that must never be stale — a hand-kept list would claim two
 * callers long after one of them was deleted, which is exactly the claim the rule rests
 * on. The sources are pulled in raw and scanned for the exported names, the same way
 * `/audit` enumerates the current app's variants from its CVA configs.
 *
 * The file a name is exported from never counts as a caller of itself, and **prose is not a
 * caller**: comments are stripped before the scan. Naming `kit/PageSpread` in a doc comment is how
 * these files explain themselves, and counting it claimed two callers for a component that had one
 * — which is a lie in the one direction that matters, because rule 8's whole job is to stop a
 * component entering the kit on a single use. `views/frame.test` strips comments the same way and
 * for the same reason.
 */

/**
 * The code, without the prose and without the path to any specimen. Only a reference the compiler
 * sees is a use, and a path naming a component's specimen (`kit/Sheet.specimen`) draws nothing of
 * it. The same as the terminal's `usesOf`, which cannot be imported into the browser.
 */
function usesOf(source: string): string {
	return source
		.replace(/\/\*[\s\S]*?\*\//g, '')
		.replace(/^[ \t]*\/\/.*$/gm, '')
		.replace(/(\bfrom\s*|^[ \t]*import\s*|\bimport\(\s*)'[^']*\.specimen[^']*'/gm, "$1''");
}

export function callersOf(symbols: string[], ownFiles: string[]): string[] {
	const own = new Set(ownFiles);
	const pattern = new RegExp(`\\b(?:${symbols.join('|')})\\b`);

	return Object.entries(DESIGN_SOURCES)
		.map(([path, source]) => [path, usesOf(source)] as const)
		.filter(([path]) => !own.has(path) && !path.includes('.test.'))
		.filter(([path]) => !path.endsWith('/index.ts'))
		.filter(([, source]) => pattern.test(source))
		.map(([path]) => path)
		.sort();
}

/** Which files render a kit component, counted from the source rather than declared. */
export function callersFor(entry: { symbols: string[]; files: string[] }): string[] {
	return callersOf(entry.symbols, entry.files);
}
