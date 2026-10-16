import {
	existsSync,
	mkdirSync,
	mkdtempSync,
	readlinkSync,
	rmSync,
	symlinkSync,
	writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import { link, readLinks } from './link';
import type { Run } from './patches';
import { dependents, failures, tryPackages } from './try';
import type { Exec } from './upstream';

let root: string;
let repo: string;
let home: string;
let installs: number;
let runs: string[];
/** What each test does on what the app installs, and on the checkout; absent means it passes. */
let outcomes: Record<string, { before?: string; after?: string }>;
/** The tests give no report: before linking, or on the checkout. */
let silent: 'before' | 'after' | null;

const json = (value: unknown) => `${JSON.stringify(value, null, '\t')}\n`;

function put(dir: string, file: string, text: string) {
	const full = path.join(dir, file);
	mkdirSync(path.dirname(full), { recursive: true });
	writeFileSync(full, text, 'utf8');
}

const STORE = 'node_modules/.bun/@acme+kit@1.0.0/node_modules/@acme/kit';
const KIT = path.join('node_modules', '@acme', 'kit');

function installKit() {
	const full = path.join(repo, KIT);
	if (existsSync(full)) return;
	mkdirSync(path.dirname(full), { recursive: true });
	symlinkSync(path.join(repo, STORE), full, 'dir');
}

const install: Run = () => {
	installs += 1;
	installKit();
};

/** Vitest with the JSON reporter, on whichever kit the root's link reaches. */
const run: Exec = (command, args, cwd) => {
	runs.push(`${path.relative(repo, cwd) || '.'}: ${[command, ...args].join(' ')}`);
	const linked = readlinkSync(path.join(repo, KIT)) === path.join(home, 'packages/kit');
	const when = linked ? 'after' : 'before';
	if (silent === when) return { status: 1, stdout: '', stderr: 'boom' };

	const out = (args.find(arg => arg.startsWith('--outputFile=')) ?? '').split('=')[1] ?? '';
	const file = path.join(cwd, 'src/app.test.ts');
	const statuses = Object.entries(outcomes).map(([fullName, o]) => ({
		fullName,
		status: o[when] ?? 'passed',
	}));
	writeFileSync(
		out,
		JSON.stringify({
			testResults: [
				{
					name: file,
					status: statuses.some(s => s.status === 'failed') ? 'failed' : 'passed',
					assertionResults: statuses,
				},
			],
		})
	);
	return { status: 0, stdout: '', stderr: '' };
};

beforeEach(() => {
	root = mkdtempSync(path.join(tmpdir(), 'toolkit-try-test-'));
	repo = path.join(root, 'app');
	home = path.join(root, 'home');
	installs = 0;
	runs = [];
	outcomes = {};
	silent = null;

	put(
		repo,
		'package.json',
		json({ name: 'shop', workspaces: ['apps/*'], scripts: { test: 'vitest --run' } })
	);
	put(
		repo,
		'apps/web/package.json',
		json({ name: 'web', dependencies: { '@acme/kit': '^1.0.0' }, scripts: { test: 'vitest' } })
	);
	put(repo, 'apps/api/package.json', json({ name: 'api', scripts: { test: 'vitest' } }));
	put(repo, `${STORE}/package.json`, json({ name: '@acme/kit', version: '1.0.0' }));
	installKit();

	put(home, 'package.json', json({ name: 'home', workspaces: ['packages/*'] }));
	put(home, 'packages/kit/package.json', json({ name: '@acme/kit', version: '1.1.0' }));
	mkdirSync(path.join(home, 'node_modules'));
});

afterEach(() => {
	rmSync(root, { recursive: true, force: true });
});

describe('dependents', () => {
	it('are the workspaces with tests that depend on a package', () => {
		expect(dependents(repo, ['@acme/kit'])).toEqual([path.join(repo, 'apps/web')]);
	});

	it('are the root alone when no workspace depends on it', () => {
		put(
			repo,
			'package.json',
			json({
				name: 'shop',
				workspaces: ['apps/*'],
				devDependencies: { '@acme/tool': '^1.0.0' },
				scripts: { test: 'vitest --run' },
			})
		);

		expect(dependents(repo, ['@acme/tool'])).toEqual([repo]);
	});
});

describe('failures', () => {
	it('names each failing test by file, and a file that failed with none by itself', () => {
		expect(
			failures(
				{
					testResults: [
						{
							name: '/app/a.test.ts',
							status: 'failed',
							assertionResults: [
								{ fullName: 'a works', status: 'passed' },
								{ fullName: 'a breaks', status: 'failed' },
							],
						},
						{ name: '/app/b.test.ts', status: 'failed', assertionResults: [] },
						{ name: '/app/c.test.ts', status: 'passed', assertionResults: [] },
					],
				},
				'/app'
			)
		).toEqual(['a.test.ts > a breaks', 'b.test.ts']);
	});
});

describe('tryPackages', () => {
	it('fails only what passed before, and unlinks', () => {
		outcomes = {
			'was broken': { before: 'failed', after: 'failed' },
			'breaks now': { after: 'failed' },
			'mended now': { before: 'failed' },
			fine: {},
		};

		const [web] = tryPackages(repo, home, ['@acme/kit'], run, install);

		const name = (test: string) => `apps/web/src/app.test.ts > ${test}`;
		expect(web).toEqual({
			workspace: 'apps/web',
			broken: [name('breaks now')],
			fixed: [name('mended now')],
			failing: [name('was broken')],
		});
		expect(runs).toHaveLength(2);
		expect(runs[0]).toMatch(/^apps\/web: bun run test --run --reporter=json --outputFile=\S+$/);
		expect(installs).toBe(1);
		expect(readLinks(repo)).toEqual({});
		expect(readlinkSync(path.join(repo, KIT))).toBe(path.join(repo, STORE));
	});

	it('gives the root no second --run', () => {
		put(
			repo,
			'package.json',
			json({
				name: 'shop',
				workspaces: ['apps/*'],
				devDependencies: { '@acme/kit': '^1.0.0' },
				scripts: { test: 'vitest --run' },
			})
		);
		rmSync(path.join(repo, 'apps/web'), { recursive: true });

		tryPackages(repo, home, ['@acme/kit'], run, install);

		expect(runs[0]).toMatch(/^\.: bun run test --reporter=json/);
	});

	it('counts a run on the checkout that gives no report as broken, and still unlinks', () => {
		silent = 'after';

		const [web] = tryPackages(repo, home, ['@acme/kit'], run, install);

		expect(web?.broken).toEqual(['apps/web: the tests gave no report on the checkout']);
		expect(readLinks(repo)).toEqual({});
	});

	it('stops before linking when the tests give no report on what the app installs', () => {
		silent = 'before';

		expect(() => tryPackages(repo, home, ['@acme/kit'], run, install)).toThrow(
			'the tests in apps/web gave no report before linking'
		);
		expect(installs).toBe(0);
		expect(readlinkSync(path.join(repo, KIT))).toBe(path.join(repo, STORE));
	});

	it('refuses a package linked already', () => {
		link(repo, home, ['@acme/kit']);

		expect(() => tryPackages(repo, home, ['@acme/kit'], run, install)).toThrow(
			'@acme/kit is linked already'
		);
		expect(runs).toEqual([]);
	});

	it('runs nothing when no workspace with tests depends on the package', () => {
		expect(tryPackages(repo, home, ['@acme/none'], run, install)).toEqual([]);
		expect(runs).toEqual([]);
	});
});
