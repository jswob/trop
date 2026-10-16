import { spawnSync } from 'node:child_process';
import {
	cpSync,
	existsSync,
	mkdirSync,
	mkdtempSync,
	readFileSync,
	realpathSync,
	rmSync,
	writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

import { afterAll, beforeAll, describe, expect, it } from 'vitest';

import {
	defaultDbPort,
	defaultPort,
	defaultTitle,
	makeApp,
	parse,
	stripTemplate,
	templateCommit,
	type Made,
} from './setup';

const template = path.resolve(import.meta.dirname, '..');

// No maintenance in the background: a commit may start one that writes into .git while the
// folder is being removed
const GIT_CONFIG = ['user.name=t', 'user.email=t@t', 'gc.auto=0', 'maintenance.auto=false'];

function git(cwd: string, args: string[]): string {
	const run = spawnSync('git', [...GIT_CONFIG.flatMap(entry => ['-c', entry]), ...args], {
		cwd,
		encoding: 'utf8',
	});
	if (run.status !== 0) throw new Error(`git ${args.join(' ')}: ${run.stderr}`);
	return run.stdout.trim();
}

const scratch = () => realpathSync(mkdtempSync(path.join(tmpdir(), 'template-setup-')));
const remove = (dir: string) =>
	rmSync(dir, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });

/** A clone of the template as it stands, uncommitted work included, in a temporary folder. */
function cloneTemplate(): string {
	const root = scratch();
	const files = git(template, ['ls-files', '-z', '--cached', '--others', '--exclude-standard'])
		.split('\0')
		.filter(file => file !== '' && existsSync(path.join(template, file)));
	for (const file of files) {
		mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
		cpSync(path.join(template, file), path.join(root, file));
	}
	git(root, ['init', '-q']);
	git(root, ['add', '--all']);
	git(root, ['commit', '-q', '-m', 'template']);
	return root;
}

describe('parse', () => {
	it('takes a name and fills the rest from it', () => {
		expect(parse(['lab-app'])).toEqual({
			name: 'lab-app',
			title: 'Lab App',
			dbPort: defaultDbPort('lab-app'),
			port: defaultPort('lab-app'),
			renovate: 'auto',
			home: '../bun-monorepo',
			install: true,
		});
		expect(
			parse([
				'x1',
				'--title',
				'X One',
				'--db-port',
				'6000',
				'--port',
				'4000',
				'--renovate',
				'off',
				'--no-install',
			])
		).toMatchObject({ title: 'X One', dbPort: 6000, port: 4000, renovate: 'off', install: false });
	});

	it('refuses what cannot name an app', () => {
		for (const argv of [
			[],
			['a', 'b'],
			['My-App'],
			['1app'],
			['app'],
			['x', '--renovate', 'some'],
		]) {
			expect(parse(argv)).toHaveProperty('error');
		}
		expect(parse(['x', '--db-port', '80'])).toHaveProperty('error');
		expect(parse(['x', '--db-port', 'five'])).toHaveProperty('error');
		expect(parse(['x', '--port', '65533'])).toHaveProperty('error');
		expect(parse(['x', '--colour'])).toHaveProperty('error');
	});

	it('words a name, and gives it ports of its own', () => {
		expect(defaultTitle('lab-app')).toBe('Lab App');
		const dbPort = defaultDbPort('lab-app');
		expect(dbPort).toBe(defaultDbPort('lab-app'));
		expect((dbPort - 5500) % 3).toBe(0);
		expect(dbPort).toBeGreaterThanOrEqual(5500);
		expect(dbPort).toBeLessThanOrEqual(5797);

		// The same slot: an app's two blocks move together
		const port = defaultPort('lab-app');
		expect((port - 3100) / 4).toBe((dbPort - 5500) / 3);
		expect(port).toBeGreaterThanOrEqual(3100);
		expect(port + 3).toBeLessThanOrEqual(3499);
	});
});

