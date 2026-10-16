#!/usr/bin/env bun
/**
 * `toolkit <command>`, run from anywhere in the repo.
 *
 *   sync      copy every installed package's Claude Code files into .claude/, stamped in the lock
 *   check     fail when a copy is not its package's file as it stands, when a patch is stale or half
 *             made, when a package is linked, when Bun is not pinned or a dependency is a folder, or
 *             when a package that must not know the app holds one of its words
 *   patch     fix an installed package in place, as Bun's patch
 *   carry     take the patches to the versions installed: delete the fixed, make again the rest
 *   upstream  take the app's patches home, as branches and pull requests in the toolkit's checkout
 *   link      run the app on the toolkit's checkout instead of the installed packages; unlink
 *   try       the app's tests on what it installs, then on the checkout: only new failures fail
 *   guard     a PreToolUse hook: refuse an edit an install or a sync would undo
 *   notice    a SessionStart hook: what is half done or out of step
 */
import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { parseArgs } from 'node:util';

import { LOCK_FILE } from './assets';
import { check, sync } from './claude';
import { guard, notice, waitingUpdate, type ToolCall } from './hooks';
import { link, linkProblems, unlink } from './link';
import {
	abortPatch,
	carry,
	commitPatch,
	patchProblems,
	startPatch,
	type PatchStanding,
} from './patches';
import { pinProblems } from './pins';
import { toolkitHome } from './repo';
import { tryPackages } from './try';
import { type Bump, upstream } from './upstream';
import { appWords } from './words';

const USAGE = `toolkit <command>, or toolkit <command> --help for this text

  sync                       Copy every installed package's skills, commands, agents and rules
                             into .claude/, and record them in ${LOCK_FILE}. A skill's PROJECT.md
                             is never written.
  check                      Fail when a copy differs from its package's file, or was never
                             synced; when a patch was made for a version no longer installed, or
                             was left half made; when a package is linked; when packageManager
                             does not pin Bun to one version, or a dependency is a folder (link:,
                             file:); and when a path in package.json's toolkit.agnostic holds one
                             of its words.
  patch <package>            Make the installed copy of <package> editable (bun patch), at
                             node_modules/<package>.
  patch <package> --commit   Write the patch to patches/: the edits there, and the edits to the
                             package's copies in .claude/. Then sync its copies.
  patch <package> --abort    Throw the edits away.
  carry                      Take each patch made for a version no longer installed to the one
                             that is: deleted when that version has the fix (git can take the
                             patch back out), made again when it still applies. Fails when one
                             does neither.
  upstream [package...]      Take each patch of a package the toolkit's checkout holds home: a
                             branch upstream/<app>/<package> there, from origin's base, with the
                             patch replayed (a conflict is committed with its markers), a
                             changeset, and the package's tests run. The checkout's own branch
                             and files are left alone.
      --into <dir>           The toolkit's checkout (else TOOLKIT_HOME, else toolkit.home)
      --base <branch>        The branch to start from and open the pull request to (master)
      --bump <level>         patch, minor or major (on 0.x, minor is breaking). By default what
                             the app's commit says: a breaking change (type!: or BREAKING
                             CHANGE:) is major, minor on 0.x; a feat is minor from 1.0 on;
                             anything else a patch
      --message <text>       What the changeset and the commit say (the subject of the app's
                             commit that last changed the patch, without its type)
      --source <link>        Where the fix came from (that commit, as a GitHub link)
      --skip-tests           Run no tests
      --push                 Push the branch and open or update its pull request (gh)
  link [package...]          Run the app on the toolkit's checkout: each folder's
                             node_modules/<package> becomes a link to the package's folder there,
                             and its copies in .claude/ are synced from it. Every package of the
                             checkout the app installs, by default. --into as for upstream.
  unlink [package...]        Remove the links, install the packages again, and sync their copies.
  try <package...>           Run the tests of each workspace that depends on the packages, link
                             them, run the tests again, and unlink. Fails only on a test that
                             passed before. --into as for upstream.

  Hooks, for .claude/settings.json; neither ever fails:
  guard                      PreToolUse on Edit, Write, MultiEdit and NotebookEdit: refuse an
                             edit to an installed copy of a package, or to a package's copy in
                             .claude/, unless that package is being patched.
  notice                     SessionStart: say what is half done or out of step (a patch being
                             made, a link, a stale patch or copy), and whether Renovate's update
                             of the toolkit waits for a review (gh).
`;

/** The repo's root: the nearest folder up from here with a `.git`. */
function repoRoot(from: string): string {
	for (let dir = from; ; dir = path.dirname(dir)) {
		if (existsSync(path.join(dir, '.git'))) return dir;
		if (path.dirname(dir) === dir) return from;
	}
}

const out = (line: string) => process.stdout.write(`${line}\n`);

/** A list under a heading, on stderr; nothing when the list is empty. */
function problems(heading: string, lines: string[], hint?: string): void {
	if (lines.length === 0) return;
	process.stderr.write(`${heading}\n${lines.map(line => `  ${line}`).join('\n')}\n`);
	if (hint !== undefined) process.stderr.write(`${hint}\n`);
}

