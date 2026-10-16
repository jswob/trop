import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import { afterEach, describe, expect, it } from 'vitest';

import { DEFAULT_PORT, findConfig, resolveConfig, type WorkbenchConfig } from './config';

const config: WorkbenchConfig = {
	name: 'Ledger',
	app: { root: '..', vite: '../vite.config.ts' },
	setup: './workbench.setup.tsx',
	styles: './index.css',
	theme: {
		css: '../src/ui/theme.css',
		ground: '--color-bg',
		face: '--font-sans',
		spentIn: ['../src/ui/**/*.{ts,tsx,css}', './views/**/*.tsx'],
	},
	settled: { root: '../src/ui', alias: '@/ui', spaces: ['kit', 'views'] },
	bans: './BANS.md',
};

const file = '/repo/apps/ledger/design/workbench.config.ts';

describe('resolveConfig', () => {
	it('makes every path absolute from the folder the config stands in', () => {
		const resolved = resolveConfig(config, file, '/repo');

		expect(resolved.root).toBe('/repo/apps/ledger/design');
		expect(resolved.repo).toBe('/repo');
		expect(resolved.app).toEqual({
			root: '/repo/apps/ledger',
			vite: '/repo/apps/ledger/vite.config.ts',
			without: [],
		});
		expect(resolved.setup).toBe('/repo/apps/ledger/design/workbench.setup.tsx');
		expect(resolved.styles).toBe('/repo/apps/ledger/design/index.css');
		expect(resolved.theme.css).toBe('/repo/apps/ledger/src/ui/theme.css');
		expect(resolved.theme.spentIn).toEqual([
			'/repo/apps/ledger/src/ui/**/*.{ts,tsx,css}',
			'/repo/apps/ledger/design/views/**/*.tsx',
		]);
		expect(resolved.settled.root).toBe('/repo/apps/ledger/src/ui');
		expect(resolved.bans.file).toBe('/repo/apps/ledger/design/BANS.md');
	});

	it('fills in the port, the address, the baselines and the empty lists', () => {
		const resolved = resolveConfig(config, file, '/repo');

		expect(resolved.port).toBe(DEFAULT_PORT);
		expect(resolved.url).toBe(`http://localhost:${String(DEFAULT_PORT)}`);
		expect(resolved.baselines).toEqual({
			probes: '/repo/apps/ledger/design/probe/baseline.json',
			sweep: '/repo/apps/ledger/design/probe/sweep-baseline.json',
		});
		expect(resolved.head).toEqual([]);
		expect(resolved.stubs).toEqual([]);
		expect(resolved.gates).toEqual([]);
		expect(resolved.settled.styles).toEqual([]);
		expect(resolved.bans).toEqual({
			file: '/repo/apps/ledger/design/BANS.md',
			subjects: {},
			always: [],
		});
		expect(resolved.census).toBeUndefined();
		expect(resolved.skill).toBeUndefined();
	});

	it('takes the bans file with the words a round uses and the sections every round reads', () => {
		const resolved = resolveConfig(
			{
				...config,
				bans: { file: './BANS.md', subjects: { colour: ['hue'] }, always: ['11'] },
			},
			file,
			'/repo'
		);

		expect(resolved.bans).toEqual({
			file: '/repo/apps/ledger/design/BANS.md',
			subjects: { colour: ['hue'] },
			always: ['11'],
		});
	});

	it('serves on the port it is given', () => {
		const resolved = resolveConfig({ ...config, port: 6100 }, file, '/repo');

		expect(resolved.url).toBe('http://localhost:6100');
	});

	it('names each stub by its specifier and runs a gate in the app unless it says where', () => {
		const resolved = resolveConfig(
			{
				...config,
				stubs: { '@/lib/api': './lib/api-stub.ts' },
				gates: [
					{ name: 'types', run: ['tsc', '--noEmit'] },
					{ name: 'eslint', cwd: '../../..', run: ['eslint', '.'] },
				],
				census: { states: ['./kit/types.ts'], type: 'SlotState', hues: '--color-' },
				skill: '../../../.claude/skills/design',
			},
			file,
			'/repo'
		);

		expect(resolved.stubs).toEqual([
			{ specifier: '@/lib/api', file: '/repo/apps/ledger/design/lib/api-stub.ts' },
		]);
		expect(resolved.gates).toEqual([
			{ name: 'types', cwd: '/repo/apps/ledger', run: ['tsc', '--noEmit'] },
			{ name: 'eslint', cwd: '/repo', run: ['eslint', '.'] },
		]);
		expect(resolved.census?.states).toEqual(['/repo/apps/ledger/design/kit/types.ts']);
		expect(resolved.skill).toBe('/repo/.claude/skills/design');
	});

	it('takes the app’s root for the repository where there is none', () => {
		expect(resolveConfig(config, file).repo).toBe('/repo/apps/ledger');
	});
});

describe('findConfig', () => {
	const made: string[] = [];
	const folder = (): string => {
		const dir = mkdtempSync(path.join(os.tmpdir(), 'workbench-config-'));
		made.push(dir);
		return dir;
	};

	afterEach(() => {
		for (const dir of made.splice(0)) rmSync(dir, { recursive: true, force: true });
	});

	it('finds the config in the folder a command was run from', () => {
		const dir = folder();
		writeFileSync(path.join(dir, 'workbench.config.ts'), '');

		expect(findConfig(dir, undefined, {})).toBe(path.join(dir, 'workbench.config.ts'));
	});

	it('takes the one it is pointed at over the one the environment names', () => {
		const dir = folder();
		writeFileSync(path.join(dir, 'a.config.ts'), '');
		writeFileSync(path.join(dir, 'b.config.ts'), '');

		expect(findConfig(dir, 'a.config.ts', { WORKBENCH_CONFIG: 'b.config.ts' })).toBe(
			path.join(dir, 'a.config.ts')
		);
		expect(findConfig(dir, undefined, { WORKBENCH_CONFIG: 'b.config.ts' })).toBe(
			path.join(dir, 'b.config.ts')
		);
	});

	it('says how to name a config when there is none to find', () => {
		const dir = folder();

		expect(() => findConfig(dir, undefined, {})).toThrow(/--config <file>/);
		expect(() => findConfig(dir, 'missing.ts', {})).toThrow(/no workbench config at/);
	});
});
