import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';

/**
 * The words of the app a repo is, kept out of the packages that are meant to travel to other repos.
 *
 * The root `package.json` names both under `toolkit.agnostic`: `paths`, the folders that must not
 * know the app, and `words`, what would give it away. The words can only live there: written into
 * a package's own test, they would be the app's words inside the package.
 */
export type Agnostic = { paths: string[]; words: string[] };

export type Hit = { file: string; line: number; word: string };

const META = /[.*+?^${}()|[\]\\]/g;

/**
 * A word is found, whatever its case, where a word starts: `order` is found in `Order` and in
 * `orders`, never in `border`; `ledger` is found in `@app/ledgers` and in `db_ledger`.
 */
export function wordPattern(words: readonly string[]): RegExp {
	const alternatives = words.map(word => word.replace(META, '\\$&')).join('|');
	return new RegExp(`(?<![\\p{L}\\p{N}])(${alternatives})`, 'giu');
}

/** Each place in a file's text where one of the words starts, by its line. */
export function hitsIn(file: string, text: string, pattern: RegExp): Hit[] {
	return text.split('\n').flatMap((line, index) =>
		[...line.matchAll(pattern)].map(match => ({
			file,
			line: index + 1,
			word: (match[1] ?? '').toLowerCase(),
		}))
	);
}

/** `toolkit.agnostic` from the repo's root `package.json`, or `null` where it names none. */
export function readAgnostic(repo: string): Agnostic | null {
	const file = path.join(repo, 'package.json');
	if (!existsSync(file)) return null;

	const manifest = JSON.parse(readFileSync(file, 'utf8')) as {
		toolkit?: { agnostic?: Partial<Agnostic> };
	};
	const agnostic = manifest.toolkit?.agnostic;
	return agnostic === undefined
		? null
		: { paths: agnostic.paths ?? [], words: agnostic.words ?? [] };
}

/** The files under the paths that git keeps or would keep: never ignored output or dependencies. */
function filesUnder(repo: string, paths: readonly string[]): string[] {
	const listed = spawnSync(
		'git',
		['ls-files', '-z', '--cached', '--others', '--exclude-standard', '--', ...paths],
		{ cwd: repo, encoding: 'utf8' }
	);
	if (listed.status !== 0) throw new Error(`git ls-files failed: ${listed.stderr}`);

	return [...new Set(listed.stdout.split('\0').filter(file => file !== ''))].sort();
}

/** Every place an app word stands in a path that must not know the app. */
export function appWords(repo: string): Hit[] {
	const agnostic = readAgnostic(repo);
	if (agnostic === null || agnostic.words.length === 0 || agnostic.paths.length === 0) return [];

	const pattern = wordPattern(agnostic.words);
	return filesUnder(repo, agnostic.paths).flatMap(file => {
		const full = path.join(repo, file);
		if (!existsSync(full)) return [];

		const text = readFileSync(full, 'utf8');
		return text.includes('\0') ? [] : hitsIn(file, text, pattern);
	});
}
