import { afterEach, describe, expect, it } from 'vitest';

import { resolveConfig } from '../config';

import { host, setHost } from './host';

afterEach(() => setHost(undefined));

describe('host', () => {
	it('refuses to guess an app when no config was loaded', () => {
		expect(() => host()).toThrow(/no workbench config is loaded/);
	});

	it('hands every verb the config the command loaded', () => {
		const config = resolveConfig(
			{
				name: 'Ledger',
				app: { root: '..', vite: '../vite.config.ts' },
				setup: './workbench.setup.tsx',
				styles: './index.css',
				theme: { css: '../src/theme.css', ground: '--bg', face: '--sans', spentIn: [] },
				settled: { root: '../src/ui', alias: '@/ui', spaces: ['kit'] },
				bans: './BANS.md',
			},
			'/repo/apps/ledger/design/workbench.config.ts',
			'/repo'
		);

		setHost(config);

		expect(host()).toBe(config);
	});
});
