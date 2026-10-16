import { globSync, readFileSync, readdirSync, statSync } from 'node:fs';
import path from 'node:path';

import type { ResolvedConfig } from './config';

/**
 * The app's source as text, read once in the terminal and handed to both halves of the workbench.
 *
 * A verb counts a component's callers from these, and the bench prints the same count on the
 * component's page: the server reads this list and serves it to the browser as a module, so the
 * two cannot disagree about which files were looked at or what they are called.
 */

/** The design root's own spaces, in the order a scan reads them. */
export const DESIGN_SPACES = ['views', 'proto', 'subjects', 'kit'];

export type Source = readonly [name: string, text: string];

type Roots = Pick<ResolvedConfig, 'root' | 'repo' | 'settled'>;

/** Every `.ts`/`.tsx` under `dir`, as `[path relative to root, source]`. */
export function sourcesUnder(dir: string, root: string): Source[] {
	const absolute = path.join(root, dir);
	let entries: string[];
	try {
		entries = readdirSync(absolute);
	} catch {
		return [];
	}

	return entries.flatMap(entry => {
		const full = path.join(absolute, entry);
		if (statSync(full).isDirectory()) return sourcesUnder(path.join(dir, entry), root);
		if (!/\.tsx?$/.test(entry)) return [];
		return [[path.join(dir, entry), readFileSync(full, 'utf8')] as const];
	});
}

/**
 * What a file is called: from the design root when it is in it, from the repo's root when not.
 *
 * So a settled module reads as where it lives (`packages/ui/src/kit/Mark.tsx`) and never as the
 * design root's file it was before it moved (`kit/Mark.tsx`), wherever the settled code stands.
 */
export function nameOf(file: string, roots: Pick<ResolvedConfig, 'root' | 'repo'>): string {
	const inRoot = path.relative(roots.root, file);
	const outside = inRoot.startsWith('..') || path.isAbsolute(inRoot);

	return (outside ? path.relative(roots.repo, file) : inRoot).split(path.sep).join('/');
}

/** The settled spaces, each as its folder and the name its files go by. */
export function settledSpaces(roots: Roots): { dir: string; name: string }[] {
	return roots.settled.spaces.map(space => {
		const dir = path.join(roots.settled.root, space);
		return { dir, name: nameOf(dir, roots) };
	});
}

/**
 * Whether a file is a component's specimen, its page on the bench: `Mark.specimen.tsx` beside
 * `Mark.tsx`. A specimen draws every state of what it documents, so it is never a caller and
 * never a second painter of a state.
 */
export function isSpecimen(file: string): boolean {
	return path.basename(file).includes('.specimen.');
}

/**
 * Every source a design scan reads: the named spaces of the design root, then the settled spaces,
 * without the specimens beside their components.
 *
 * Only the first is narrowed by `spaces`. A state painted from a settled file and from a design
 * root file is still painted from two places.
 */
export function designSources(roots: Roots, spaces: string[] = DESIGN_SPACES): Source[] {
	return [
		...spaces.flatMap(dir => sourcesUnder(dir, roots.root)),
		...settledSpaces(roots).flatMap(({ dir, name }) =>
			sourcesUnder('.', dir).map(([file, source]) => [`${name}/${file}`, source] as const)
		),
	].filter(([file]) => !isSpecimen(file));
}

/**
 * Every file that may spend a token, for counting which ones do.
 *
 * A test spends nothing a reader sees, and the stylesheet that declares the tokens names every
 * one of them without using any.
 */
export function spendSources(config: Pick<ResolvedConfig, 'root' | 'repo' | 'theme'>): Source[] {
	const files = new Set(config.theme.spentIn.flatMap(pattern => globSync(pattern)));

	return [...files]
		.filter(file => file !== config.theme.css && !path.basename(file).includes('.test.'))
		.map(file => [nameOf(file, config), readFileSync(file, 'utf8')] as const)
		.sort(([a], [b]) => a.localeCompare(b));
}
