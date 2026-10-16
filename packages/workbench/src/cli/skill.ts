import path from 'node:path';
import process from 'node:process';

import { check, LOCK_FILE, sync } from '@jswob/toolkit';

import type { Argv } from './args';
import { engine, ENGINE_ROOT } from './engine';
import { host } from './host';

/** The skill's source: a folder of this package, which its manifest's `claude` field names. */
export const SKILL_SRC = path.join(ENGINE_ROOT, 'skill');

/** Where the toolkit records what it copied, in the repo. */
export const SKILL_LOCK = LOCK_FILE;

/** Why the repo's copy of the skill is stale, or nothing to say when the config names no copy. */
export function skillStaleness(): string[] | null {
	const { skill: copy, repo } = host();
	if (copy === undefined) return null;

	return check(repo, [engine().name]);
}

/**
 * `skill` — copies the design skill into the repo's `.claude/skills/`, recorded in the toolkit's
 * lock with this engine's version. It is `toolkit sync` for this package alone.
 *
 * The copy is what an agent reads, and it is committed with the repo. The project's own file of
 * it, `PROJECT.md`, is never written, and neither is any other file a copy did not put there. A
 * change to the skill is made in this package's `skill/` folder and copied again: an edit made in
 * the copy is overwritten here, and `check` fails on it until it is.
 */
export function skill(_argv: Argv): number {
	const { skill: copy, repo } = host();
	if (copy === undefined) {
		throw new Error(`the config names no folder for the skill — add skill: '<folder>' to it`);
	}

	const { synced } = sync(repo, [engine().name]);
	for (const { name, version, write, same, remove } of synced) {
		for (const file of write) process.stdout.write(`  wrote    ${file}\n`);
		for (const file of remove) process.stdout.write(`  removed  ${file}\n`);
		process.stdout.write(
			`\n  ${name} ${version} → .claude/, recorded in ${SKILL_LOCK}\n` +
				`  ${String(write.length)} written · ${String(same.length)} already the same · ${String(remove.length)} removed\n`
		);
	}

	return 0;
}