describe('templateCommit', () => {
	it('finds the commit with the tree in the checkout, and none without one', () => {
		const dir = scratch();
		try {
			git(dir, ['init', '-q']);
			writeFileSync(path.join(dir, 'a.txt'), 'a\n');
			git(dir, ['add', '--all']);
			git(dir, ['commit', '-q', '-m', 'a']);
			const tree = git(dir, ['rev-parse', 'HEAD^{tree}']);
			expect(templateCommit(dir, tree)).toBe(git(dir, ['rev-parse', 'HEAD']));
			expect(templateCommit(dir, '0'.repeat(40))).toBeUndefined();
			expect(templateCommit(path.join(dir, 'nowhere'), tree)).toBeUndefined();
		} finally {
			remove(dir);
		}
	});
});

describe('stripTemplate', () => {
	it('drops each block with its markers, and one of the blank lines around it', () => {
		const markdown = 'a\n\n<!-- template:start -->\n\nonly here\n\n<!-- template:end -->\n\nb\n';
		expect(stripTemplate(markdown, 'x.md')).toBe('a\n\nb\n');

		const yaml =
			'      - run: a\n      # template:start\n      - run: b\n      # template:end\n      - run: c\n';
		expect(stripTemplate(yaml, 'x.yml')).toBe('      - run: a\n      - run: c\n');

		const ts = "[\n\t'a',\n\t// template:start\n\t'b',\n\t// template:end\n]\n";
		expect(stripTemplate(ts, 'x.ts')).toBe("[\n\t'a',\n]\n");
	});

	it('leaves a line that only names the markers', () => {
		const prose = 'Blocks between `template:start` and `template:end` lines go.\n';
		expect(stripTemplate(prose, 'x.md')).toBe(prose);
	});

	it('refuses a marker without its pair', () => {
		expect(() => stripTemplate('# template:start\na\n', 'x.yml')).toThrow('x.yml:1');
		expect(() => stripTemplate('a\n# template:end\n', 'x.yml')).toThrow('x.yml:2');
		expect(() =>
			stripTemplate('# template:start\n# template:start\n# template:end\n', 'x')
		).toThrow();
	});
});

