import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import {
	branchName,
	bumpFor,
	changesetText,
	migrationNote,
	commitHeader,
	commitLink,
	exec,
	slug,
	upstream,
	withoutType,
	type Exec,
} from './upstream';

let root: string;
let home: string;
let app: string;
let calls: string[];
let testsPass: boolean;
let prs: Map<string, { number: number; url: string; isDraft: boolean }>;

const json = (value: unknown) => `${JSON.stringify(value, null, '\t')}\n`;

function put(dir: string, file: string, text: string) {
	const full = path.join(dir, file);
	mkdirSync(path.dirname(full), { recursive: true });
	writeFileSync(full, text, 'utf8');
}

function git(cwd: string, ...args: string[]): string {
	const result = exec('git', args, cwd);
	if (result.status !== 0) throw new Error(`git ${args.join(' ')}: ${result.stderr}`);
	return result.stdout.trim();
}

const KIT = ['one = 1', 'two = 2', 'pad1', 'pad2', 'pad3', 'pad4', 'pad5', 'three = 3']
	.map(line => `export const ${line.includes('=') ? line : `${line} = 0`};\n`)
	.join('');
const FIXED = KIT.replace('two = 2', 'two = 22');
const PATCH = 'patches/@acme%2Fkit@1.0.0.patch';

/** Real git; Bun runs the tests as `testsPass` says; gh keeps its pull requests in `prs`. */
const run: Exec = (command, args, cwd) => {
	if (command === 'git') return exec(command, args, cwd);
	calls.push([command, ...args].join(' '));
	const ok = (stdout = '') => ({ status: 0, stdout, stderr: '' });

	if (command === 'bun') return args[0] === 'run' && !testsPass ? { ...ok(), status: 1 } : ok();
	if (args[1] === 'list') {
		const pr = prs.get(args[args.indexOf('--head') + 1] ?? '');
		return ok(JSON.stringify(pr === undefined ? [] : [pr]));
	}
	if (args[1] === 'create') {
		const branch = args[args.indexOf('--head') + 1] ?? '';
		const url = `https://github.com/acme/home/pull/${String(prs.size + 1)}`;
		prs.set(branch, { number: prs.size + 1, url, isDraft: args.includes('--draft') });
		return ok(`Creating pull request\n${url}`);
	}
	return ok();
};

/** The kit's home pushed to a bare origin, and an app whose patch was made against it. */
beforeEach(() => {
	vi.stubEnv('GIT_CONFIG_GLOBAL', '/dev/null');
	vi.stubEnv('GIT_CONFIG_NOSYSTEM', '1');
	vi.stubEnv('GIT_AUTHOR_NAME', 'Test');
	vi.stubEnv('GIT_AUTHOR_EMAIL', 'test@example.com');
	vi.stubEnv('GIT_COMMITTER_NAME', 'Test');
	vi.stubEnv('GIT_COMMITTER_EMAIL', 'test@example.com');

	root = mkdtempSync(path.join(tmpdir(), 'toolkit-upstream-test-'));
	home = path.join(root, 'home');
	app = path.join(root, 'app');
	calls = [];
	testsPass = true;
	prs = new Map();

	git(root, 'init', '--quiet', '--bare', '-b', 'master', 'origin.git');
	mkdirSync(home);
	git(home, 'init', '--quiet', '-b', 'master');
	put(home, 'package.json', json({ name: 'home', workspaces: ['packages/*'] }));
	put(
		home,
		'packages/kit/package.json',
		json({ name: '@acme/kit', version: '1.0.0', scripts: { test: 'vitest' } })
	);
	put(home, 'packages/kit/src/index.ts', KIT);
	git(home, 'add', '--all');
	git(home, 'commit', '--quiet', '-m', 'kit');
	git(home, 'remote', 'add', 'origin', path.join(root, 'origin.git'));
	git(home, 'push', '--quiet', 'origin', 'master');

	put(home, 'packages/kit/src/index.ts', FIXED);
	const patch = git(home, 'diff', '--relative=packages/kit');
	git(home, 'checkout', '--', '.');

	mkdirSync(app);
	git(app, 'init', '--quiet', '-b', 'main');
	put(
		app,
		'package.json',
		json({
			name: 'shop',
			patchedDependencies: {
				'@acme/kit@1.0.0': PATCH,
				'left-pad@1.0.0': 'patches/left-pad@1.0.0.patch',
			},
		})
	);
	put(app, PATCH, `${patch}\n`);
	git(app, 'add', '--all');
	git(app, 'commit', '--quiet', '-m', 'fix(kit): two is twenty-two');
	git(app, 'remote', 'add', 'origin', 'git@github.com:acme/shop.git');
});

afterEach(() => {
	vi.unstubAllEnvs();
	rmSync(root, { recursive: true, force: true });
});

