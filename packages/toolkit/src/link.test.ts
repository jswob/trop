import {
	existsSync,
	lstatSync,
	mkdirSync,
	mkdtempSync,
	readFileSync,
	readlinkSync,
	rmSync,
	symlinkSync,
	writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import { sync } from './claude';
import { LINK_FILE, link, linkProblems, readLinks, unlink } from './link';
import { patchProblems, startPatch, type Run } from './patches';

let root: string;
let repo: string;
let home: string;
let homeKit: string;
let calls: string[];

const json = (value: unknown) => `${JSON.stringify(value, null, '\t')}\n`;

function put(dir: string, file: string, text: string) {
	const full = path.join(dir, file);
	mkdirSync(path.dirname(full), { recursive: true });
	writeFileSync(full, text, 'utf8');
}

const read = (file: string) => readFileSync(path.join(repo, file), 'utf8');
const STORE = 'node_modules/.bun/@acme+kit@1.0.0/node_modules/@acme/kit';
const INSTALLED = ['node_modules/@acme/kit', 'apps/web/node_modules/@acme/kit'];

/** Bun's isolated layout: the root and the workspace each link the store's copy. */
function installKit() {
	for (const file of INSTALLED) {
		const full = path.join(repo, file);
		if (existsSync(full)) continue;
		mkdirSync(path.dirname(full), { recursive: true });
		symlinkSync(path.join(repo, STORE), full, 'dir');
	}
}

/** `bun install` puts back what was removed. */
const bun: Run = (command, args, cwd) => {
	calls.push([command, ...args].join(' '));
	if (args[0] === 'install' && cwd === repo) installKit();
};

const isLink = (file: string) => lstatSync(path.join(repo, file)).isSymbolicLink();
const target = (file: string) => readlinkSync(path.join(repo, file));

beforeEach(() => {
	root = mkdtempSync(path.join(tmpdir(), 'toolkit-link-'));
	repo = path.join(root, 'app');
	home = path.join(root, 'home');
	homeKit = path.join(home, 'packages/kit');
	calls = [];

	put(repo, 'package.json', json({ name: 'shop', workspaces: ['apps/*'] }));
	put(
		repo,
		'apps/web/package.json',
		json({ name: 'web', dependencies: { '@acme/kit': '^1.0.0' } })
	);
	put(
		repo,
		`${STORE}/package.json`,
		json({ name: '@acme/kit', version: '1.0.0', claude: { skills: { kit: 'skill' } } })
	);
	put(repo, `${STORE}/skill/SKILL.md`, '# Kit\n');
	installKit();

	put(home, 'package.json', json({ name: 'home', workspaces: ['packages/*'] }));
	put(
		home,
		'packages/kit/package.json',
		json({ name: '@acme/kit', version: '1.1.0', claude: { skills: { kit: 'skill' } } })
	);
	put(home, 'packages/kit/skill/SKILL.md', '# Kit, at home\n');
	put(home, 'packages/other/package.json', json({ name: '@acme/other', version: '1.0.0' }));
	mkdirSync(path.join(home, 'node_modules'));
});

afterEach(() => {
	rmSync(root, { recursive: true, force: true });
});

describe('link', () => {
	it("links the root's and each workspace's copy to the checkout, and syncs the copies from it", () => {
		sync(repo);

		const { linked } = link(repo, home);

		expect(linked).toEqual([{ name: '@acme/kit', dir: homeKit, paths: INSTALLED }]);
		for (const file of INSTALLED) {
			expect(isLink(file)).toBe(true);
			expect(target(file)).toBe(homeKit);
		}
		expect(read('.claude/skills/kit/SKILL.md')).toBe('# Kit, at home\n');
		expect(readLinks(repo)).toEqual({ '@acme/kit': { dir: homeKit, paths: INSTALLED } });
		expect(linkProblems(repo)).toEqual([
			`@acme/kit is linked to ${homeKit}: bun toolkit unlink @acme/kit before you commit`,
		]);
	});

	it('leaves the store as it was, and a package the app does not install alone', () => {
		link(repo, home);

		expect(read(`${STORE}/skill/SKILL.md`)).toBe('# Kit\n');
		expect(readLinks(repo)['@acme/other']).toBeUndefined();
	});

	it('puts back a link an install replaced, and says so until then', () => {
		link(repo, home);
		rmSync(path.join(repo, INSTALLED[1] ?? ''));
		installKit();

		expect(linkProblems(repo)).toEqual([
			`@acme/kit was linked to ${homeKit}, and an install put apps/web/node_modules/@acme/kit back: ` +
				'bun toolkit link @acme/kit again, or bun toolkit unlink @acme/kit',
		]);

		link(repo, home, ['@acme/kit']);

		expect(target(INSTALLED[1] ?? '')).toBe(homeKit);
	});

	it('refuses a checkout that is not installed', () => {
		rmSync(path.join(home, 'node_modules'), { recursive: true });

		expect(() => link(repo, home)).toThrow(
			'is not installed: a linked package takes its dependencies'
		);
		expect(isLink(INSTALLED[0] ?? '')).toBe(true);
		expect(target(INSTALLED[0] ?? '')).toBe(path.join(repo, STORE));
	});

	it('refuses a package the checkout does not hold, or the app does not install', () => {
		expect(() => link(repo, home, ['@acme/none'])).toThrow(
			`@acme/none is not a package of ${home}`
		);
		expect(() => link(repo, home, ['@acme/other'])).toThrow('@acme/other is not installed');
		expect(existsSync(path.join(repo, LINK_FILE))).toBe(false);
	});

	it('refuses a package being patched', () => {
		startPatch(repo, '@acme/kit', bun);

		expect(() => link(repo, home, ['@acme/kit'])).toThrow('@acme/kit is being patched');
	});

	it('refuses the repo itself as the checkout', () => {
		expect(() => link(repo, repo)).toThrow('is this repo');
	});
});

describe('unlink', () => {
	it('removes the links, installs the packages again and syncs their copies back', () => {
		sync(repo);
		link(repo, home);

		expect(unlink(repo, undefined, bun).unlinked).toEqual(['@acme/kit']);

		expect(calls).toEqual(['bun install']);
		for (const file of INSTALLED) expect(target(file)).toBe(path.join(repo, STORE));
		expect(read('.claude/skills/kit/SKILL.md')).toBe('# Kit\n');
		expect(readLinks(repo)).toEqual({});
		expect(linkProblems(repo)).toEqual([]);
	});

	it('does nothing when nothing is linked', () => {
		expect(unlink(repo, undefined, bun)).toEqual({ unlinked: [], synced: [] });
		expect(calls).toEqual([]);
	});

	it('refuses a package that is not linked', () => {
		expect(() => unlink(repo, ['@acme/kit'], bun)).toThrow('@acme/kit is not linked');
	});
});

it("a linked package's patch is not called stale: the link is the problem", () => {
	put(
		repo,
		'package.json',
		json({
			...(JSON.parse(read('package.json')) as object),
			patchedDependencies: { '@acme/kit@1.0.0': 'patches/@acme%2Fkit@1.0.0.patch' },
		})
	);
	link(repo, home);

	expect(patchProblems(repo)).toEqual([]);
});
