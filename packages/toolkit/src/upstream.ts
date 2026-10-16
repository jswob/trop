/**
 * `toolkit upstream`: an app's patches, taken home. Each patch of a package that lives in the
 * toolkit's checkout becomes a branch there with a changeset, and with `push` a pull request.
 *
 * The branch is one per app and package (`upstream/<app>/<package>`), made again from the base each
 * run, so running twice changes nothing and a patch made again on a newer version updates the same
 * pull request. The work happens in a worktree of its own: the checkout's own branch and files are
 * never touched. A patch is replayed with `git apply --3way`; Bun's patches record the blobs they
 * were made against, so a base that moved on still merges. A conflict is committed with its markers
 * and the pull request opens as a draft that says so: somebody, or Claude, resolves it there.
 */
import { spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

import { splitKey } from './patches';
import { packagesOf, readManifest, rootManifest } from './repo';

/** Runs a command and says how it went; it never throws on a failing exit. */
export type Exec = (
	command: string,
	args: string[],
	cwd: string
) => { status: number; stdout: string; stderr: string };

export const exec: Exec = (command, args, cwd) => {
	const result = spawnSync(command, args, { cwd, encoding: 'utf8' });
	return { status: result.status ?? 1, stdout: result.stdout, stderr: result.stderr };
};

export type Bump = 'patch' | 'minor' | 'major';

export type UpstreamOptions = {
	/** The toolkit's checkout: a git repo whose workspaces hold the packages. */
	into: string;
	/** The branch the pull requests go to. */
	base?: string;
	/**
	 * How far the changeset bumps; while a package is `0.x`, a minor is breaking. By default, what the
	 * app's commit that last changed the patch says it is (`bumpFor`), so a run by hand and the
	 * app's workflow write the same changeset.
	 */
	bump?: Bump;
	/** What the changeset and the commit say: the app's commit that last changed the patch, by default. */
	message?: string;
	/** Where the fix came from: a link to that commit, by default. */
	source?: string;
	/** Run each package's tests on the branch. */
	test?: boolean;
	/** Push the branch and open or update its pull request. */
	push?: boolean;
	/** Only these packages. */
	only?: readonly string[];
};

export type Upstreamed = {
	name: string;
	patch: string;
	/**
	 * `skipped`: not a package of the checkout. `already`: the base has the fix. `committed`: on the
	 * branch, not pushed. `unchanged`: the pushed branch already holds it. `pushed`: pushed now.
	 */
	state: 'skipped' | 'already' | 'committed' | 'unchanged' | 'pushed';
	branch?: string;
	bump?: Bump;
	conflicts?: string[];
	tests?: 'passed' | 'failed' | 'none' | 'skipped';
	pr?: string;
};

/** A name fit for a branch or a file: the app's or the package's, without the scope. */
export function slug(name: string): string {
	return name
		.replace(/^@[^/]+\//, '')
		.replace(/[^A-Za-z0-9._-]+/g, '-')
		.replace(/^-+|-+$/g, '');
}

export function branchName(app: string, pkg: string): string {
	return `upstream/${slug(app)}/${slug(pkg)}`;
}

/** A commit subject without its conventional type: `fix(kit): two` says `two`. */
export function withoutType(subject: string): string {
	return subject.replace(/^[a-z]+(\([^)]*\))?!?:\s*/i, '');
}

/**
 * How far a conventional commit bumps a package at `version`. A breaking change (`type!:`, or a
 * `BREAKING CHANGE:` footer) is a major, or a minor while the package is `0.x`, where a minor is
 * what a caret range does not take. A `feat` is a minor from 1.0 on. Anything else is a patch.
 */
export function bumpFor(message: string, version: string): Bump {
	const [subject = '', ...body] = message.split('\n');
	const early = version.startsWith('0.');
	if (/^[a-z]+(\([^)]*\))?!:/i.test(subject) || /^BREAKING[ -]CHANGE:/m.test(body.join('\n'))) {
		return early ? 'minor' : 'major';
	}
	return /^feat(\([^)]*\))?:/i.test(subject) && !early ? 'minor' : 'patch';
}

/**
 * The conventional header of the commit and the pull request in the checkout: the app's commit's
 * type (`fix` when it has none), the package as the scope, and `!` when the bump is breaking.
 */
export function commitHeader(subject: string, name: string, bump: Bump, version: string): string {
	const type = /^([a-z]+)(\([^)]*\))?!?:/i.exec(subject)?.[1] ?? 'fix';
	const breaking = bump === 'major' || (bump === 'minor' && version.startsWith('0.'));
	return `${type}(${slug(name)})${breaking ? '!' : ''}`;
}

/**
 * What an app must change for a breaking commit: its `BREAKING CHANGE:` footer, up to the next
 * trailer. `undefined` when it has none. It goes into the changeset, and so into the changelog, the
 * release and Renovate's pull request in every app.
 */