const onBranch = (file: string) => git(home, 'show', `upstream/shop/kit:${file}`);

/** A change pushed to the home's master. */
function landOnMaster(text: string) {
	put(home, 'packages/kit/src/index.ts', text);
	git(home, 'commit', '--quiet', '--all', '-m', 'moved on');
	git(home, 'push', '--quiet', 'origin', 'master');
}

describe('names', () => {
	it('drop the scope and anything a branch cannot hold', () => {
		expect(slug('@acme/design-workbench')).toBe('design-workbench');
		expect(slug('my app')).toBe('my-app');
		expect(branchName('shop', '@acme/kit')).toBe('upstream/shop/kit');
	});

	it('link a GitHub commit, and leave any other remote’s bare', () => {
		expect(commitLink('git@github.com:acme/shop.git\n', 'abc')).toBe(
			'https://github.com/acme/shop/commit/abc'
		);
		expect(commitLink('https://github.com/acme/shop', 'abc')).toBe(
			'https://github.com/acme/shop/commit/abc'
		);
		expect(commitLink('https://example.com/shop.git', 'abc')).toBe('abc');
	});

	it('take the type off a commit subject', () => {
		expect(withoutType('fix(kit)!: two')).toBe('two');
		expect(withoutType('docs: two')).toBe('two');
		expect(withoutType('two: three')).toBe('three');
		expect(withoutType('Two, plainly')).toBe('Two, plainly');
	});

	it('bump as the commit says: breaking is major, or minor on 0.x; feat is minor from 1.0', () => {
		expect(bumpFor('fix(kit): two', '1.2.0')).toBe('patch');
		expect(bumpFor('feat(kit): two', '1.2.0')).toBe('minor');
		expect(bumpFor('feat(kit): two', '0.3.0')).toBe('patch');
		expect(bumpFor('fix(kit)!: two', '1.2.0')).toBe('major');
		expect(bumpFor('refactor!: two', '0.3.0')).toBe('minor');
		expect(bumpFor('fix: two\n\nBREAKING CHANGE: no more one', '1.2.0')).toBe('major');
		expect(bumpFor('Two, plainly', '1.2.0')).toBe('patch');
	});

	it("head the commit with the app's type, the package, and ! when the bump breaks", () => {
		expect(commitHeader('feat(shop): two', '@acme/kit', 'minor', '1.0.0')).toBe('feat(kit)');
		expect(commitHeader('fix: two', '@acme/kit', 'minor', '0.1.0')).toBe('fix(kit)!');
		expect(commitHeader('Two, plainly', '@acme/kit', 'major', '1.0.0')).toBe('fix(kit)!');
	});

	it('write a changeset for one package', () => {
		expect(changesetText('@acme/kit', 'minor', 'Breaks it.')).toBe(
			"---\n'@acme/kit': minor\n---\n\nBreaks it.\n"
		);
		expect(changesetText('@acme/kit', 'minor', 'Breaks it.', 'Call two().')).toBe(
			"---\n'@acme/kit': minor\n---\n\nBreaks it.\n\nBreaking: Call two().\n"
		);
	});

	it("take what an app must change from a breaking commit's footer, up to the next trailer", () => {
		expect(migrationNote('fix!: two\n\nWhy.\n\nBREAKING CHANGE: call two()\nnot one().')).toBe(
			'call two()\nnot one().'
		);
		expect(migrationNote('fix!: two\n\nBREAKING-CHANGE: call two()\nRefs: #4')).toBe('call two()');
		expect(migrationNote('fix!: two\n\nBREAKING CHANGE:')).toBeUndefined();
		expect(migrationNote('fix: two')).toBeUndefined();
	});
});

