import { designSources } from '../sources';
import { host } from './host';

/** The code, without the prose. Only a reference the compiler sees is a use. */
export function codeOf(source: string): string {
	return source.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^[ \t]*\/\/.*$/gm, '');
}

/**
 * The code a caller is counted in: without the prose, and without the path to any specimen. A
 * fixture that takes a component's scenes names the component in that path (`kit/Sheet.specimen`)
 * and draws nothing of it.
 */
export function usesOf(source: string): string {
	return codeOf(source).replace(
		/(\bfrom\s*|^[ \t]*import\s*|\bimport\(\s*)'[^']*\.specimen[^']*'/gm,
		"$1''"
	);
}

/** Every exported symbol of a module, for the export line a promotion has to write. */
export function exportedSymbols(source: string): { values: string[]; types: string[] } {
	const code = codeOf(source);
	const values = [
		...code.matchAll(/^export\s+(?:async\s+)?(?:function|const|let|class)\s+([A-Za-z0-9_$]+)/gm),
	].map(match => match[1] ?? '');
	const types = [...code.matchAll(/^export\s+(?:type|interface)\s+([A-Za-z0-9_$]+)/gm)].map(
		match => match[1] ?? ''
	);

	return {
		values: [...new Set(values)].filter(name => name !== '' && name !== 'PROPOSED_SHAPE').sort(),
		types: [...new Set(types)].filter(name => name !== '' && name !== 'PROPOSED_SHAPE').sort(),
	};
}

/**
 * Who actually renders these symbols — **rule 8's number, read off the source**.
 *
 * The same rules `lib/kit-callers` applies in the browser, applied here from the filesystem,
 * because a promotion happens at a terminal and the number it turns on must be the same number the
 * kit page prints. Prose is not a caller: naming a component in a doc comment claimed two callers
 * for a component that had one, which is a lie in the one direction that matters.
 */
export function callersOf(symbols: string[], ownFiles: string[]): string[] {
	if (symbols.length === 0) return [];

	const own = new Set(ownFiles);
	const pattern = new RegExp(`\\b(?:${symbols.join('|')})\\b`);

	return designSources(host())
		.filter(([file]) => !own.has(file))
		.filter(([file]) => !file.includes('.test.'))
		.filter(([file]) => !file.endsWith('/index.ts'))
		.filter(([, source]) => pattern.test(usesOf(source)))
		.map(([file]) => file)
		.sort();
}
