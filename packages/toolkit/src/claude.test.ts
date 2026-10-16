import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import { LOCK_FILE } from './assets';
import { check, installedPackages, readLock, sync } from './claude';

let repo: string;

function put(file: string, text: string) {
	const full = path.join(repo, file);
	mkdirSync(path.dirname(full), { recursive: true });
	writeFileSync(full, text, 'utf8');
}

const json = (value: unknown) => `${JSON.stringify(value, null, '\t')}\n`;
const copy = (name: string) => readFileSync(path.join(repo, '.claude', name), 'utf8');
const copied = (name: string) => existsSync(path.join(repo, '.claude', name));

/**
 * A repo whose root depends on `@acme/kit` (a skill and a command) and whose `apps/web` depends on
 * `@acme/tools` (an agent), installed in the workspace's own `node_modules`. `@acme/unused` is
 * installed and declares files, but nothing depends on it.
 */
beforeEach(() => {
	repo = mkdtempSync(path.join(tmpdir(), 'toolkit-claude-'));

	put('package.json', json({ workspaces: ['apps/*'], devDependencies: { '@acme/kit': '*' } }));
	put('apps/web/package.json', json({ name: 'web', dependencies: { '@acme/tools': '*' } }));

	put(
		'node_modules/@acme/kit/package.json',
		json({
			name: '@acme/kit',
			version: '1.0.0',
			claude: { skills: { kit: 'skill' }, commands: 'commands' },
		})
	);
	put('node_modules/@acme/kit/skill/SKILL.md', '# Kit\n');
	put('node_modules/@acme/kit/skill/ref/LAYOUT.md', '# Layout\n');
	put('node_modules/@acme/kit/skill/.notes', 'not shipped');
	put('node_modules/@acme/kit/commands/go.md', '# Go\n');
	put('node_modules/@acme/kit/commands/script.sh', 'not a command');

	put(
		'apps/web/node_modules/@acme/tools/package.json',
		json({ name: '@acme/tools', version: '2.0.0', claude: { agents: 'agents' } })
	);
	put('apps/web/node_modules/@acme/tools/agents/helper.md', '# Helper\n');

	put(
		'node_modules/@acme/unused/package.json',
		json({ name: '@acme/unused', version: '1.0.0', claude: { agents: 'agents' } })
	);
	put('node_modules/@acme/unused/agents/stray.md', '# Stray\n');
});

afterEach(() => {
	rmSync(repo, { recursive: true, force: true });
});

describe('installedPackages', () => {
	it('reads the packages the root and the workspaces depend on, and no other', () => {
		expect(installedPackages(repo).map(pkg => `${pkg.name}@${pkg.version}`)).toEqual([
			'@acme/kit@1.0.0',
			'@acme/tools@2.0.0',
		]);
	});
});

