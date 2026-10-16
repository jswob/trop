/**
 * `toolkit sync` and `toolkit check` on disk: which installed packages ship Claude Code files, what
 * those files are, and the repo's copies of them in `.claude/`.
 *
 * A package declares its files in its manifest's `claude` field:
 *
 *     "claude": { "skills": { "design": "skill" }, "commands": "claude/commands" }
 *
 * A skill's folder is copied whole to `.claude/skills/<name>/`, except a `PROJECT.md`, which is the
 * repo's. Every `.md` file in a commands, agents or rules folder is copied to `.claude/commands/`,
 * `.claude/agents/` or `.claude/rules/`. Only packages the repo depends on are read: an app gets a package's files by
 * installing it.
 */
import { existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import path from 'node:path';

import {
	type Files,
	type Lock,
	LOCK_FILE,
	lockText,
	parseLock,
	staleness,
	syncPlan,
} from './assets';
import { readManifest, resolvePackage, rootManifest, workspaceDirs } from './repo';

/** What a manifest's `claude` field declares, as folders in the package. */
export type Declared = {
	skills?: Record<string, string>;
	commands?: string;
	agents?: string;
	rules?: string;
};

/** An installed package that ships Claude Code files. */
export type Installed = { name: string; version: string; root: string; claude: Declared };

/** What a sync did for one package. */
export type Synced = {
	name: string;
	version: string;
	write: string[];
	same: string[];
	remove: string[];
};

const SKIPPED = new Set(['__pycache__', 'node_modules']);

/** Every file in `dir` and below, keyed `<prefix>/<path in dir>`. Dotfiles are left out. */
function filesUnder(dir: string, prefix: string): Record<string, string> {
	const files: Record<string, string> = {};
	for (const entry of readdirSync(dir, { withFileTypes: true })) {
		if (entry.name.startsWith('.') || SKIPPED.has(entry.name)) continue;
		const full = path.join(dir, entry.name);
		const key = `${prefix}/${entry.name}`;
		if (entry.isDirectory()) Object.assign(files, filesUnder(full, key));
		else if (entry.isFile()) files[key] = full;
	}
	return files;
}

/** The `.md` files directly in `dir`, keyed `<prefix>/<name>`. */
function markdownIn(dir: string, prefix: string): Record<string, string> {
	return Object.fromEntries(
		readdirSync(dir, { withFileTypes: true })
			.filter(entry => entry.isFile() && entry.name.endsWith('.md'))
			.map(entry => [`${prefix}/${entry.name}`, path.join(dir, entry.name)])
	);
}

/**
 * Every package the repo's manifests depend on whose manifest declares Claude Code files.
 * @param repo - The repo's root
 */
export function installedPackages(repo: string): Installed[] {
	const root = rootManifest(repo);

	const found = new Map<string, Installed>();
	for (const dir of [repo, ...workspaceDirs(repo, root)]) {
		const manifest = dir === repo ? root : readManifest(dir);
		if (manifest === null) continue;
		const names = Object.keys({ ...manifest.dependencies, ...manifest.devDependencies });
		for (const name of names) {
			if (found.has(name)) continue;
			const pkgRoot = resolvePackage(repo, dir, name);
			const pkg = pkgRoot === null ? null : readManifest(pkgRoot);
			if (pkgRoot === null || pkg?.claude === undefined) continue;
			found.set(name, { name, version: pkg.version ?? '0.0.0', root: pkgRoot, claude: pkg.claude });
		}
	}
	return [...found.values()].sort((a, b) => a.name.localeCompare(b.name));
}

/**
 * Where each of a package's Claude Code files is in the package, keyed by its path under `.claude/`.
 * @param pkg - The installed package
 */
export function sourcePaths(
	pkg: Pick<Installed, 'name' | 'root' | 'claude'>
): Record<string, string> {
	const folder = (dir: string): string => {
		const full = path.join(pkg.root, dir);
		if (!existsSync(full)) throw new Error(`${pkg.name} declares ${dir}, which it does not ship`);
		return full;
	};

	return {
		...Object.fromEntries(
			Object.entries(pkg.claude.skills ?? {}).flatMap(([name, dir]) =>
				Object.entries(filesUnder(folder(dir), `skills/${name}`))
			)
		),
		...(pkg.claude.commands ? markdownIn(folder(pkg.claude.commands), 'commands') : {}),
		...(pkg.claude.agents ? markdownIn(folder(pkg.claude.agents), 'agents') : {}),
		...(pkg.claude.rules ? markdownIn(folder(pkg.claude.rules), 'rules') : {}),
	};
}

/**
 * A package's Claude Code files, keyed by their path under `.claude/`.
 * @param pkg - The installed package
 */
export function sourceOf(pkg: Pick<Installed, 'name' | 'root' | 'claude'>): Files {
	return Object.fromEntries(
		Object.entries(sourcePaths(pkg)).map(([name, file]) => [name, readFileSync(file, 'utf8')])
	);
}

/** The repo's copies of `paths`, those that exist. */
export function copyOf(repo: string, paths: Iterable<string>): Files {
	const files: Files = {};
	for (const name of paths) {
		const file = path.join(repo, '.claude', name);
		if (existsSync(file)) files[name] = readFileSync(file, 'utf8');
	}
	return files;
}

export function readLock(repo: string): Lock {
	const file = path.join(repo, LOCK_FILE);
	return parseLock(existsSync(file) ? readFileSync(file, 'utf8') : undefined);
}

/** Each package with its files; two packages that would write one path throw. */
function sources(repo: string, only: readonly string[] | undefined) {
	const owner = new Map<string, string>();
	return installedPackages(repo)
		.filter(pkg => only === undefined || only.includes(pkg.name))
		.map(pkg => {
			const source = sourceOf(pkg);
			for (const name of Object.keys(source)) {
				const other = owner.get(name);
				if (other !== undefined) throw new Error(`${other} and ${pkg.name} both ship ${name}`);
				owner.set(name, pkg.name);
			}
			return { pkg, source };
		});
}

/**
 * Copy every installed package's Claude Code files into `.claude/` and record them in the lock.
 * Files a package dropped are removed, and so are the files of a package no longer installed.
 * @param repo - The repo's root
 * @param only - Sync these packages alone, leaving the rest of the lock as it is
 */
export function sync(
	repo: string,
	only?: readonly string[]
): { synced: Synced[]; uninstalled: { name: string; removed: string[] }[] } {
	const lock = readLock(repo);
	const next: Lock = { version: 1, packages: { ...lock.packages } };
	const write = (name: string, text: string | null) => {
		const file = path.join(repo, '.claude', name);
		if (text === null) {
			rmSync(file, { force: true });
			return;
		}
		mkdirSync(path.dirname(file), { recursive: true });
		writeFileSync(file, text, 'utf8');
	};

	const all = sources(repo, only);
	// A file that moved from one package to another is the new one's, and is never removed.
	const shipped = new Set(all.flatMap(({ source }) => Object.keys(source)));

	const synced = all.map(({ pkg, source }): Synced => {
		const written = (lock.packages[pkg.name]?.files ?? []).filter(
			name => name in source || !shipped.has(name)
		);
		const plan = syncPlan(source, copyOf(repo, [...Object.keys(source), ...written]), written);
		for (const name of plan.write) write(name, source[name] ?? '');
		for (const name of plan.remove) write(name, null);
		next.packages[pkg.name] = { version: pkg.version, files: Object.keys(source).sort() };
		return { name: pkg.name, version: pkg.version, ...plan };
	});

	const installed = new Set(synced.map(s => s.name));
	const uninstalled =
		only !== undefined
			? []
			: Object.entries(lock.packages)
					.filter(([name]) => !installed.has(name))
					.map(([name, entry]) => {
						const removed = Object.keys(
							copyOf(
								repo,
								entry.files.filter(file => !shipped.has(file))
							)
						);
						for (const file of removed) write(file, null);
						delete next.packages[name];
						return { name, removed };
					});

	mkdirSync(path.join(repo, '.claude'), { recursive: true });
	writeFileSync(path.join(repo, LOCK_FILE), lockText(next), 'utf8');
	return { synced, uninstalled };
}

/**
 * Why the repo's copies are not the installed packages' files. Empty when they are.
 * @param repo - The repo's root
 * @param only - Check these packages alone
 */
export function check(repo: string, only?: readonly string[]): string[] {
	const lock = readLock(repo);
	const checked = sources(repo, only);
	const names = new Set(checked.map(({ pkg }) => pkg.name));
	const shipped = new Set(checked.flatMap(({ source }) => Object.keys(source)));

	return [
		...checked.flatMap(({ pkg, source }) => {
			const entry = lock.packages[pkg.name];
			const copy = copyOf(repo, [...Object.keys(source), ...(entry?.files ?? [])]);
			// What moved to another package is that one's to answer for.
			for (const name of Object.keys(copy)) {
				if (!(name in source) && shipped.has(name)) delete copy[name];
			}
			return staleness(pkg, source, copy, entry);
		}),
		...(only !== undefined
			? []
			: Object.keys(lock.packages)
					.filter(name => !names.has(name))
					.map(name => `${name} is no longer installed, but its files are`)),
	];
}