function patch(repo: string, args: string[]): number {
	const { values, positionals } = parseArgs({
		args,
		allowPositionals: true,
		options: { commit: { type: 'boolean' }, abort: { type: 'boolean' } },
	});
	const name = positionals[0];
	if (name === undefined || positionals.length > 1 || (values.commit && values.abort)) {
		process.stdout.write(USAGE);
		return 1;
	}

	if (values.abort) {
		abortPatch(repo, name);
		out(`${name}: the edits are thrown away, and the package is installed again.`);
		return 0;
	}

	if (values.commit) {
		const { patch: file, adopted, synced } = commitPatch(repo, name);
		for (const copy of adopted) out(`  took     .claude/${copy}`);
		for (const { write } of synced) for (const copy of write) out(`  synced   .claude/${copy}`);
		out(
			file === null
				? `${name}: nothing was changed, so there is no patch.`
				: `${name}: ${file}. Commit it with package.json and bun.lock; toolkit upstream takes it home.`
		);
		return 0;
	}

	const { dir, started } = startPatch(repo, name);
	out(
		`${name} ${started ? 'is' : 'was already'} editable at ${dir}. Edit it there, or edit its copies in .claude/,\n` +
			`then: bun toolkit patch ${name} --commit`
	);
	return 0;
}

/** The guard's answer, as Claude Code reads a PreToolUse hook's: nothing lets the edit through. */
function guardCommand(): number {
	try {
		const reason = guard(JSON.parse(readFileSync(0, 'utf8')) as ToolCall);
		if (reason !== null) {
			out(
				JSON.stringify({
					hookSpecificOutput: {
						hookEventName: 'PreToolUse',
						permissionDecision: 'deny',
						permissionDecisionReason: reason,
					},
				})
			);
		}
	} catch {
		// A guard that cannot read the call lets it through: it never stands in the way of a session.
	}
	return 0;
}

/** What a new session is told; Claude Code adds a SessionStart hook's output to the context. */
function noticeCommand(repo: string): number {
	try {
		const root = process.env.CLAUDE_PROJECT_DIR ?? repo;
		const lines = notice(root, waitingUpdate(root));
		if (lines.length > 0) out(['The toolkit says:', ...lines.map(line => `- ${line}`)].join('\n'));
	} catch {
		// Nothing to say is better than a session that does not start.
	}
	return 0;
}

function carryCommand(repo: string): number {
	const { dropped, carried, stuck } = carry(repo);
	const at = (patch: PatchStanding) => `${patch.name}@${patch.installed.join(', ')}`;
	for (const patch of dropped) out(`  deleted  ${patch.file}: ${at(patch)} has the fix`);
	for (const { from, patch } of carried) {
		out(`  carried  ${from.file} → ${patch ?? `nothing: ${at(from)} needs none`}`);
	}
	problems(
		'These patches do not apply to the version installed:',
		stuck.map(patch => `${patch.file} (${patch.standing}, ${at(patch)})`),
		'Make each again by hand: bun toolkit patch <package>'
	);
	if (dropped.length + carried.length + stuck.length === 0) {
		out('Every patch is made for the version installed.');
	} else if (dropped.length + carried.length > 0) {
		out('Commit package.json, bun.lock, patches/ and .claude/.');
	}
	return stuck.length > 0 ? 1 : 0;
}

const BUMPS: readonly Bump[] = ['patch', 'minor', 'major'];

function upstreamCommand(repo: string, args: string[]): number {
	const { values, positionals } = parseArgs({
		args,
		allowPositionals: true,
		options: {
			into: { type: 'string' },
			base: { type: 'string' },
			bump: { type: 'string' },
			message: { type: 'string' },
			source: { type: 'string' },
			'skip-tests': { type: 'boolean' },
			push: { type: 'boolean' },
		},
	});
	const bump = BUMPS.find(level => level === values.bump);
	if (values.bump !== undefined && bump === undefined) {
		process.stdout.write(USAGE);
		return 1;
	}

	const into = toolkitHome(repo, values.into);
	const where = path.relative(repo, into) || into;
	const results = upstream(repo, {
		into,
		...(bump === undefined ? {} : { bump }),
		...(values.base === undefined ? {} : { base: values.base }),
		...(values.message === undefined ? {} : { message: values.message }),
		...(values.source === undefined ? {} : { source: values.source }),
		test: values['skip-tests'] !== true,
		push: values.push === true,
		...(positionals.length === 0 ? {} : { only: positionals }),
	});

	if (results.length === 0) out('No patches to take home.');
	for (const r of results) {
		const tests = r.tests === undefined || r.tests === 'skipped' ? '' : `, tests ${r.tests}`;
		const bump = r.bump === undefined ? '' : ` (${r.bump})`;
		switch (r.state) {
			case 'skipped':
				out(`${r.name}: not a package of ${where}, left alone`);
				break;
			case 'already':
				out(
					`${r.name}: ${where} has the fix already. Delete ${r.patch} once the app takes a version with it.`
				);
				break;
			case 'committed':
				out(
					`${r.name}${bump}: committed on ${r.branch ?? ''} in ${where}${tests}. Push it with --push.`
				);
				break;
			default:
				out(
					`${r.name}${bump}: ${r.state === 'pushed' ? 'pushed' : 'already pushed'}${tests}, ${r.pr ?? ''}`
				);
		}
		if (r.conflicts !== undefined && r.conflicts.length > 0) {
			out(`  conflicts, committed with their markers: ${r.conflicts.join(', ')}`);
		}
	}
	return 0;
}