describe('upstream', () => {
	it('commits the patch on its branch with a changeset, and leaves the checkout as it was', () => {
		const [kit, pad] = upstream(app, { into: home }, run);

		expect(kit).toMatchObject({
			state: 'committed',
			branch: 'upstream/shop/kit',
			conflicts: [],
			tests: 'passed',
		});
		expect(pad).toEqual({
			name: 'left-pad',
			patch: 'patches/left-pad@1.0.0.patch',
			state: 'skipped',
		});
		expect(onBranch('packages/kit/src/index.ts')).toBe(FIXED.trimEnd());
		expect(onBranch('.changeset/upstream-shop-kit.md')).toBe(
			changesetText('@acme/kit', 'patch', 'two is twenty-two').trimEnd()
		);
		expect(git(home, 'log', '-1', '--format=%B', 'upstream/shop/kit')).toBe(
			`fix(kit): two is twenty-two\n\nFrom shop: https://github.com/acme/shop/commit/${git(app, 'rev-parse', 'HEAD')}\nPatch: ${PATCH}`
		);
		expect(calls).toEqual(['bun install --frozen-lockfile', 'bun run test --run']);
		expect(git(home, 'branch', '--show-current')).toBe('master');
		expect(git(home, 'status', '--porcelain')).toBe('');
		expect(git(home, 'worktree', 'list').split('\n')).toHaveLength(1);
	});

	it('merges onto a base that moved on elsewhere in the file', () => {
		landOnMaster(KIT.replace('three = 3', 'three = 33'));

		const [kit] = upstream(app, { into: home, only: ['@acme/kit'] }, run);

		expect(kit?.conflicts).toEqual([]);
		expect(onBranch('packages/kit/src/index.ts')).toBe(
			FIXED.replace('three = 3', 'three = 33').trimEnd()
		);
	});

	it('commits a conflict with its markers, and runs no tests on it', () => {
		landOnMaster(KIT.replace('two = 2', 'two = 2000'));

		const [kit] = upstream(app, { into: home, only: ['@acme/kit'] }, run);

		expect(kit).toMatchObject({
			state: 'committed',
			conflicts: ['packages/kit/src/index.ts'],
			tests: 'skipped',
		});
		expect(onBranch('packages/kit/src/index.ts')).toContain('<<<<<<<');
		expect(calls).toEqual([]);
	});

	it('says the base has the fix already, and commits nothing', () => {
		landOnMaster(FIXED);

		const [kit] = upstream(app, { into: home, only: ['@acme/kit'] }, run);

		expect(kit?.state).toBe('already');
		expect(git(home, 'rev-parse', 'upstream/shop/kit')).toBe(
			git(home, 'rev-parse', 'origin/master')
		);
	});

	it('pushes and opens the pull request; run again, it pushes nothing and updates the same one', () => {
		const [first] = upstream(app, { into: home, only: ['@acme/kit'], push: true }, run);
		const pushed = git(root, '--git-dir=origin.git', 'rev-parse', 'upstream/shop/kit');

		expect(first).toMatchObject({ state: 'pushed', pr: 'https://github.com/acme/home/pull/1' });
		expect(calls.filter(call => call.startsWith('gh pr create'))).toHaveLength(1);
		expect(calls.find(call => call.startsWith('gh pr create'))).not.toContain('--draft');

		calls = [];
		const [second] = upstream(app, { into: home, only: ['@acme/kit'], push: true }, run);

		expect(second).toMatchObject({ state: 'unchanged', pr: 'https://github.com/acme/home/pull/1' });
		expect(git(root, '--git-dir=origin.git', 'rev-parse', 'upstream/shop/kit')).toBe(pushed);
		const [list, edit, ...more] = calls.filter(call => call.startsWith('gh pr'));
		expect(list).toBe('gh pr list --head upstream/shop/kit --state open --json number,url,isDraft');
		expect(edit).toMatch(/^gh pr edit 1 --title fix\(kit\): /);
		expect(more).toEqual([]);
	});

	it("bumps as the app's commit says, and says it in the commit", () => {
		git(app, 'commit', '--quiet', '--amend', '-m', 'feat(kit)!: two is twenty-two');

		const [kit] = upstream(app, { into: home, only: ['@acme/kit'] }, run);

		expect(kit?.bump).toBe('major');
		expect(onBranch('.changeset/upstream-shop-kit.md')).toContain("'@acme/kit': major");
		expect(git(home, 'log', '-1', '--format=%s', 'upstream/shop/kit')).toBe(
			'feat(kit)!: two is twenty-two'
		);
	});

	it('carries what an app must change into the changeset and the pull request', () => {
		git(
			app,
			'commit',
			'--quiet',
			'--amend',
			'-m',
			'fix(kit)!: two is twenty-two',
			'-m',
			'BREAKING CHANGE: read two where you read one.'
		);

		upstream(app, { into: home, only: ['@acme/kit'], push: true }, run);

		expect(onBranch('.changeset/upstream-shop-kit.md')).toBe(
			changesetText(
				'@acme/kit',
				'major',
				'two is twenty-two',
				'read two where you read one.'
			).trimEnd()
		);
		expect(calls.find(call => call.startsWith('gh pr create'))).toContain(
			'**Breaking.** What an app must change:\n\nread two where you read one.'
		);
	});

	it('opens a draft when the tests fail, and says so', () => {
		testsPass = false;

		const [kit] = upstream(
			app,
			{ into: home, only: ['@acme/kit'], push: true, bump: 'minor' },
			run
		);

		expect(kit?.tests).toBe('failed');
		expect(prs.get('upstream/shop/kit')?.isDraft).toBe(true);
		expect(calls.find(call => call.startsWith('gh pr create'))).toContain(
			"The package's tests failed on this branch."
		);
		expect(onBranch('.changeset/upstream-shop-kit.md')).toContain("'@acme/kit': minor");
	});
});