describe('makeApp on a clone of this template', () => {
	let root: string;
	let made: Made;
	let commit: string;
	const read = (file: string) => readFileSync(path.join(root, file), 'utf8');
	const json = (file: string) =>
		JSON.parse(read(file)) as {
			name?: string;
			scripts: Record<string, string>;
			devDependencies: Record<string, string>;
			toolkit?: unknown;
			template?: { repo: string; commit?: string; tree?: string };
			extends?: string[];
		};

	let home: string;

	beforeAll(() => {
		root = cloneTemplate();
		commit = git(root, ['rev-parse', 'HEAD']);
		// The toolkit's checkout has the template's commit; the app's repo, made with "Use this
		// template", has the same tree in a commit of its own
		home = path.join(scratch(), 'home');
		git(path.dirname(home), ['clone', '-q', root, home]);
		git(root, ['checkout', '-q', '--orphan', 'generated']);
		git(root, ['commit', '-q', '-m', 'Initial commit']);
		made = makeApp(root, {
			name: 'lab-app',
			title: 'Lab & App',
			dbPort: 5897,
			port: 3200,
			renovate: 'manual',
			home: path.relative(root, home),
		});
	}, 60_000);

	afterAll(() => {
		remove(root);
		remove(path.dirname(home));
	});

	it('deletes the packages and what only the template has', () => {
		expect(made.packages.map(entry => entry.name).sort()).toEqual([
			'@jswob/design-workbench',
			'@jswob/drizzle-fixtures',
			'@jswob/drizzle-snapshots',
			'@jswob/drizzle-utils',
			'@jswob/llm-evals',
			'@jswob/style-guide',
			'@jswob/toolkit',
		]);
		for (const gone of [
			...made.packages.map(entry => entry.dir),
			'template',
			'TEMPLATE.md',
			'.changeset',
			'renovate',
			'scripts/release.ts',
			'.github/workflows/release.yml',
		]) {
			expect(existsSync(path.join(root, gone)), gone).toBe(false);
		}
		expect(existsSync(path.join(root, 'packages/ui/package.json'))).toBe(true);
		expect(existsSync(path.join(root, 'scripts/setup.ts'))).toBe(true);
	});

	it('takes the packages by version, and keeps only the scripts that still run', () => {
		const manifest = json('package.json');
		expect(manifest.name).toBe('lab-app');
		expect(manifest.devDependencies['@jswob/toolkit']).toMatch(/^\^\d+\.\d+\.\d+$/);
		expect(manifest.devDependencies).not.toHaveProperty('@changesets/cli');
		expect(manifest.devDependencies).not.toHaveProperty('semver');
		expect(manifest.toolkit).toEqual({
			agnostic: { paths: ['patches'], words: [] },
			home: path.relative(root, home),
		});
		expect(manifest.template?.repo).toBe('jswob/bun-monorepo');
		expect(manifest.template?.commit).toBe(commit);
		expect(manifest.template?.tree).toBe(git(home, ['rev-parse', 'HEAD^{tree}']));

		const { scripts } = manifest;
		expect(Object.keys(scripts)).not.toEqual(expect.arrayContaining(['template:test']));
		for (const gone of [
			'changeset',
			'release:publish',
			'scripts:test',
			'template:typecheck',
			'toolkit:test',
			'workbench:test',
			'llm-evals:typecheck',
			'drizzle-fixtures:test:integration',
		]) {
			expect(scripts, gone).not.toHaveProperty(gone);
		}
		for (const kept of [
			'toolkit:check',
			'workbench',
			'designs:test',
			'evals:test',
			'ui:test',
			'scripts:typecheck',
			'setup',
		]) {
			expect(scripts, kept).toHaveProperty(kept);
		}
		for (const [name, command] of Object.entries(scripts)) {
			for (const [, dir] of command.matchAll(/--cwd (\S+)/g)) {
				expect(existsSync(path.join(root, dir!)), `${name}: ${dir!}`).toBe(true);
			}
		}

		for (const file of made.changed.filter(file => file.endsWith('package.json'))) {
			expect(read(file), file).not.toMatch(/"@jswob\/[^"]+": "workspace:/);
		}
	});

	it('names the app everywhere the template said app', () => {
		expect(json('apps/api/package.json').name).toBe('@lab-app/api');
		expect(read('apps/web/src/lib/api.ts')).toContain("from '@lab-app/api'");
		expect(read('apps/workbench/kit/kit.test.ts')).toContain('@lab-app\\/api');
		expect(read('apps/api/src/db/target.ts')).toContain("'lab_app_test_db'");
		expect(read('deploy/fly-api.toml')).toContain('app = "lab-app-api"');
		expect(read('apps/web/index.html')).toContain('<title>Lab &#38; App</title>');
		expect(read('README.md').split('\n')[0]).toBe('# Lab & App');

		const env = read('.env.example');
		for (const line of [
			'DB_PORT=5897',
			'TEST_DB_PORT=5898',
			'E2E_DB_PORT=5899',
			'DB_NAME=lab_app_db',
			'E2E_DB_NAME=lab_app_e2e_db',
			'COMPOSE_PROJECT_NAME=lab-app',
			'API_PORT=3200',
			'WEB_PORT=3201',
		]) {
			expect(env).toMatch(new RegExp(`^${line}$`, 'm'));
		}
		expect(read('apps/web/playwright.config.ts')).toContain('const E2E_WEB_PORT = 3202;');
		expect(read('apps/workbench/workbench.config.ts')).toContain('port: 3203,');
		expect(read('README.md')).toContain('http://localhost:3201');
	});

	it('writes renovate.json on the preset asked for', () => {
		expect(json('renovate.json').extends).toEqual([
			'config:best-practices',
			'github>jswob/bun-monorepo//renovate/third-party',
			'github>jswob/bun-monorepo//renovate/toolkit-manual',
		]);
	});

	it('leaves no file pointing at what it deleted', () => {
		// Globs and ignore lists that match nothing in an app; bun install rewrites the lock
		const harmless = new Set(['.dockerignore', '.prettierignore', 'eslint.config.js', 'bun.lock']);
		const removedScripts = Object.keys(
			(
				JSON.parse(readFileSync(path.join(template, 'package.json'), 'utf8')) as {
					scripts: Record<string, string>;
				}
			).scripts
		).filter(script => !(script in json('package.json').scripts));
		const leftovers = [
			/^\s*(?:<!--|#|\/\/)\s*template:(?:start|end)/m,
			/@app[\\/]/,
			/packages\/(?:config|drizzle-fixtures|drizzle-snapshots|drizzle-utils|llm-evals|toolkit|workbench)(?![\w-])/,
			/TEMPLATE\.md|\.changeset|bun changeset|release\.yml|scripts\/release|(?<![\w-])template\//,
			// The template's dev ports, which the app's share without an error
			/\b(?:3001|5173|5174|5175)\b/,
			new RegExp(
				`bun (?:run )?(?:${removedScripts.map(script => script.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')).join('|')})(?![\\w:-])`
			),
		];

		// The upstream skill runs bun changeset in the toolkit's checkout, not in the app
		const allowed: Record<string, string[]> = {
			'.claude/skills/upstream/SKILL.md': ['bun changeset'],
		};

		const files = git(root, ['ls-files', '-z', '--cached', '--others', '--exclude-standard'])
			.split('\0')
			.filter(file => file !== '' && !harmless.has(file) && existsSync(path.join(root, file)));
		const hits = files.flatMap(file => {
			const text = read(file);
			if (text.includes('\0')) return [];
			return leftovers.flatMap(pattern => {
				const global = new RegExp(pattern.source, `${pattern.flags}g`);
				return [...text.matchAll(global)]
					.map(match => match[0].trim())
					.filter(found => !(allowed[file] ?? []).includes(found))
					.map(found => `${file}: ${found}`);
			});
		});
		expect(hits).toEqual([]);
	});

	it('keeps what an app needs of the docs and the workflows', () => {
		const readme = read('README.md');
		expect(readme).not.toContain('## Starting an app from it');
		expect(readme).toContain("## Making it the app's");
		expect(read('CLAUDE.md')).not.toContain('Publish to npm');
		expect(read('docs/removing-notes.md')).toContain('# Removing notes');
		expect(read('vitest.config.ts')).toMatch(/projects: \[\s*'apps\/\*',\s*'packages\/\*',\s*\]/);

		const ci = read('.github/workflows/ci.yml');
		expect(ci).toContain('bun evals:test');
		expect(ci).toContain('bun designs:test');
		expect(ci).not.toContain('toolkit:test');
	});

	it('refuses a repo that is an app already, one with changes, and a folder outside git', () => {
		const options = {
			name: 'x',
			title: 'X',
			dbPort: 6000,
			port: 4000,
			renovate: 'auto',
			home: '..',
		} as const;
		const changed = scratch();
		const loose = scratch();
		try {
			for (const dir of [changed, loose]) {
				mkdirSync(path.join(dir, 'template'));
				writeFileSync(path.join(dir, 'template', 'setup.ts'), '\n');
			}
			git(changed, ['init', '-q']);
			git(changed, ['add', '--all']);
			git(changed, ['commit', '-q', '-m', 'template']);
			writeFileSync(path.join(changed, 'stray.txt'), 'x\n');

			expect(() => makeApp(root, options)).toThrow('it has no template/setup.ts');
			expect(() => makeApp(changed, options)).toThrow('commit or stash the changes first');
			expect(() => makeApp(loose, options)).toThrow('is not a git checkout');
		} finally {
			remove(changed);
			remove(loose);
		}
	});
});