function linkCommand(repo: string, args: string[]): number {
	const { values, positionals } = parseArgs({
		args,
		allowPositionals: true,
		options: { into: { type: 'string' } },
	});
	const home = toolkitHome(repo, values.into);
	const { linked, synced } = link(repo, home, positionals.length === 0 ? undefined : positionals);

	if (linked.length === 0) out(`The app installs no package of ${home}.`);
	for (const { name, dir, paths } of linked) {
		out(`${name} -> ${dir}`);
		for (const file of paths) out(`  linked   ${file}`);
	}
	for (const { write } of synced) for (const copy of write) out(`  synced   .claude/${copy}`);
	if (linked.length > 0) out('Put the installed packages back with: bun toolkit unlink');
	return 0;
}

function tryCommand(repo: string, args: string[]): number {
	const { values, positionals } = parseArgs({
		args,
		allowPositionals: true,
		options: { into: { type: 'string' } },
	});
	if (positionals.length === 0) {
		process.stdout.write(USAGE);
		return 1;
	}
	const results = tryPackages(repo, toolkitHome(repo, values.into), positionals);

	if (results.length === 0) out(`No workspace with tests depends on ${positionals.join(', ')}.`);
	for (const { workspace, broken, fixed, failing } of results) {
		out(
			`${workspace}: ${String(broken.length)} broken · ${String(fixed.length)} fixed · ${String(failing.length)} failing before and after`
		);
		for (const test of broken) out(`  broken   ${test}`);
		for (const test of fixed) out(`  fixed    ${test}`);
	}
	return results.some(({ broken }) => broken.length > 0) ? 1 : 0;
}

function main(argv: string[]): number {
	const repo = repoRoot(process.cwd());

	// Asked for help, no command runs: sync and carry would write
	if (argv.includes('--help') || argv.includes('-h')) {
		process.stdout.write(USAGE);
		return 0;
	}

	switch (argv[0]) {
		case 'sync': {
			const { synced, uninstalled } = sync(repo);
			for (const { name, version, write, same, remove } of synced) {
				out(`${name} ${version}`);
				for (const file of write) out(`  wrote    ${file}`);
				for (const file of remove) out(`  removed  ${file}`);
				out(
					`  ${String(write.length)} written · ${String(same.length)} already the same · ${String(remove.length)} removed`
				);
			}
			for (const { name, removed } of uninstalled) {
				out(`${name}, no longer installed`);
				for (const file of removed) out(`  removed  ${file}`);
			}
			return 0;
		}
		case 'check': {
			const reasons = check(repo);
			const patches = patchProblems(repo);
			const links = linkProblems(repo);
			const pins = pinProblems(repo);
			const hits = appWords(repo);

			problems(
				'The copies in .claude/ are stale:',
				reasons,
				"Make a change in the package's own files, or patch it (bun toolkit patch), then: bun toolkit sync"
			);
			problems('The patches are not in order:', patches);
			problems("Packages run on the toolkit's checkout, not on what the app installs:", links);
			problems('The install may not be the one CI tested:', pins);
			problems(
				"The app's words, in packages that must not know the app (toolkit.agnostic):",
				hits.map(hit => `${hit.file}:${String(hit.line)}  ${hit.word}`)
			);

			if (reasons.length + patches.length + links.length + pins.length + hits.length > 0) return 1;
			out(
				'Every copy in .claude/ is its package’s file, every patch is in order, nothing is linked, and Bun is pinned.'
			);
			return 0;
		}
		case 'patch':
			return patch(repo, argv.slice(1));
		case 'carry':
			return carryCommand(repo);
		case 'guard':
			return guardCommand();
		case 'notice':
			return noticeCommand(repo);
		case 'upstream':
			return upstreamCommand(repo, argv.slice(1));
		case 'link':
			return linkCommand(repo, argv.slice(1));
		case 'unlink': {
			const names = argv.slice(1);
			const { unlinked, synced } = unlink(repo, names.length === 0 ? undefined : names);
			for (const { write } of synced) for (const copy of write) out(`  synced   .claude/${copy}`);
			out(
				unlinked.length === 0
					? 'Nothing is linked.'
					: `${unlinked.join(', ')}: the installed ${unlinked.length === 1 ? 'package is' : 'packages are'} back.`
			);
			return 0;
		}
		case 'try':
			return tryCommand(repo, argv.slice(1));
		default:
			process.stdout.write(USAGE);
			return argv[0] === undefined || argv[0] === 'help' ? 0 : 1;
	}
}

try {
	process.exit(main(process.argv.slice(2)));
} catch (error) {
	process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
	process.exit(1);
}