export function migrationNote(message: string): string | undefined {
	const lines = message.split('\n');
	const at = lines.findIndex(line => /^BREAKING[ -]CHANGE:/.test(line));
	if (at < 0) return undefined;
	const end = lines.findIndex((line, index) => index > at && /^[A-Za-z][\w-]*(: | #)/.test(line));
	const note = [
		(lines[at] ?? '').replace(/^BREAKING[ -]CHANGE:\s*/, ''),
		...lines.slice(at + 1, end < 0 ? undefined : end),
	]
		.join('\n')
		.trim();
	return note === '' ? undefined : note;
}

export function changesetText(
	name: string,
	bump: Bump,
	message: string,
	migration?: string
): string {
	const body = migration === undefined ? message : `${message}\n\nBreaking: ${migration}`;
	return `---\n'${name}': ${bump}\n---\n\n${body}\n`;
}

/** An app's commit as a link, from its `origin` when that is on GitHub; the bare commit otherwise. */
export function commitLink(remote: string, commit: string): string {
	const match = /github\.com[:/]([^/]+\/[^/]+?)(?:\.git)?$/.exec(remote.trim());
	return match ? `https://github.com/${match[1] ?? ''}/commit/${commit}` : commit;
}

export function prBody(o: {
	app: string;
	name: string;
	patch: string;
	source?: string;
	migration?: string;
	conflicts: string[];
	tests: Upstreamed['tests'];
}): string {
	const lines = [
		`A fix made in **${o.app}** as a patch of \`${o.name}\` (\`${o.patch}\`), taken home by \`toolkit upstream\`.`,
		'',
		...(o.source === undefined ? [] : [`From: ${o.source}`, '']),
		...(o.migration === undefined
			? []
			: ['**Breaking.** What an app must change:', '', o.migration, '']),
	];
	if (o.conflicts.length > 0) {
		lines.push(
			'**The patch did not merge cleanly.** These files hold conflict markers to resolve on this branch:',
			'',
			...o.conflicts.map(file => `- \`${file}\``),
			''
		);
	}
	if (o.tests === 'failed') lines.push("**The package's tests failed on this branch.**", '');
	if (o.tests === 'passed') lines.push("The package's tests passed on this branch.", '');
	lines.push(
		"Once this is released, Renovate's branch in each app takes the version, and the app's toolkit sync deletes its patch (`toolkit carry`)."
	);
	return `${lines.join('\n')}\n`;
}

/**
 * Take the app's patches home.
 * @param repo - The app's root
 * @param options - Where home is, and what to do there
 * @param run - Runs git, gh and bun
 */
export function upstream(repo: string, options: UpstreamOptions, run: Exec = exec): Upstreamed[] {
	const into = path.resolve(options.into);
	const base = options.base ?? 'master';
	const app = rootManifest(repo).name ?? path.basename(repo);
	const patched = rootManifest(repo).patchedDependencies ?? {};
	const packages = packagesOf(into);

	const git = (cwd: string, ...args: string[]) => {
		const result = run('git', args, cwd);
		if (result.status !== 0) {
			throw new Error(`git ${args.join(' ')} failed in ${cwd}:\n${result.stderr.trim()}`);
		}
		return result.stdout.trim();
	};

	const remote = run('git', ['remote', 'get-url', 'origin'], repo);
	/** The app's commit that last changed the patch: its link, and its message. */
	const madeIn = (patch: string): { source?: string; message?: string } => {
		const log = run('git', ['log', '-1', '--format=%H%n%B', '--', patch], repo);
		const [commit, ...message] = log.status === 0 ? log.stdout.trim().split('\n') : [];
		if (commit === undefined || commit === '') return {};
		return {
			source: remote.status === 0 ? commitLink(remote.stdout, commit) : commit,
			...(message.length === 0 ? {} : { message: message.join('\n') }),
		};
	};

	const hasOrigin = run('git', ['remote', 'get-url', 'origin'], into).status === 0;
	if (hasOrigin) git(into, 'fetch', '--quiet', 'origin', base);
	const baseRef = hasOrigin ? `origin/${base}` : base;

	return Object.entries(patched)
		.filter(([key]) => options.only === undefined || options.only.includes(splitKey(key).name))
		.map(([key, patch]): Upstreamed => {
			const { name } = splitKey(key);
			const home = packages.get(name);
			if (home === undefined) return { name, patch, state: 'skipped' };
			const dir = path.relative(into, home);

			const branch = branchName(app, name);
			const made = madeIn(patch);
			const source = options.source ?? made.source;
			const message =
				options.message ??
				(made.message === undefined
					? `A fix made in ${app} (${path.basename(patch)}).`
					: withoutType(made.message.split('\n')[0] ?? ''));
			const tree = mkdtempSync(path.join(tmpdir(), 'toolkit-upstream-'));
			rmSync(tree, { recursive: true });
			git(into, 'worktree', 'add', '--quiet', '--force', '-B', branch, tree, baseRef);

			try {
				const applied = run(
					'git',
					['apply', '--3way', `--directory=${dir}`, path.resolve(repo, patch)],
					tree
				);
				const conflicts = git(tree, 'diff', '--name-only', '--diff-filter=U')
					.split('\n')
					.filter(file => file !== '');
				if (applied.status !== 0 && conflicts.length === 0) {
					throw new Error(`${patch} does not apply to ${dir}:\n${applied.stderr.trim()}`);
				}
				git(tree, 'add', '--all');
				if (git(tree, 'status', '--porcelain') === '') {
					return { name, patch, state: 'already', branch };
				}

				const version = readManifest(path.join(tree, dir))?.version ?? '0.0.0';
				const bump = options.bump ?? bumpFor(made.message ?? '', version);
				const header = commitHeader(made.message ?? '', name, bump, version);
				const migration = migrationNote(made.message ?? '');

				mkdirSync(path.join(tree, '.changeset'), { recursive: true });
				writeFileSync(
					path.join(tree, '.changeset', `upstream-${slug(app)}-${slug(name)}.md`),
					changesetText(name, bump, message, migration),
					'utf8'
				);

				let tests: Upstreamed['tests'] = 'skipped';
				if (options.test !== false && conflicts.length === 0) {
					if (readManifest(path.join(tree, dir))?.scripts?.test === undefined) tests = 'none';
					else {
						const installed = run('bun', ['install', '--frozen-lockfile'], tree);
						const tested =
							installed.status === 0
								? run('bun', ['run', 'test', '--run'], path.join(tree, dir))
								: installed;
						tests = tested.status === 0 ? 'passed' : 'failed';
					}
				}

				git(tree, 'add', '--all');
				git(
					tree,
					// The hooks would lint a branch that may hold conflict markers; its pull request's CI does.
					'commit',
					'--quiet',
					'--no-verify',
					'-m',
					`${header}: ${message}`,
					'-m',
					[`From ${app}${source === undefined ? '' : `: ${source}`}`, `Patch: ${patch}`].join('\n')
				);

				if (options.push !== true)
					return { name, patch, state: 'committed', branch, bump, conflicts, tests };

				const fetched = run('git', ['fetch', '--quiet', 'origin', branch], tree);
				const pushedTree = fetched.status === 0 ? git(tree, 'rev-parse', 'FETCH_HEAD^{tree}') : '';
				const unchanged = pushedTree !== '' && pushedTree === git(tree, 'rev-parse', 'HEAD^{tree}');
				if (!unchanged)
					git(tree, 'push', '--quiet', '--force', 'origin', `HEAD:refs/heads/${branch}`);

				const pr = openPullRequest(run, tree, {
					base,
					branch,
					title: `${conflicts.length > 0 ? '[conflict] ' : ''}${header}: ${message}`,
					body: prBody({ app, name, patch, source, migration, conflicts, tests }),
					draft: conflicts.length > 0 || tests === 'failed',
				});
				return {
					name,
					patch,
					state: unchanged ? 'unchanged' : 'pushed',
					branch,
					bump,
					conflicts,
					tests,
					pr,
				};
			} finally {
				run('git', ['worktree', 'remove', '--force', tree], into);
			}
		});
}

/** Open the branch's pull request, or bring the open one up to date. Returns its address. */
function openPullRequest(
	run: Exec,
	cwd: string,
	pr: { base: string; branch: string; title: string; body: string; draft: boolean }
): string {
	const gh = (...args: string[]) => {
		const result = run('gh', args, cwd);
		if (result.status !== 0)
			throw new Error(`gh ${args[0] ?? ''} ${args[1] ?? ''} failed:\n${result.stderr.trim()}`);
		return result.stdout.trim();
	};

	const open = JSON.parse(
		gh('pr', 'list', '--head', pr.branch, '--state', 'open', '--json', 'number,url,isDraft')
	) as { number: number; url: string; isDraft: boolean }[];
	const current = open[0];

	if (current === undefined) {
		return (
			gh(
				'pr',
				'create',
				'--base',
				pr.base,
				'--head',
				pr.branch,
				'--title',
				pr.title,
				'--body',
				pr.body,
				...(pr.draft ? ['--draft'] : [])
			)
				.split('\n')
				.at(-1) ?? ''
		);
	}

	const number = String(current.number);
	gh('pr', 'edit', number, '--title', pr.title, '--body', pr.body);
	if (pr.draft && !current.isDraft) gh('pr', 'ready', number, '--undo');
	if (!pr.draft && current.isDraft) gh('pr', 'ready', number);
	return current.url;
}
