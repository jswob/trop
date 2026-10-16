/**
 * The toolkit's Claude Code hooks, which a repo wires in its `.claude/settings.json`.
 *
 * `guard` (PreToolUse, on the edit tools) refuses an edit an install or a sync would undo: a file
 * in an installed copy of a package, and a package's copy in `.claude/`, unless that package is
 * being patched (`toolkit patch`). It says what to do instead. A copy whose package is the repo's
 * own source, or linked to a checkout, points at that source.
 *
 * `notice` (SessionStart) tells a new session what is half done or out of step: a package being
 * patched, a link to the checkout, a stale patch or copy, and Renovate's update of the toolkit when
 * it waits for a review, as a breaking release does.
 */
import { spawnSync } from 'node:child_process';
import { existsSync, realpathSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';

import { check, readLock, sourcePaths } from './claude';
import { readLinks } from './link';
import { readPatching, staleProblems } from './patches';
import { installedCopies, readManifest } from './repo';

/** What a PreToolUse hook is handed on stdin, as far as the guard reads it. */
export type ToolCall = {
	tool_name?: string;
	tool_input?: { file_path?: string; notebook_path?: string };
	cwd?: string;
};

const EDITS = new Set(['Edit', 'Write', 'MultiEdit', 'NotebookEdit']);

function inside(dir: string, file: string): boolean {
	const relative = path.relative(dir, file);
	return relative === '' || (relative.split(path.sep)[0] !== '..' && !path.isAbsolute(relative));
}

/** A path with its deepest existing folder resolved, so a new file is placed where it would land. */
function resolved(file: string): string {
	const rest: string[] = [];
	let dir = file;
	while (!existsSync(dir) && path.dirname(dir) !== dir) {
		rest.unshift(path.basename(dir));
		dir = path.dirname(dir);
	}
	return path.join(realpathSync(dir), ...rest);
}

/** The repo a file is in: the nearest folder up from it with a `.git`. */
function repoOf(file: string): string | null {
	for (let dir = path.dirname(file); ; dir = path.dirname(dir)) {
		if (existsSync(path.join(dir, '.git'))) return dir;
		if (path.dirname(dir) === dir) return null;
	}
}

/** The package an installed file belongs to: the name after the path's last `node_modules`. */
function installedPackage(file: string): string | null {
	const parts = file.split(path.sep);
	const at = parts.lastIndexOf('node_modules');
	const first = parts[at + 1];
	if (at < 0 || first === undefined || first.startsWith('.')) return null;
	if (!first.startsWith('@')) return first;
	const second = parts[at + 2];
	return second === undefined ? null : `${first}/${second}`;
}

const SKILL = 'the upstream skill has the rest';

/** Why an edit to an installed copy is refused, or `null` when it is the copy being patched. */
function installedEdit(repo: string, file: string, name: string): string | null {
	const shown = path.relative(repo, file);
	const entry = readPatching(repo)[name];
	if (entry === undefined) {
		return (
			`${shown} is in the installed copy of ${name}: the next install puts it back, and nothing ships the edit. ` +
			`To fix ${name} in this repo, run bun toolkit patch ${name}, then edit it at node_modules/${name} (${SKILL}).`
		);
	}
	return inside(resolved(path.join(repo, entry.dir)), file)
		? null
		: `${name} is being patched at ${entry.dir}, not at ${shown}: edit it there.`;
}

/** Why an edit to a package's copy in `.claude/` is refused, or `null` when it may be made. */
function copyEdit(repo: string, copy: string): string | null {
	const owner = Object.entries(readLock(repo).packages).find(([, entry]) =>
		entry.files.includes(copy)
	);
	if (owner === undefined) return null;
	const [name] = owner;
	if (readPatching(repo)[name] !== undefined) return null;

	const root = installedCopies(repo, name)[0]?.root;
	const claude = root === undefined ? undefined : readManifest(root)?.claude;
	let source: string | undefined;
	try {
		source = root && claude ? sourcePaths({ name, root, claude })[copy] : undefined;
	} catch {
		source = undefined;
	}
	const own = root !== undefined && !root.split(path.sep).includes('node_modules');
	if (own && source !== undefined) {
		return (
			`.claude/${copy} is a copy of ${path.relative(realpathSync(repo), source)}, which ${name}'s source holds: ` +
			'edit that, then bun toolkit sync.'
		);
	}
	return (
		`.claude/${copy} is ${name}'s copy: bun toolkit sync writes it again, and bun toolkit check fails on an edit. ` +
		`To change it here, run bun toolkit patch ${name}, edit this copy, then bun toolkit patch ${name} --commit (${SKILL}).`
	);
}

/**
 * Why an edit is refused, or `null` when it may go ahead.
 * @param call - The tool call, as the hook is handed it
 */
export function guard(call: ToolCall): string | null {
	if (!EDITS.has(call.tool_name ?? '')) return null;
	const given = call.tool_input?.file_path ?? call.tool_input?.notebook_path;
	if (given === undefined || given === '') return null;

	const file = resolved(path.resolve(call.cwd ?? process.cwd(), given));
	const found = repoOf(file);
	if (found === null) return null;
	const repo = realpathSync(found);

	const name = installedPackage(path.relative(repo, file));
	if (name !== null) return installedEdit(repo, file, name);

	const claude = path.join(repo, '.claude');
	if (!inside(claude, file)) return null;
	return copyEdit(repo, path.relative(claude, file).split(path.sep).join('/'));
}

/** Renovate's open pull request that updates the toolkit's packages, when it waits for a review. */
export type Waiting = { number: number; title: string; url: string };

/** An open pull request, as `gh pr list --json` gives it. */
export type OpenPull = Waiting & { headRefName: string; body: string };

/**
 * Renovate's toolkit pull request among the open ones, when it waits for a review. The presets
 * group the packages as `toolkit`, so the branch is `renovate/toolkit`, or `renovate/major-toolkit`
 * for a major update, which is the one that waits.
 * @param pulls - The repo's open pull requests
 */
export function waitingAmong(pulls: OpenPull[]): Waiting | null {
	// Renovate's body says whether it merges the PR itself; one it does not waits for a review.
	const pull = pulls.find(
		candidate =>
			/^renovate\/(?:[\w.-]+-)?toolkit$/.test(candidate.headRefName) &&
			/\*\*Automerge\*\*: Disabled/.test(candidate.body)
	);
	return pull === undefined ? null : { number: pull.number, title: pull.title, url: pull.url };
}

/** Asks GitHub, through `gh`, for Renovate's toolkit pull request; `null` without one, or offline. */
export function waitingUpdate(repo: string): Waiting | null {
	const listed = spawnSync(
		'gh',
		[
			'pr',
			'list',
			'--state',
			'open',
			'--limit',
			'50',
			'--json',
			'number,title,url,headRefName,body',
		],
		{ cwd: repo, encoding: 'utf8', timeout: 5000 }
	);
	if (listed.status !== 0) return null;
	try {
		return waitingAmong(JSON.parse(listed.stdout) as OpenPull[]);
	} catch {
		return null;
	}
}

/**
 * What a new session should know before it starts, one line each. Empty when all is in order.
 * @param repo - The repo's root
 * @param waiting - Renovate's toolkit pull request when it waits for a review
 */
export function notice(repo: string, waiting: Waiting | null = null): string[] {
	const patching = Object.entries(readPatching(repo)).map(
		([name, entry]) =>
			`${name} is being patched: edit it at ${entry.dir} (or its copies in .claude/), then bun toolkit patch ${name} --commit, or --abort.`
	);
	const links = Object.entries(readLinks(repo)).map(
		([name, entry]) =>
			`${name} runs on the toolkit's checkout, ${entry.dir}: bun toolkit unlink before committing.`
	);
	const patches = staleProblems(repo).map(problem => `A patch is out of step: ${problem}.`);
	const copies = check(repo);
	const stale =
		copies.length === 0
			? []
			: [`The copies in .claude/ are out of step (bun toolkit sync): ${copies.join('; ')}.`];
	const update =
		waiting === null
			? []
			: [
					`Renovate's update of the toolkit waits for a review: #${String(waiting.number)} ${waiting.title} (${waiting.url}). ` +
						`A breaking release says in its notes what to change: gh pr view ${String(waiting.number)}.`,
				];
	return [...patching, ...links, ...patches, ...stale, ...update];
}
