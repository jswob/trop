import net from 'node:net';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

import type { Alias, InlineConfig, Plugin, PluginOption, UserConfig, ViteDevServer } from 'vite';

import type { ResolvedConfig } from './config';
import { DESIGN_SPACES, designSources, settledSpaces, spendSources, type Source } from './sources';

/**
 * How the workbench is served: the engine's own document and routes, built with the app's own
 * Vite plugins and resolution, so a frame is compiled exactly as the app compiles it.
 *
 * The app's config is read, not merged. Its plugins, aliases and CSS options are taken; its
 * server, its `define` and its environment are not, because a workbench boots with no backend and
 * no `.env` and must not learn an API address from the app.
 */

/** The folder the engine's document, entry and routes are served from. */
export const ENGINE_SRC = import.meta.dirname;

const VIRTUAL = 'virtual:workbench/';
const GENERATED = ['host', 'sources', 'spend'] as const;
type Generated = (typeof GENERATED)[number];

const escapeRegExp = (text: string): string => text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

/** The app's aliases as a list, whichever way its config wrote them. */
export function aliasList(alias: NonNullable<UserConfig['resolve']>['alias']): Alias[] {
	if (alias === undefined) return [];
	if (Array.isArray(alias)) return alias as Alias[];

	return Object.entries(alias as Record<string, string>).map(([find, replacement]) => ({
		find,
		replacement,
	}));
}

/**
 * The stubs, then the app's aliases.
 *
 * The order is the point: a stub is matched whole and before anything the app declares, or the
 * module it replaces is bundled and the workbench can reach the backend.
 */
export function aliasesWithStubs(stubs: ResolvedConfig['stubs'], alias: Alias[]): Alias[] {
	return [
		...stubs.map(stub => ({
			find: new RegExp(`^${escapeRegExp(stub.specifier)}$`),
			replacement: stub.file,
		})),
		...alias,
	];
}

/** A plugin option as the plugins it comes to: a preset is a list, and may be a promise of one. */
async function flatPlugins(option: PluginOption): Promise<Plugin[]> {
	const settled = await option;
	if (Array.isArray(settled)) return (await Promise.all(settled.map(flatPlugins))).flat();

	return settled === null || settled === undefined || settled === false ? [] : [settled];
}

/** The app's plugins, flat, without the ones the config leaves out. */
export async function pluginsWithout(
	plugins: PluginOption[] | undefined,
	without: string[]
): Promise<Plugin[]> {
	return (await flatPlugins(plugins ?? [])).filter(
		plugin => !without.some(prefix => plugin.name.startsWith(prefix))
	);
}

type ConfigExport =
	UserConfig | ((env: Record<string, unknown>) => UserConfig | Promise<UserConfig>);

/**
 * The app's own Vite config, evaluated as a dev server would evaluate it.
 *
 * It is evaluated from the app's root, since a config may read a path relative to where it was
 * started. A config written as a function makes new plugins each time it is asked, which is what
 * lets two builds in one process each have their own.
 */
export async function appViteConfig(config: ResolvedConfig): Promise<UserConfig> {
	const from = process.cwd();
	process.chdir(config.app.root);

	try {
		const module = (await import(pathToFileURL(config.app.vite).href)) as { default: ConfigExport };
		const exported = module.default;

		return typeof exported === 'function'
			? await exported({
					command: 'serve',
					mode: 'development',
					isSsrBuild: false,
					isPreview: false,
				})
			: exported;
	} finally {
		process.chdir(from);
	}
}

const asModule = (value: unknown): string => `export default ${JSON.stringify(value)};\n`;
const asRecord = (sources: Source[]): Record<string, string> => Object.fromEntries(sources);

/** Each module the plugin writes, as source. */
export function generated(which: Generated, config: ResolvedConfig): string {
	switch (which) {
		case 'host':
			return [
				`import themeCss from ${JSON.stringify(`${config.theme.css}?raw`)};`,
				'',
				`export const HOST = { ...${JSON.stringify({
					name: config.name,
					ground: config.theme.ground,
					face: config.theme.face,
				})}, themeCss };`,
				'',
			].join('\n');
		case 'sources':
			return asModule(asRecord(designSources(config)));
		case 'spend':
			return asModule(asRecord(spendSources(config)));
	}
}

