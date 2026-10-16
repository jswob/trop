/**
 * The repo as Bun laid it out: its manifests, its workspaces, and where each dependency is
 * installed. Shared by every command.
 */
import { existsSync, readFileSync, readdirSync, realpathSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';

import type { Declared } from './claude';

export type Manifest = {
	name?: string;
	version?: string;
	workspaces?: string[] | { packages?: string[] };
	dependencies?: Record<string, string>;
	devDependencies?: Record<string, string>;
	optionalDependencies?: Record<string, string>;
	packageManager?: string;
	patchedDependencies?: Record<string, string>;
	scripts?: Record<string, string>;
	claude?: Declared;
	toolkit?: { home?: string };
};

/** One installed copy of a package: where it is, its version, and whether it is the repo's own. */
export type Copy = { root: string; version: string; workspace: boolean };

export function readManifest(dir: string): Manifest | null {
	const file = path.join(dir, 'package.json');
	return existsSync(file) ? (JSON.parse(readFileSync(file, 'utf8')) as Manifest) : null;
}

/** The root manifest; a repo without one throws. */
export function rootManifest(repo: string): Manifest {
	const root = readManifest(repo);
	if (root === null) throw new Error(`no package.json in ${repo}`);
	return root;
}

/** The repo's workspace folders: each `workspaces` entry is a folder or `<folder>/*`. */
export function workspaceDirs(repo: string, root: Manifest = rootManifest(repo)): string[] {
	const globs = Array.isArray(root.workspaces)
		? root.workspaces
		: (root.workspaces?.packages ?? []);
	return globs.flatMap(glob => {
		if (!glob.endsWith('/*')) return [path.join(repo, glob)];
		const parent = path.join(repo, glob.slice(0, -2));
		if (!existsSync(parent)) return [];
		return readdirSync(parent, { withFileTypes: true })
			.filter(entry => entry.isDirectory())
			.map(entry => path.join(parent, entry.name));
	});
}

/** Where `name` is installed for a package in `from`: its own `node_modules`, then the repo's. */
export function resolvePackage(repo: string, from: string, name: string): string | null {
	for (const dir of [from, repo]) {
		const candidate = path.join(dir, 'node_modules', name);
		if (existsSync(path.join(candidate, 'package.json'))) return realpathSync(candidate);
	}
	return null;
}

/**
 * Every installed copy of `name`, once each, as the root and the workspaces resolve it.
 *
 * A copy outside every `node_modules` is a workspace of this repo, or a folder linked into it: its
 * source is edited where it is, never patched.
 */
export function installedCopies(repo: string, name: string): Copy[] {
	const copies = new Map<string, Copy>();
	for (const dir of [repo, ...workspaceDirs(repo)]) {
		const root = resolvePackage(repo, dir, name);
		if (root === null || copies.has(root)) continue;
		copies.set(root, {
			root,
			version: readManifest(root)?.version ?? '0.0.0',
			workspace: !root.split(path.sep).includes('node_modules'),
		});
	}
	return [...copies.values()];
}

/** A repo's own packages by name, each with its folder. */
export function packagesOf(repo: string): Map<string, string> {
	return new Map(
		workspaceDirs(repo).flatMap(dir => {
			const name = readManifest(dir)?.name;
			return name === undefined ? [] : [[name, dir] as const];
		})
	);
}

/**
 * The toolkit's checkout, where `upstream`, `link` and `try` reach the packages' source: the flag,
 * then `TOOLKIT_HOME`, then `toolkit.home` in the root `package.json`, relative to the repo.
 */
export function toolkitHome(repo: string, flag: string | undefined): string {
	const home = flag ?? process.env.TOOLKIT_HOME ?? rootManifest(repo).toolkit?.home;
	if (home === undefined || home === '') {
		throw new Error(
			"where is the toolkit's checkout? Pass --into, set TOOLKIT_HOME, or name it in package.json's toolkit.home"
		);
	}
	const dir = path.resolve(repo, home);
	if (!existsSync(path.join(dir, '.git'))) throw new Error(`${dir} is not a git checkout`);
	return dir;
}
