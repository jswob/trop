import { mkdirSync, mkdtempSync, realpathSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import { afterEach, describe, expect, it } from 'vitest';

import { resolveConfig, type ResolvedConfig } from './config';
import { designSources, nameOf, settledSpaces, sourcesUnder, spendSources } from './sources';

const made: string[] = [];

/** A repo with one app in it, written to a temporary folder, and its config. */
function app(files: Record<string, string>): ResolvedConfig {
	const repo = realpathSync(mkdtempSync(path.join(os.tmpdir(), 'workbench-sources-')));
	made.push(repo);

	for (const [file, text] of Object.entries(files)) {
		const full = path.join(repo, 'apps/ledger', file);
		mkdirSync(path.dirname(full), { recursive: true });
		writeFileSync(full, text);
	}

	return resolveConfig(
		{
			name: 'Ledger',
			app: { root: '..', vite: '../vite.config.ts' },
			setup: './workbench.setup.tsx',
			styles: './index.css',
			theme: {
				css: '../src/ui/theme.css',
				ground: '--color-bg',
				face: '--font-sans',
				spentIn: ['../src/{ui,routes}/**/*.{ts,tsx,css}', './{views,kit}/**/*.{ts,tsx}'],
			},
			settled: { root: '../src/ui', alias: '@/ui', spaces: ['kit', 'views'] },
			bans: './BANS.md',
		},
		path.join(repo, 'apps/ledger/design/workbench.config.ts'),
		repo
	);
}

afterEach(() => {
	for (const dir of made.splice(0)) rmSync(dir, { recursive: true, force: true });
});

describe('sourcesUnder', () => {
	it('reads every script under a folder, named from the root it was given', () => {
		const config = app({
			'design/kit/Mark.tsx': 'mark',
			'design/kit/deep/Seat.ts': 'seat',
			'design/kit/notes.md': 'not a script',
		});

		expect(sourcesUnder('kit', config.root).sort()).toEqual([
			['kit/Mark.tsx', 'mark'],
			['kit/deep/Seat.ts', 'seat'],
		]);
	});

	it('finds nothing in a folder that is not there', () => {
		expect(sourcesUnder('subjects', app({}).root)).toEqual([]);
	});
});

describe('nameOf', () => {
	it('names a file of the design root from it, and any other from the repo’s root', () => {
		const config = app({});

		expect(nameOf(path.join(config.root, 'views/people.tsx'), config)).toBe('views/people.tsx');
		expect(nameOf(path.join(config.app.root, 'src/ui/kit/Mark.tsx'), config)).toBe(
			'apps/ledger/src/ui/kit/Mark.tsx'
		);
	});

	it('names a UI package beside the app from the repo’s root, never with a climb', () => {
		const config = app({});

		expect(nameOf(path.join(config.repo, 'packages/ui/src/kit/Mark.tsx'), config)).toBe(
			'packages/ui/src/kit/Mark.tsx'
		);
	});
});

describe('settledSpaces', () => {
	it('gives each settled space its folder and the name its files go by', () => {
		const config = app({});

		expect(settledSpaces(config)).toEqual([
			{ dir: path.join(config.app.root, 'src/ui/kit'), name: 'apps/ledger/src/ui/kit' },
			{ dir: path.join(config.app.root, 'src/ui/views'), name: 'apps/ledger/src/ui/views' },
		]);
	});
});

describe('designSources', () => {
	const files = {
		'design/views/people.tsx': 'people',
		'design/proto/split.tsx': 'split',
		'design/kit/Sheet.tsx': 'sheet',
		'design/fixtures/people.ts': 'not a space',
		'src/ui/kit/Mark.tsx': 'mark',
		'src/ui/views/frame.tsx': 'frame',
		'src/ui/adapters/people.ts': 'not a settled space',
	};

	it('reads the design root’s spaces, then the settled ones under the repo’s names', () => {
		expect(designSources(app(files)).map(([name]) => name)).toEqual([
			'views/people.tsx',
			'proto/split.tsx',
			'kit/Sheet.tsx',
			'apps/ledger/src/ui/kit/Mark.tsx',
			'apps/ledger/src/ui/views/frame.tsx',
		]);
	});

	it('narrows the design root’s spaces only: a settled file is always read', () => {
		expect(designSources(app(files), ['kit']).map(([name]) => name)).toEqual([
			'kit/Sheet.tsx',
			'apps/ledger/src/ui/kit/Mark.tsx',
			'apps/ledger/src/ui/views/frame.tsx',
		]);
	});

	it('leaves out a component’s specimen beside it: its page is documentation, not a use', () => {
		const config = app({
			'design/kit/Sheet.tsx': 'sheet',
			'design/kit/Sheet.specimen.tsx': 'its page',
			'src/ui/kit/Mark.tsx': 'mark',
			'src/ui/kit/Mark.specimen.tsx': 'its page',
		});

		expect(designSources(config).map(([name]) => name)).toEqual([
			'kit/Sheet.tsx',
			'apps/ledger/src/ui/kit/Mark.tsx',
		]);
	});
});

describe('spendSources', () => {
	it('reads every file that may spend a token, without the tests and the sheet that declares them', () => {
		const config = app({
			'src/ui/theme.css': '--color-bg: #000',
			'src/ui/scroll.css': 'scroll',
			'src/ui/kit/Mark.tsx': 'mark',
			'src/ui/kit/Mark.test.tsx': 'a test',
			'src/routes/people.tsx': 'route',
			'src/hooks/people.ts': 'not in the list',
			'design/views/people.tsx': 'people',
			'design/kit/Sheet.tsx': 'sheet',
			'design/proto/split.tsx': 'not in the list',
		});

		expect(spendSources(config)).toEqual([
			['apps/ledger/src/routes/people.tsx', 'route'],
			['apps/ledger/src/ui/kit/Mark.tsx', 'mark'],
			['apps/ledger/src/ui/scroll.css', 'scroll'],
			['kit/Sheet.tsx', 'sheet'],
			['views/people.tsx', 'people'],
		]);
	});
});
