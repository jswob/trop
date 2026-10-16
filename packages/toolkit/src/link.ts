/**
 * `toolkit link`: the app runs on the toolkit's checkout instead of the installed packages, to try
 * a change too big for a patch before it is released. `toolkit try` runs the app's tests across it.
 *
 * Bun's own link is not used: run from the root it was ignored without a word, and a workspace has
 * to be linked one by one. Here each folder's `node_modules/<package>`, the root's and every
 * workspace's, becomes a link to the package's folder in the checkout, and its copies in `.claude/`
 * are synced from there. A linked package resolves its own dependencies from the checkout, so the
 * checkout must be installed, and a Vite or Vitest config that must see one React dedupes it.
 * The marker records each link; `unlink` removes them and lets Bun put the installed copies back.
 */
import {
	existsSync,
	lstatSync,
	mkdirSync,
	readFileSync,
	readlinkSync,
	realpathSync,
	rmSync,
	symlinkSync,
	writeFileSync,
} from 'node:fs';
import path from 'node:path';

import { sync, type Synced } from './claude';
import { readPatching, runCommand, type Run } from './patches';
import { installedCopies, packagesOf, workspaceDirs } from './repo';

/** Each linked package: its folder in the checkout, and the links made to it, from the repo's root. */
export type Linked = Record<string, { dir: string; paths: string[] }>;

/** Where the marker is kept, under the repo. Inside `node_modules`, so it is never committed. */
export const LINK_FILE = 'node_modules/.toolkit/linked.json';

export function readLinks(repo: string): Linked {
	const file = path.join(repo, LINK_FILE);
	return existsSync(file) ? (JSON.parse(readFileSync(file, 'utf8')) as Linked) : {};
}

function writeLinks(repo: string, linked: Linked): void {
	const file = path.join(repo, LINK_FILE);
	if (Object.keys(linked).length === 0) {
		rmSync(file, { force: true });
		return;
	}
	mkdirSync(path.dirname(file), { recursive: true });
	writeFileSync(file, `${JSON.stringify(linked, null, '\t')}\n`, 'utf8');
}

function lexists(file: string): boolean {
	try {
		lstatSync(file);
		return true;
	} catch {
		return false;
	}
}

/** Whether `file` is a link to `dir`. */
function linksTo(file: string, dir: string): boolean {
	return lexists(file) && lstatSync(file).isSymbolicLink() && readlinkSync(file) === dir;
}

/** Each folder's `node_modules/<name>` that holds the package: the root's, then the workspaces'. */
function installPaths(repo: string, name: string): string[] {
	return [repo, ...workspaceDirs(repo)]
		.map(dir => path.join(dir, 'node_modules', name))
		.filter(file => existsSync(path.join(file, 'package.json')))
		.map(file => path.relative(repo, file));
}

/** The packages of the checkout this repo installs from the registry, or has linked. */
function linkable(repo: string, packages: Map<string, string>, linked: Linked): string[] {
	return [...packages.keys()].filter(
		name => linked[name] !== undefined || installedCopies(repo, name).some(copy => !copy.workspace)
	);
}

/**
 * Link packages to their folders in the toolkit's checkout, and sync their copies from there.
 * Linking a linked package again puts back the links an install replaced.
 * @param repo - The app's root
 * @param home - The toolkit's checkout
 * @param only - These packages; every package of the checkout the app installs, by default
 */
export function link(
	repo: string,
	home: string,
	only?: readonly string[]
): { linked: { name: string; dir: string; paths: string[] }[]; synced: Synced[] } {
	if (realpathSync(home) === realpathSync(repo)) {
		throw new Error(`${home} is this repo: its packages are its own source already`);
	}
	const packages = packagesOf(home);
	const linked = readLinks(repo);
	const names = only === undefined ? linkable(repo, packages, linked) : [...only];

	if (!existsSync(path.join(home, 'node_modules'))) {
		throw new Error(
			`${home} is not installed: a linked package takes its dependencies from there (bun install in it)`
		);
	}

	const planned = names.map(name => {
		const dir = packages.get(name);
		if (dir === undefined) throw new Error(`${name} is not a package of ${home}`);
		if (readPatching(repo)[name] !== undefined) {
			throw new Error(`${name} is being patched: bun toolkit patch ${name} --commit, or --abort`);
		}
		const own = installedCopies(repo, name).find(
			copy => copy.workspace && copy.root !== realpathSync(dir)
		);
		if (own !== undefined) {
			throw new Error(
				`${name} is this repo's own source at ${path.relative(realpathSync(repo), own.root)}, or linked elsewhere`
			);
		}
		const paths = [...new Set([...(linked[name]?.paths ?? []), ...installPaths(repo, name)])];
		if (paths.length === 0) throw new Error(`${name} is not installed`);
		return { name, dir, paths };
	});

	for (const { name, dir, paths } of planned) {
		for (const file of paths.map(p => path.join(repo, p))) {
			if (linksTo(file, dir)) continue;
			rmSync(file, { recursive: true, force: true });
			mkdirSync(path.dirname(file), { recursive: true });
			symlinkSync(dir, file, 'dir');
		}
		linked[name] = { dir, paths };
	}
	writeLinks(repo, linked);

	return {
		linked: planned,
		synced: sync(
			repo,
			planned.map(({ name }) => name)
		).synced,
	};
}

/**
 * Remove the links and let Bun put the installed packages back, then sync their copies again.
 * @param repo - The app's root
 * @param only - These packages; every linked package, by default
 * @param run - Runs `bun install`
 */
export function unlink(
	repo: string,
	only?: readonly string[],
	run: Run = runCommand
): { unlinked: string[]; synced: Synced[] } {
	const linked = readLinks(repo);
	const names = only === undefined ? Object.keys(linked) : [...only];
	if (names.length === 0) return { unlinked: [], synced: [] };

	for (const name of names) {
		const entry = linked[name];
		if (entry === undefined) throw new Error(`${name} is not linked`);
		for (const file of entry.paths.map(p => path.join(repo, p))) {
			if (lexists(file) && lstatSync(file).isSymbolicLink()) rmSync(file);
		}
	}
	run('bun', ['install'], repo);
	writeLinks(
		repo,
		Object.fromEntries(Object.entries(linked).filter(([name]) => !names.includes(name)))
	);

	return { unlinked: names, synced: sync(repo, names).synced };
}

/**
 * The packages linked to the toolkit's checkout, as problems: the app is not running on what it
 * installs, and its copies in `.claude/` are the checkout's. Empty when nothing is linked.
 * @param repo - The repo's root
 */
export function linkProblems(repo: string): string[] {
	return Object.entries(readLinks(repo)).map(([name, { dir, paths }]) => {
		const replaced = paths.filter(p => !linksTo(path.join(repo, p), dir));
		return replaced.length === 0
			? `${name} is linked to ${dir}: bun toolkit unlink ${name} before you commit`
			: `${name} was linked to ${dir}, and an install put ${replaced.join(', ')} back: ` +
					`bun toolkit link ${name} again, or bun toolkit unlink ${name}`;
	});
}