/** The part of a glob before its first wildcard: the folder everything it matches is under. */
export function globBase(pattern: string): string {
	const wild = pattern.search(/[*?{[]/);
	if (wild === -1) return path.dirname(pattern);

	return pattern.slice(0, pattern.lastIndexOf('/', wild));
}

/** The folders each module of source is read from, so a file saved in one is read again. */
function watched(config: ResolvedConfig): Record<'sources' | 'spend', string[]> {
	return {
		sources: [
			...DESIGN_SPACES.map(space => path.join(config.root, space)),
			...settledSpaces(config).map(space => space.dir),
		],
		spend: config.theme.spentIn.map(globBase),
	};
}

/**
 * The binding, as modules the engine imports by name.
 *
 * - `virtual:workbench/setup` is the app's setup file, and `virtual:workbench/styles.css` its
 *   stylesheet: both resolve to the real file, so the app's plugins compile them as their own.
 * - `virtual:workbench/host` is the app's name and theme, with the theme's stylesheet as text.
 * - `virtual:workbench/sources` is the source a caller count is read off, and
 *   `virtual:workbench/spend` the source a token's users are read off: both read in the terminal,
 *   by the function the verbs use, and read again when a file under them is saved.
 *
 * It also names the app in the document's title and adds the config's tags to its head.
 */
export function workbenchPlugin(config: ResolvedConfig): Plugin {
	const idOf = (which: Generated): string => `\0${VIRTUAL}${which}`;
	const folders = watched(config);

	return {
		name: 'workbench',
		enforce: 'pre',

		resolveId(source) {
			if (!source.startsWith(VIRTUAL)) return null;

			const name = source.slice(VIRTUAL.length);
			if (name === 'setup') return config.setup;
			if (name === 'styles.css') return config.styles;
			if ((GENERATED as readonly string[]).includes(name)) return idOf(name as Generated);

			return null;
		},

		load(id) {
			const which = GENERATED.find(name => id === idOf(name));

			return which === undefined ? null : generated(which, config);
		},

		configureServer(server: ViteDevServer) {
			const reread = (file: string): void => {
				for (const which of ['sources', 'spend'] as const) {
					if (!folders[which].some(dir => file.startsWith(`${dir}${path.sep}`))) continue;

					const module = server.moduleGraph.getModuleById(idOf(which));
					if (module !== undefined) void server.reloadModule(module);
				}
			};

			server.watcher.on('add', reread);
			server.watcher.on('change', reread);
			server.watcher.on('unlink', reread);
		},

		transformIndexHtml: {
			order: 'pre',
			handler(html) {
				return {
					html: html.replace(
						/<title>[^<]*<\/title>/,
						`<title>Design Workbench — ${config.name}</title>`
					),
					tags: config.head.map(tag => ({ ...tag, injectTo: 'head' as const })),
				};
			},
		},
	};
}

/**
 * How a module is resolved, for the bench and for a board alike: the app's own resolution, its
 * aliases behind the stubs, and one copy of what both sides must share.
 *
 * One React and one router, whoever imports them: the engine's routes and the app's components
 * meet in one tree, and a second copy of either would give a frame a router with no routes in it.
 */
export function resolveWith(
	config: ResolvedConfig,
	app: UserConfig
): NonNullable<InlineConfig['resolve']> {
	return {
		...app.resolve,
		alias: aliasesWithStubs(config.stubs, aliasList(app.resolve?.alias)),
		dedupe: [
			...new Set([...(app.resolve?.dedupe ?? []), 'react', 'react-dom', '@tanstack/react-router']),
		],
	};
}

/**
 * Where the router's generator writes the bench's route tree.
 *
 * Beside the routes while the engine is being worked on, so the tree the bench imports follows a
 * route added, renamed or removed. Once the engine is installed, into the app's own cache: a
 * generator of another version than the one that wrote the shipped tree rewrites it, and a write
 * into `node_modules` lands in Bun's shared cache wherever Bun installs by hard link.
 */
export function routeTreeFile(engineSrc: string, appRoot: string): string {
	return engineSrc.split(path.sep).includes('node_modules')
		? path.join(appRoot, 'node_modules/.vite-workbench/routeTree.gen.ts')
		: path.join(engineSrc, 'routeTree.gen.ts');
}

/**
 * The bench's own routes, split as they were written to be: a frame's document loads the frame's
 * route and not the pages of the bench around it.
 *
 * The generator stays on even where its tree is never imported: the plugin splits only the files
 * its own generator has listed, so with the generator off every frame would load the whole bench.
 * It writes the tree only when a route was added, renamed or removed.
 */
async function benchRoutes(config: ResolvedConfig): Promise<PluginOption> {
	const { tanstackRouter } = await import('@tanstack/router-plugin/vite');

	return tanstackRouter({
		target: 'react',
		autoCodeSplitting: true,
		routesDirectory: path.join(ENGINE_SRC, 'routes'),
		generatedRouteTree: routeTreeFile(ENGINE_SRC, config.app.root),
	});
}

/**
 * The config a workbench is served or built with.
 *
 * The bench's routes come before the app's plugins: the router's transform has to see a route
 * before React's does.
 */
export async function viteConfig(
	config: ResolvedConfig,
	options: { host?: boolean } = {}
): Promise<InlineConfig> {
	const { searchForWorkspaceRoot } = await import('vite');
	const app = await appViteConfig(config);

	return {
		configFile: false,
		root: ENGINE_SRC,
		cacheDir: path.join(config.app.root, 'node_modules/.vite-workbench'),
		plugins: [
			workbenchPlugin(config),
			await benchRoutes(config),
			...(await pluginsWithout(app.plugins, config.app.without)),
		],
		resolve: resolveWith(config, app),
		...(app.css === undefined ? {} : { css: app.css }),
		optimizeDeps: { entries: [path.join(ENGINE_SRC, 'index.html'), config.setup] },
		server: {
			port: config.port,
			/*
			 * A stale server holding the port used to push the new one to the next port in silence,
			 * and every tool that addresses the workbench by its port then measured the old build.
			 * Refusing to start is the loud version of the same fact. `serve` asks `portAnswers` first,
			 * because under Bun this alone lets a second server share the port.
			 */
			strictPort: true,
			...(options.host === true ? { host: true } : {}),
			fs: {
				allow: [
					path.resolve(ENGINE_SRC, '..'),
					config.root,
					config.app.root,
					searchForWorkspaceRoot(config.app.root),
				],
			},
		},
	};
}

/**
 * Whether anything already accepts a connection on the port, on either loopback address.
 *
 * `strictPort` alone does not keep a second workbench off the port under Bun: Bun binds a port
 * another process already holds, the two servers split the requests between them, and Vite never
 * hears `EADDRINUSE`.
 */
export async function portAnswers(port: number): Promise<boolean> {
	const answers = (host: string): Promise<boolean> =>
		new Promise(resolve => {
			const socket = net.connect({ port, host });
			socket.once('connect', () => {
				socket.destroy();
				resolve(true);
			});
			socket.once('error', () => resolve(false));
		});
	const heard = await Promise.all([answers('127.0.0.1'), answers('::1')]);

	return heard.includes(true);
}

/** Serves the workbench until the process is stopped. Refuses a port something else answers on. */
export async function serve(
	config: ResolvedConfig,
	options: { host?: boolean } = {}
): Promise<void> {
	if (await portAnswers(config.port)) {
		throw new Error(
			`Port ${config.port} is already in use. Stop what holds it (lsof -ti tcp:${config.port}) and start again.`
		);
	}

	const { createServer } = await import('vite');
	const server = await createServer(await viteConfig(config, options));

	await server.listen();
	server.printUrls();
}
