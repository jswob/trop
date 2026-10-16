import { mkdirSync, mkdtempSync, realpathSync, rmSync, symlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import { sync } from './claude';
import { guard, notice, waitingAmong } from './hooks';
import { LINK_FILE } from './link';
import { MARKER_FILE } from './patches';

let repo: string;

function put(file: string, text: string) {
	const full = path.join(repo, file);
	mkdirSync(path.dirname(full), { recursive: true });
	writeFileSync(full, text, 'utf8');
}

const json = (value: unknown) => `${JSON.stringify(value, null, '\t')}\n`;
const edit = (file: string, tool = 'Edit') =>
	guard({ tool_name: tool, tool_input: { file_path: path.join(repo, file) } });
const patching = () =>
	put(MARKER_FILE, json({ '@acme/kit': { version: '1.0.0', dir: 'node_modules/@acme/kit' } }));

beforeEach(() => {
	repo = realpathSync(mkdtempSync(path.join(tmpdir(), 'toolkit-hooks-')));
	mkdirSync(path.join(repo, '.git'));
	put(
		'package.json',
		json({ workspaces: ['apps/*', 'packages/*'], devDependencies: { '@acme/kit': '^1.0.0' } })
	);
	put(
		'node_modules/@acme/kit/package.json',
		json({ name: '@acme/kit', version: '1.0.0', claude: { skills: { kit: 'skill' } } })
	);
	put('node_modules/@acme/kit/skill/SKILL.md', '# Kit\n');
	put('node_modules/@acme/kit/src/index.ts', 'export const x = 1;\n');
	sync(repo);
});

afterEach(() => {
	rmSync(repo, { recursive: true, force: true });
});

describe('guard', () => {
	it('refuses an edit to an installed copy, and says how to patch it', () => {
		expect(edit('node_modules/@acme/kit/src/index.ts')).toBe(
			'node_modules/@acme/kit/src/index.ts is in the installed copy of @acme/kit: the next install ' +
				'puts it back, and nothing ships the edit. To fix @acme/kit in this repo, run bun toolkit ' +
				'patch @acme/kit, then edit it at node_modules/@acme/kit (the upstream skill has the rest).'
		);
		expect(edit('node_modules/@acme/kit/src/new.ts', 'Write')).toContain(
			'is in the installed copy of @acme/kit'
		);
	});

	it('lets the copy being patched through, and no other copy of it', () => {
		patching();
		const store = 'node_modules/.bun/@acme+kit@1.0.0/node_modules/@acme/kit';
		put(`${store}/src/index.ts`, 'export const x = 1;\n');

		expect(edit('node_modules/@acme/kit/src/index.ts')).toBeNull();
		expect(edit(`${store}/src/index.ts`)).toBe(
			`@acme/kit is being patched at node_modules/@acme/kit, not at ${store}/src/index.ts: edit it there.`
		);
	});

	it("refuses an edit to a package's copy in .claude/ until the package is being patched", () => {
		expect(edit('.claude/skills/kit/SKILL.md')).toBe(
			".claude/skills/kit/SKILL.md is @acme/kit's copy: bun toolkit sync writes it again, and bun " +
				'toolkit check fails on an edit. To change it here, run bun toolkit patch @acme/kit, edit ' +
				'this copy, then bun toolkit patch @acme/kit --commit (the upstream skill has the rest).'
		);

		patching();

		expect(edit('.claude/skills/kit/SKILL.md')).toBeNull();
	});

	it("points a copy of the repo's own package at its source", () => {
		put(
			'packages/own/package.json',
			json({ name: '@acme/own', version: '0.1.0', claude: { skills: { own: 'skill' } } })
		);
		put('packages/own/skill/SKILL.md', '# Own\n');
		put('apps/web/package.json', json({ name: 'web', dependencies: { '@acme/own': '*' } }));
		mkdirSync(path.join(repo, 'node_modules/@acme'), { recursive: true });
		symlinkSync(path.join(repo, 'packages/own'), path.join(repo, 'node_modules/@acme/own'));
		sync(repo);

		expect(edit('.claude/skills/own/SKILL.md')).toBe(
			".claude/skills/own/SKILL.md is a copy of packages/own/skill/SKILL.md, which @acme/own's " +
				'source holds: edit that, then bun toolkit sync.'
		);
		expect(edit('node_modules/@acme/own/skill/SKILL.md')).toBeNull();
	});

	it("lets the repo's own files through, and every tool that does not edit", () => {
		expect(edit('.claude/skills/kit/PROJECT.md', 'Write')).toBeNull();
		expect(edit('.claude/settings.json')).toBeNull();
		expect(edit('apps/web/src/main.ts', 'Write')).toBeNull();
		expect(edit('node_modules/@acme/kit/src/index.ts', 'Read')).toBeNull();
		expect(guard({ tool_name: 'Edit', tool_input: {} })).toBeNull();
	});

	it('reads a relative path from the working directory', () => {
		expect(
			guard({
				tool_name: 'Edit',
				tool_input: { file_path: 'node_modules/@acme/kit/src/index.ts' },
				cwd: repo,
			})
		).toContain('is in the installed copy of @acme/kit');
	});

	it('lets a linked package through: its files are the checkout’s source', () => {
		const home = realpathSync(mkdtempSync(path.join(tmpdir(), 'toolkit-hooks-home-')));
		try {
			mkdirSync(path.join(home, '.git'));
			mkdirSync(path.join(home, 'packages/kit/src'), { recursive: true });
			writeFileSync(path.join(home, 'packages/kit/src/index.ts'), 'export const x = 1;\n');
			rmSync(path.join(repo, 'node_modules/@acme/kit'), { recursive: true });
			symlinkSync(path.join(home, 'packages/kit'), path.join(repo, 'node_modules/@acme/kit'));

			expect(edit('node_modules/@acme/kit/src/index.ts')).toBeNull();
		} finally {
			rmSync(home, { recursive: true, force: true });
		}
	});
});

describe('notice', () => {
	it('says nothing when all is in order', () => {
		expect(notice(repo)).toEqual([]);
	});

	it('names a patch being made, a link, a stale copy and a waiting update', () => {
		patching();
		put(LINK_FILE, json({ '@acme/tools': { dir: '../home/packages/tools', paths: [] } }));
		put('.claude/skills/kit/SKILL.md', '# Kit, edited\n');

		expect(
			notice(repo, { number: 12, title: 'Update toolkit', url: 'https://github.com/a/b/pull/12' })
		).toEqual([
			'@acme/kit is being patched: edit it at node_modules/@acme/kit (or its copies in .claude/), ' +
				'then bun toolkit patch @acme/kit --commit, or --abort.',
			"@acme/tools runs on the toolkit's checkout, ../home/packages/tools: bun toolkit unlink " +
				'before committing.',
			'The copies in .claude/ are out of step (bun toolkit sync): skills/kit/SKILL.md differs from ' +
				'@acme/kit’s.',
			"Renovate's update of the toolkit waits for a review: #12 Update toolkit " +
				'(https://github.com/a/b/pull/12). A breaking release says in its notes what to change: ' +
				'gh pr view 12.',
		]);
	});

	it("finds Renovate's toolkit pull request when it waits for a review", () => {
		const pull = (number: number, headRefName: string, automerge: string) => ({
			number,
			title: `Update #${String(number)}`,
			url: `https://github.com/a/b/pull/${String(number)}`,
			headRefName,
			body: `📅 **Schedule**: Branch creation - At any time.\n\n🚦 **Automerge**: ${automerge}.\n`,
		});

		expect(
			waitingAmong([
				pull(1, 'feat/toolkit', 'Disabled by config'),
				pull(2, 'renovate/toolkit', 'Enabled'),
				pull(3, 'renovate/major-toolkit', 'Disabled by config'),
			])
		).toEqual({ number: 3, title: 'Update #3', url: 'https://github.com/a/b/pull/3' });
		expect(waitingAmong([pull(4, 'renovate/toolkit', 'Disabled by config')])?.number).toBe(4);
		expect(waitingAmong([pull(5, 'renovate/not-toolkit-x', 'Disabled by config')])).toBeNull();
		expect(waitingAmong([])).toBeNull();
	});

	it('names a patch made for another version', () => {
		put(
			'package.json',
			json({
				devDependencies: { '@acme/kit': '^1.0.0' },
				patchedDependencies: { '@acme/kit@0.9.0': 'patches/kit.patch' },
			})
		);

		expect(notice(repo)).toEqual([
			'A patch is out of step: patches/kit.patch was made for @acme/kit@0.9.0, and Bun drops it on ' +
				'1.0.0, which it does not apply to: make it again by hand (bun toolkit patch @acme/kit).',
		]);
	});
});
