/**
 * A package's Claude Code files and their copies in a repo's `.claude/`, as paths and text: the
 * half of `toolkit sync` and `toolkit check` that needs no filesystem.
 *
 * A package that ships a skill, a command or an agent keeps its source; an agent reads the copy in
 * the repo's `.claude/`, which is committed. A copy can fall behind: the package is updated and the
 * copy is not, or somebody edits the copy and the next sync throws the edit away. Both are the same
 * fact, a copy that is not the source, and `check` names it. The lock file records which files a
 * package wrote and at which version, so a file a package drops is removed, and a file no package
 * wrote is the repo's own and is never touched.
 */

/** Files by their path under `.claude/` (`skills/design/SKILL.md`), each with its text. */
export type Files = Record<string, string>;

/** What one package wrote: its version then, and the files. */
export type LockEntry = { version: string; files: string[] };

/** `.claude/toolkit-lock.json`: what each package wrote. */
export type Lock = { version: 1; packages: Record<string, LockEntry> };

/** Where the lock is kept, under the repo. */
export const LOCK_FILE = '.claude/toolkit-lock.json';

/** The one file of a skill that is the repo's own. A sync never writes it. */
export const PROJECT_FILE = 'PROJECT.md';

export const EMPTY_LOCK: Lock = { version: 1, packages: {} };

/** A lock as the file's text. */
export function lockText(lock: Lock): string {
	const packages = Object.fromEntries(
		Object.entries(lock.packages).sort(([a], [b]) => a.localeCompare(b))
	);
	return `${JSON.stringify({ version: 1, packages }, null, '\t')}\n`;
}

/** The lock a file holds; an empty lock when there is no file. A file that is not one throws. */
export function parseLock(text: string | undefined): Lock {
	if (text === undefined) return EMPTY_LOCK;

	const { version, packages } = JSON.parse(text) as Partial<Lock>;
	const whole =
		version === 1 &&
		typeof packages === 'object' &&
		packages !== null &&
		Object.values(packages).every(
			entry =>
				typeof entry.version === 'string' &&
				Array.isArray(entry.files) &&
				entry.files.every(file => typeof file === 'string')
		);
	if (!whole) throw new Error(`${LOCK_FILE} is not a toolkit lock`);

	return { version, packages };
}

/**
 * What the copy has to do to become the source.
 *
 * `remove` holds only what the package wrote before and has since dropped: a file it never wrote
 * is the repo's own and stays.
 * @param source - The package's files
 * @param copy - The same paths as they are in `.claude/`, those that exist
 * @param written - What the package wrote last time, from the lock
 */
export function syncPlan(
	source: Files,
	copy: Files,
	written: readonly string[]
): { write: string[]; same: string[]; remove: string[] } {
	const own = Object.keys(source).find(name => name.split('/').at(-1) === PROJECT_FILE);
	if (own !== undefined) {
		throw new Error(`the source holds ${own}, which is each repo's own file`);
	}

	const names = Object.keys(source).sort();

	return {
		write: names.filter(name => copy[name] !== source[name]),
		same: names.filter(name => copy[name] === source[name]),
		remove: written.filter(name => !(name in source) && name in copy).sort(),
	};
}

/**
 * Why a package's copy is not its files as they stand. Empty when it is.
 * @param pkg - The installed package's name and version
 * @param source - Its files
 * @param copy - The same paths as they are in `.claude/`, with what the lock says it wrote
 * @param entry - The lock's entry for it
 */
export function staleness(
	pkg: { name: string; version: string },
	source: Files,
	copy: Files,
	entry: LockEntry | undefined
): string[] {
	if (entry === undefined) return [`${pkg.name} was never synced`];

	const { write, remove } = syncPlan(source, copy, entry.files);

	return [
		...(entry.version === pkg.version
			? []
			: [`${pkg.name} was synced at ${entry.version}, ${pkg.version} is installed`]),
		...write.map(name =>
			name in copy ? `${name} differs from ${pkg.name}’s` : `${name} is missing`
		),
		...remove.map(name => `${name} is no longer part of ${pkg.name}`),
	];
}