describe('sync', () => {
	it('copies a package’s rules into .claude/rules/', () => {
		put(
			'apps/web/node_modules/@acme/tools/package.json',
			json({ name: '@acme/tools', version: '2.0.0', claude: { rules: 'rules' } })
		);
		put('apps/web/node_modules/@acme/tools/rules/installed.md', '---\npaths: ["x/**"]\n---\n');

		expect(sync(repo).synced.find(s => s.name === '@acme/tools')?.write).toEqual([
			'rules/installed.md',
		]);
		expect(copy('rules/installed.md')).toBe('---\npaths: ["x/**"]\n---\n');
	});

	it('copies each package’s files into .claude/ and records them in the lock', () => {
		const { synced } = sync(repo);

		expect(synced.map(s => [s.name, s.write])).toEqual([
			['@acme/kit', ['commands/go.md', 'skills/kit/SKILL.md', 'skills/kit/ref/LAYOUT.md']],
			['@acme/tools', ['agents/helper.md']],
		]);
		expect(copy('skills/kit/ref/LAYOUT.md')).toBe('# Layout\n');
		expect(copied('skills/kit/.notes')).toBe(false);
		expect(copied('commands/script.sh')).toBe(false);
		expect(copied('agents/stray.md')).toBe(false);
		expect(readLock(repo).packages['@acme/tools']).toEqual({
			version: '2.0.0',
			files: ['agents/helper.md'],
		});
		expect(check(repo)).toEqual([]);
	});

	it('writes nothing the second time', () => {
		sync(repo);

		expect(sync(repo).synced.flatMap(s => s.write)).toEqual([]);
	});

	it('puts back a copy edited in the repo, which check names first', () => {
		sync(repo);
		put('.claude/skills/kit/SKILL.md', '# Edited here\n');

		expect(check(repo)).toEqual(['skills/kit/SKILL.md differs from @acme/kit’s']);
		sync(repo);
		expect(copy('skills/kit/SKILL.md')).toBe('# Kit\n');
	});

	it('removes a file the package dropped, and leaves the skill’s PROJECT.md', () => {
		sync(repo);
		put('.claude/skills/kit/PROJECT.md', '# This repo\n');
		rmSync(path.join(repo, 'node_modules/@acme/kit/skill/ref/LAYOUT.md'));

		expect(check(repo)).toEqual(['skills/kit/ref/LAYOUT.md is no longer part of @acme/kit']);
		expect(sync(repo).synced[0]?.remove).toEqual(['skills/kit/ref/LAYOUT.md']);
		expect(copied('skills/kit/ref/LAYOUT.md')).toBe(false);
		expect(copy('skills/kit/PROJECT.md')).toBe('# This repo\n');
		expect(check(repo)).toEqual([]);
	});

	it('names a new version and the files it changed', () => {
		sync(repo);
		put(
			'node_modules/@acme/kit/package.json',
			json({
				name: '@acme/kit',
				version: '1.1.0',
				claude: { skills: { kit: 'skill' }, commands: 'commands' },
			})
		);
		put('node_modules/@acme/kit/commands/stop.md', '# Stop\n');

		expect(check(repo)).toEqual([
			'@acme/kit was synced at 1.0.0, 1.1.0 is installed',
			'commands/stop.md is missing',
		]);
	});

	it('removes the files of a package no longer depended on', () => {
		sync(repo);
		put('apps/web/package.json', json({ name: 'web' }));

		expect(check(repo)).toEqual(['@acme/tools is no longer installed, but its files are']);
		expect(sync(repo).uninstalled).toEqual([
			{ name: '@acme/tools', removed: ['agents/helper.md'] },
		]);
		expect(copied('agents/helper.md')).toBe(false);
		expect(readLock(repo).packages['@acme/tools']).toBeUndefined();
	});

	it('with only, syncs and checks those packages and leaves the rest of the lock as it is', () => {
		sync(repo, ['@acme/kit']);

		expect(Object.keys(readLock(repo).packages)).toEqual(['@acme/kit']);
		expect(copied('agents/helper.md')).toBe(false);
		expect(check(repo, ['@acme/kit'])).toEqual([]);
		expect(check(repo)).toEqual(['@acme/tools was never synced']);

		sync(repo);
		put('apps/web/package.json', json({ name: 'web' }));
		sync(repo, ['@acme/kit']);
		expect(Object.keys(readLock(repo).packages)).toEqual(['@acme/kit', '@acme/tools']);
	});

	it('keeps a file that moved to another package', () => {
		sync(repo);
		rmSync(path.join(repo, 'node_modules/@acme/kit/commands/go.md'));
		put(
			'apps/web/node_modules/@acme/tools/package.json',
			json({
				name: '@acme/tools',
				version: '2.1.0',
				claude: { agents: 'agents', commands: 'commands' },
			})
		);
		put('apps/web/node_modules/@acme/tools/commands/go.md', '# Go, moved\n');

		sync(repo);

		expect(copy('commands/go.md')).toBe('# Go, moved\n');
		expect(readLock(repo).packages['@acme/kit']?.files).not.toContain('commands/go.md');
		expect(check(repo)).toEqual([]);
	});

	it('refuses two packages that ship the same file', () => {
		put('apps/web/node_modules/@acme/tools/commands/go.md', '# Go\n');
		put(
			'apps/web/node_modules/@acme/tools/package.json',
			json({
				name: '@acme/tools',
				version: '2.0.0',
				claude: { agents: 'agents', commands: 'commands' },
			})
		);

		expect(() => sync(repo)).toThrow('@acme/kit and @acme/tools both ship commands/go.md');
		expect(existsSync(path.join(repo, LOCK_FILE))).toBe(false);
	});
});
