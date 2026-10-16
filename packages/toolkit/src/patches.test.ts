import {
	existsSync,
	mkdirSync,
	mkdtempSync,
	readFileSync,
	rmSync,
	symlinkSync,
	writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import { check, sync } from './claude';
import {
	MARKER_FILE,
	abortPatch,
	carry,
	commitPatch,
	patchProblems,
	patchStandings,
	readPatching,
	splitKey,
	startPatch,
	type Run,
} from './patches';

let repo: string;
let calls: string[];

function put(file: string, text: string) {
	const full = path.join(repo, file);
	mkdirSync(path.dirname(full), { recursive: true });
	writeFileSync(full, text, 'utf8');
}

const json = (value: unknown) => `${JSON.stringify(value, null, '\t')}\n`;
const read = (file: string) => readFileSync(path.join(repo, file), 'utf8');
const manifest = () => JSON.parse(read('package.json')) as Record<string, unknown>;

const KIT = { 'skill/SKILL.md': '# Kit\n', 'src/index.ts': 'export const x = 1;\n' };

/** What a patch made of the kit's files, once one is committed; `install` applies it. */
let patched: Record<string, string> | null;

function installKit() {
	put(
		'node_modules/@acme/kit/package.json',
		json({ name: '@acme/kit', version: '1.0.0', claude: { skills: { kit: 'skill' } } })
	);
	for (const [file, text] of Object.entries(patched ?? KIT))
		put(`node_modules/@acme/kit/${file}`, text);
}

/**
 * Bun as the lab saw it: `patch --commit` writes the patch and the key when the copy changed, and
 * `install` puts back a copy that was removed, patched when there is a patch.
 */
const bun: Run = (command, args, cwd) => {
	calls.push([command, ...args].join(' '));
	if (args[0] === 'install' && !existsSync(path.join(cwd, 'node_modules/@acme/kit'))) installKit();
	if (args[0] === 'patch' && args[1] === '--commit') {
		const now = Object.fromEntries(
			Object.keys(KIT).map(file => [file, read(`node_modules/@acme/kit/${file}`)])
		);
		if (Object.entries(KIT).every(([file, text]) => now[file] === text)) return;
		patched = now;
		put('patches/@acme%2Fkit@1.0.0.patch', 'diff --git a/src/index.ts b/src/index.ts\n');
		put(
			'package.json',
			json({
				...manifest(),
				patchedDependencies: { '@acme/kit@1.0.0': 'patches/@acme%2Fkit@1.0.0.patch' },
			})
		);
	}
};

beforeEach(() => {
	repo = mkdtempSync(path.join(tmpdir(), 'toolkit-patches-'));
	calls = [];
	patched = null;
	put('package.json', json({ workspaces: ['apps/*'], devDependencies: { '@acme/kit': '^1.0.0' } }));
	put('apps/web/package.json', json({ name: 'web', dependencies: { '@acme/kit': '^1.0.0' } }));
	installKit();
});

afterEach(() => {
	rmSync(repo, { recursive: true, force: true });
});

describe('splitKey', () => {
	it('splits a scoped and a plain key at the version', () => {
		expect(splitKey('@acme/kit@1.0.0')).toEqual({ name: '@acme/kit', version: '1.0.0' });
		expect(splitKey('kit@0.1.0-next.1')).toEqual({ name: 'kit', version: '0.1.0-next.1' });
	});

	it('refuses a key with no version', () => {
		expect(() => splitKey('@acme/kit')).toThrow('@acme/kit is not <name>@<version>');
	});
});

describe('startPatch', () => {
	it('asks Bun for the installed version and marks the package as being patched', () => {
		expect(startPatch(repo, '@acme/kit', bun)).toEqual({
			version: '1.0.0',
			dir: 'node_modules/@acme/kit',
			started: true,
		});
		expect(calls).toEqual(['bun patch @acme/kit@1.0.0']);
		expect(readPatching(repo)).toEqual({
			'@acme/kit': { version: '1.0.0', dir: 'node_modules/@acme/kit' },
		});
	});

	it('the second time, says so and runs nothing', () => {
		startPatch(repo, '@acme/kit', bun);

		expect(startPatch(repo, '@acme/kit', bun).started).toBe(false);
		expect(calls).toHaveLength(1);
	});

	it('refuses a package that is not installed', () => {
		expect(() => startPatch(repo, '@acme/none', bun)).toThrow('@acme/none is not installed');
	});

	it('refuses a package that is a workspace of the repo', () => {
		put('packages/own/package.json', json({ name: '@acme/own', version: '1.0.0' }));
		mkdirSync(path.join(repo, 'node_modules/@acme'), { recursive: true });
		symlinkSync(path.join(repo, 'packages/own'), path.join(repo, 'node_modules/@acme/own'));

		expect(() => startPatch(repo, '@acme/own', bun)).toThrow(
			"@acme/own is this repo's own source at packages/own, or linked to it: edit it there"
		);
		expect(calls).toEqual([]);
	});

	it('refuses a package installed at two versions', () => {
		put(
			'apps/web/node_modules/@acme/kit/package.json',
			json({ name: '@acme/kit', version: '2.0.0' })
		);

		expect(() => startPatch(repo, '@acme/kit', bun)).toThrow(
			'several versions of @acme/kit are installed (1.0.0, 2.0.0)'
		);
	});
});

describe('commitPatch', () => {
	it('writes the patch, clears the mark, and syncs the copies', () => {
		sync(repo);
		startPatch(repo, '@acme/kit', bun);
		put('node_modules/@acme/kit/src/index.ts', 'export const x = 2;\n');

		const { patch, adopted } = commitPatch(repo, '@acme/kit', bun);

		expect(patch).toBe('patches/@acme%2Fkit@1.0.0.patch');
		expect(adopted).toEqual([]);
		expect(calls.slice(-2)).toEqual(['bun patch --commit node_modules/@acme/kit', 'bun install']);
		expect(read('node_modules/@acme/kit/src/index.ts')).toBe('export const x = 2;\n');
		expect(existsSync(path.join(repo, MARKER_FILE))).toBe(false);
	});

	it('takes an edit made to a copy in .claude/ into the patch', () => {
		sync(repo);
		startPatch(repo, '@acme/kit', bun);
		put('.claude/skills/kit/SKILL.md', '# Kit, fixed in the app\n');

		const { patch, adopted, synced } = commitPatch(repo, '@acme/kit', bun);

		expect(adopted).toEqual(['skills/kit/SKILL.md']);
		expect(read('node_modules/@acme/kit/skill/SKILL.md')).toBe('# Kit, fixed in the app\n');
		expect(patch).toBe('patches/@acme%2Fkit@1.0.0.patch');
		expect(synced.flatMap(s => s.write)).toEqual([]);
		expect(check(repo)).toEqual([]);
	});

	it('refuses to take copies synced at another version, and changes nothing', () => {
		sync(repo);
		put('.claude/toolkit-lock.json', read('.claude/toolkit-lock.json').replace('1.0.0', '0.9.0'));
		startPatch(repo, '@acme/kit', bun);
		put('.claude/skills/kit/SKILL.md', '# Kit, stale\n');

		expect(() => commitPatch(repo, '@acme/kit', bun)).toThrow(
			'the copies of @acme/kit in .claude/ were synced at 0.9.0, not at 1.0.0'
		);
		expect(read('node_modules/@acme/kit/skill/SKILL.md')).toBe('# Kit\n');
		expect(readPatching(repo)['@acme/kit']).toBeDefined();
	});

	it('says there is no patch when nothing changed', () => {
		startPatch(repo, '@acme/kit', bun);

		expect(commitPatch(repo, '@acme/kit', bun).patch).toBeNull();
		expect(readPatching(repo)).toEqual({});
	});

	it('refuses a package that is not being patched', () => {
		expect(() => commitPatch(repo, '@acme/kit', bun)).toThrow(
			'@acme/kit is not being patched: bun toolkit patch @acme/kit'
		);
	});
});

describe('abortPatch', () => {
	it('removes the edited copy, installs the package again and clears the mark', () => {
		startPatch(repo, '@acme/kit', bun);
		put('node_modules/@acme/kit/src/index.ts', 'export const x = 2;\n');

		abortPatch(repo, '@acme/kit', bun);

		expect(read('node_modules/@acme/kit/src/index.ts')).toBe('export const x = 1;\n');
		expect(calls.at(-1)).toBe('bun install');
		expect(readPatching(repo)).toEqual({});
	});
});

/** A diff of one line of a kit file, as Bun writes it. */
const diff = (file: string, before: string, after: string) =>
	`diff --git a/${file} b/${file}\n--- a/${file}\n+++ b/${file}\n@@ -1 +1 @@\n-${before}\n+${after}\n`;

/** Made on a version where x was 0; 1.0.0 has the fix. */
const FIXED = diff('src/index.ts', 'export const x = 0;', 'export const x = 1;');
/** Made on a version with the same skill as 1.0.0, which still lacks the fix. */
const APPLIES = diff('skill/SKILL.md', '# Kit', '# Kit, fixed');
/** Made on a version whose x 1.0.0 changed again. */
const CONFLICTS = diff('src/index.ts', 'export const x = 5;', 'export const x = 6;');

/** Patches made for versions before the installed 1.0.0: 0.9.0, 0.9.1 and so on, in order. */
function older(patches: Record<string, string>) {
	const keys = Object.keys(patches).map((name, index) => {
		put(`patches/${name}.patch`, patches[name] ?? '');
		return [`@acme/kit@0.9.${String(index)}`, `patches/${name}.patch`] as const;
	});
	put('package.json', json({ ...manifest(), patchedDependencies: Object.fromEntries(keys) }));
}

describe('patchStandings', () => {
	it('holds a patch for another version against the installed one', () => {
		older({ fixed: FIXED, applies: APPLIES, conflicts: CONFLICTS });

		expect(patchStandings(repo).map(({ file, standing }) => [file, standing])).toEqual([
			['patches/fixed.patch', 'fixed'],
			['patches/applies.patch', 'applies'],
			['patches/conflicts.patch', 'conflicts'],
		]);
		expect(read('node_modules/@acme/kit/src/index.ts')).toBe('export const x = 1;\n');
	});

	it('finds a copy the way the workspaces resolve it, under the isolated linker too', () => {
		rmSync(path.join(repo, 'node_modules/@acme'), { recursive: true });
		const store = 'node_modules/.bun/@acme+kit@1.0.0/node_modules/@acme/kit';
		put(`${store}/package.json`, json({ name: '@acme/kit', version: '1.0.0' }));
		put(`${store}/src/index.ts`, 'export const x = 1;\n');
		mkdirSync(path.join(repo, 'apps/web/node_modules/@acme'), { recursive: true });
		symlinkSync(path.join(repo, store), path.join(repo, 'apps/web/node_modules/@acme/kit'));
		older({ fixed: FIXED });

		expect(patchStandings(repo)[0]?.standing).toBe('fixed');
	});
});

describe('carry', () => {
	it('deletes a patch the installed version has the fix of', () => {
		older({ fixed: FIXED });

		const { dropped, carried, stuck } = carry(repo, bun);

		expect(dropped.map(patch => patch.file)).toEqual(['patches/fixed.patch']);
		expect([carried, stuck]).toEqual([[], []]);
		expect(existsSync(path.join(repo, 'patches/fixed.patch'))).toBe(false);
		expect(manifest().patchedDependencies).toBeUndefined();
		expect(calls).toEqual(['bun install']);
	});

	it('makes a patch that still applies again on the installed version, and syncs its copies', () => {
		sync(repo);
		older({ applies: APPLIES });

		const { carried } = carry(repo, bun);

		expect(carried.map(({ from, patch }) => [from.file, patch])).toEqual([
			['patches/applies.patch', 'patches/@acme%2Fkit@1.0.0.patch'],
		]);
		expect(calls).toEqual([
			'bun patch @acme/kit@1.0.0',
			'bun patch --commit node_modules/@acme/kit',
			'bun install',
		]);
		expect(existsSync(path.join(repo, 'patches/applies.patch'))).toBe(false);
		expect(manifest().patchedDependencies).toEqual({
			'@acme/kit@1.0.0': 'patches/@acme%2Fkit@1.0.0.patch',
		});
		expect(read('.claude/skills/kit/SKILL.md')).toBe('# Kit, fixed\n');
		expect(check(repo)).toEqual([]);
	});

	it('leaves a patch that does not apply, and changes nothing for it', () => {
		older({ fixed: FIXED, conflicts: CONFLICTS });

		const { dropped, stuck } = carry(repo, bun);

		expect(dropped).toHaveLength(1);
		expect(stuck.map(patch => patch.file)).toEqual(['patches/conflicts.patch']);
		expect(manifest().patchedDependencies).toEqual({
			'@acme/kit@0.9.1': 'patches/conflicts.patch',
		});
		expect(read('patches/conflicts.patch')).toBe(CONFLICTS);
	});

	it('does nothing when every patch is for the installed version', () => {
		put(
			'package.json',
			json({ ...manifest(), patchedDependencies: { '@acme/kit@1.0.0': 'patches/kit.patch' } })
		);

		expect(carry(repo, bun)).toEqual({ dropped: [], carried: [], stuck: [] });
		expect(calls).toEqual([]);
	});

	it('refuses while a patch is being made', () => {
		startPatch(repo, '@acme/kit', bun);

		expect(() => carry(repo, bun)).toThrow(
			'@acme/kit is being patched: --commit or --abort it first'
		);
	});
});

describe('patchProblems', () => {
	const patched = (key: string) =>
		put(
			'package.json',
			json({ ...manifest(), patchedDependencies: { [key]: `patches/${key}.patch` } })
		);

	it('is empty when each patch is for the installed version', () => {
		patched('@acme/kit@1.0.0');

		expect(patchProblems(repo)).toEqual([]);
	});

	it('names a patch made for a version no longer installed, and what carry does with it', () => {
		older({ fixed: FIXED, applies: APPLIES, conflicts: CONFLICTS });

		expect(patchProblems(repo)).toEqual([
			'patches/fixed.patch was made for @acme/kit@0.9.0, and Bun drops it on 1.0.0, which has ' +
				'the fix: bun toolkit carry deletes it',
			'patches/applies.patch was made for @acme/kit@0.9.1, and Bun drops it on 1.0.0, which it ' +
				'still applies to: bun toolkit carry makes it again',
			'patches/conflicts.patch was made for @acme/kit@0.9.2, and Bun drops it on 1.0.0, which it ' +
				'does not apply to: make it again by hand (bun toolkit patch @acme/kit)',
		]);
	});

	it('names a patch for a package no longer installed', () => {
		patched('@acme/gone@1.0.0');

		expect(patchProblems(repo)).toEqual([
			'patches/@acme/gone@1.0.0.patch: @acme/gone is not installed',
		]);
	});

	it('names a patch left half made', () => {
		startPatch(repo, '@acme/kit', bun);

		expect(patchProblems(repo)).toEqual([
			'@acme/kit is being patched: bun toolkit patch @acme/kit --commit, or --abort',
		]);
	});
});
