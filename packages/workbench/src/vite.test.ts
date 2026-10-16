import net, { type AddressInfo } from 'node:net';

import type { Plugin } from 'vite';
import { describe, expect, it } from 'vitest';

import { resolveConfig } from './config';
import {
	aliasList,
	aliasesWithStubs,
	generated,
	globBase,
	pluginsWithout,
	portAnswers,
	routeTreeFile,
} from './vite';

const config = resolveConfig(
	{
		name: 'Ledger',
		app: { root: '..', vite: '../vite.config.ts' },
		setup: './workbench.setup.tsx',
		styles: './index.css',
		stubs: { '@/lib/api': './lib/api-stub.ts' },
		theme: { css: '../src/ui/theme.css', ground: '--color-bg', face: '--font-sans', spentIn: [] },
		settled: { root: '../src/ui', alias: '@/ui', spaces: ['kit'] },
		bans: './BANS.md',
	},
	'/repo/apps/ledger/design/workbench.config.ts',
	'/repo'
);

const plugin = (name: string): Plugin => ({ name });

describe('aliasList', () => {
	it('reads aliases written as an object or as a list', () => {
		expect(aliasList({ '@': '/app/src' })).toEqual([{ find: '@', replacement: '/app/src' }]);
		expect(aliasList([{ find: /^@\//, replacement: '/app/src/' }])).toEqual([
			{ find: /^@\//, replacement: '/app/src/' },
		]);
		expect(aliasList(undefined)).toEqual([]);
	});
});

describe('aliasesWithStubs', () => {
	it('puts a stub before the app’s aliases and matches its specifier whole', () => {
		const [stub, ...rest] = aliasesWithStubs(config.stubs, [
			{ find: '@', replacement: '/app/src' },
		]);

		expect(stub?.replacement).toBe('/repo/apps/ledger/design/lib/api-stub.ts');
		expect(rest).toEqual([{ find: '@', replacement: '/app/src' }]);

		const find = stub?.find as RegExp;
		expect(find.test('@/lib/api')).toBe(true);
		expect(find.test('@/lib/api-client')).toBe(false);
		expect(find.test('x@/lib/api')).toBe(false);
	});
});

describe('pluginsWithout', () => {
	it('flattens presets and promises, and leaves out plugins by the start of their name', async () => {
		const kept = await pluginsWithout(
			[
				plugin('tanstack:router-generator'),
				[plugin('@tailwindcss/vite:scan'), Promise.resolve(plugin('vite:react-babel'))],
				false,
				null,
			],
			['tanstack']
		);

		expect(kept.map(each => each.name)).toEqual(['@tailwindcss/vite:scan', 'vite:react-babel']);
	});

	it('takes no plugins for none', async () => {
		expect(await pluginsWithout(undefined, [])).toEqual([]);
	});
});

describe('globBase', () => {
	it('finds the folder everything a glob matches is under', () => {
		expect(globBase('/app/src/{ui,routes}/**/*.{ts,tsx}')).toBe('/app/src');
		expect(globBase('/app/design/views/**/*.tsx')).toBe('/app/design/views');
		expect(globBase('/app/src/theme.css')).toBe('/app/src');
	});
});

describe('routeTreeFile', () => {
	it('writes beside the routes in a checkout and into the app’s cache once installed', () => {
		expect(routeTreeFile('/repo/packages/workbench/src', '/repo/apps/web')).toBe(
			'/repo/packages/workbench/src/routeTree.gen.ts'
		);
		expect(routeTreeFile('/app/node_modules/@jswob/design-workbench/src', '/app/apps/web')).toBe(
			'/app/apps/web/node_modules/.vite-workbench/routeTree.gen.ts'
		);
	});
});

describe('portAnswers', () => {
	const listening = async (host: string): Promise<net.Server> => {
		const server = net.createServer();
		await new Promise<void>(done => server.listen(0, host, done));
		return server;
	};
	const closed = (server: net.Server): Promise<void> =>
		new Promise(done => server.close(() => done()));

	it.for(['127.0.0.1', '::1'])('hears a server listening on %s', async host => {
		const server = await listening(host);
		const { port } = server.address() as AddressInfo;

		expect(await portAnswers(port)).toBe(true);

		await closed(server);
		expect(await portAnswers(port)).toBe(false);
	});
});

describe('generated', () => {
	it('hands the bench the app’s name, its two tokens and its theme as text', () => {
		const source = generated('host', config);

		expect(source).toContain(`import themeCss from "/repo/apps/ledger/src/ui/theme.css?raw";`);
		expect(source).toContain(
			`export const HOST = { ...{"name":"Ledger","ground":"--color-bg","face":"--font-sans"}, themeCss };`
		);
	});
});
