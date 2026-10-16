import { existsSync, mkdirSync, readdirSync, realpathSync, statSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';

import { build, type InlineConfig, type Plugin, type PluginOption } from 'vite';

import { appViteConfig, ENGINE_SRC, pluginsWithout, resolveWith, workbenchPlugin } from '../vite';
import type { Argv } from './args';
import {
	assertBoardDir,
	drawingFiles,
	frameEntry,
	frameHtml,
	KIT_DIR,
	KIT_FILES,
	pageEntry,
	sheetHrefs,
	styleEntry,
} from './board-files';
import { host } from './host';

/**
 * Resolves a board's drawing as if it stood in the design root's `proto/`.
 *
 * A drawing file is outside the repo, so `react` has no `node_modules` above it and `../kit` points
 * at nothing. Both are resolved from where the file will stand if it is approved: a prototype
 * imports `../kit`, `../fixtures/people` and this package by name, and a drawing written with
 * exactly those lines moves into `proto/` without an edit. A file that really is beside the drawing
 * still wins, so a board can keep a helper of its own.
 */
function boardResolver(dir: string): Plugin {
	const standIn = path.join(host().root, 'proto', 'board-drawing.tsx');

	return {
		name: 'board-resolver',
		enforce: 'pre',
		async resolveId(source, importer, options) {
			if (importer === undefined || !importer.startsWith(`${dir}${path.sep}`)) return null;
			if (source.startsWith('@/') || source.startsWith('\0') || path.isAbsolute(source))
				return null;

			if (source.startsWith('.')) {
				const beside = await this.resolve(source, importer, { ...options, skipSelf: true });
				if (beside !== null) return beside;
			}

			return this.resolve(source, standIn, { ...options, skipSelf: true });
		},
	};
}

/**
 * One classic script out of one entry, built the way the workbench is served: the app's resolution
 * with the stubs ahead of its aliases, and for a frame the app's own plugins. Both come from the
 * functions the server uses (`vite.ts`), because the order of that alias list is what keeps a
 * stubbed module out of a bundle.
 *
 * The app's config is asked again for every build: a plugin holds the state of one build, and two
 * builds sharing one would each see the other's.
 */
async function kitBuild(
	dir: string,
	entry: string,
	name: string,
	file: string,
	plugins: (app: PluginOption[]) => PluginOption[]
): Promise<InlineConfig> {
	const config = host();
	const app = await appViteConfig(config);

	return {
		configFile: false,
		root: config.root,
		mode: 'production',
		logLevel: 'warn',
		publicDir: false,
		plugins: plugins(await pluginsWithout(app.plugins, config.app.without)),
		resolve: resolveWith(config, app),
		define: { 'process.env.NODE_ENV': JSON.stringify('production') },
		build: {
			outDir: dir,
			emptyOutDir: false,
			cssCodeSplit: false,
			lib: { entry, formats: ['iife'], name, fileName: () => file, cssFileName: 'frame' },
			rollupOptions: { output: { inlineDynamicImports: true } },
		},
	};
}

function size(file: string): string {
	const bytes = statSync(file).size;
	return bytes < 1024 ? `${bytes} B` : `${Math.round(bytes / 1024)} KB`;
}

/**
 * The board kit, built into a board's folder.
 *
 * It writes four files beside the board's page: `frame.js` (the kit, every frame the workbench
 * stages, the board's own drawings and the app's React), `frame.css` (the app's stylesheet, with
 * whatever classes the drawings added), `frame.html` (the document each frame opens) and `board.js`
 * (which turns a `[data-frame]` box on the page into that frame at its true size).
 *
 * **None of the four is named after the project.** A file called after one app would be the first
 * thing to rename in the next.
 *
 * It is run again after every change to a drawing: the kit is built from the working tree each
 * time, so a board never draws a stored copy of anything.
 */
export async function board(argv: Argv): Promise<number> {
	const named = argv.positional[0];
	const dir = assertBoardDir(
		named !== undefined && existsSync(named) ? realpathSync(named) : named,
		realpathSync(host().repo),
		existsSync
	);

	const drawings = drawingFiles(readdirSync(dir));
	const kit = path.join(dir, KIT_DIR);
	mkdirSync(kit, { recursive: true });
	writeFileSync(path.join(kit, 'entry.css'), styleEntry(kit, host().styles), 'utf8');
	writeFileSync(
		path.join(kit, 'entry.tsx'),
		frameEntry(path.join(ENGINE_SRC, 'lib/board-frame'), drawings),
		'utf8'
	);
	writeFileSync(
		path.join(kit, 'page.ts'),
		pageEntry(path.join(ENGINE_SRC, 'lib/board-page')),
		'utf8'
	);

	const started = Date.now();
	await build(
		await kitBuild(dir, path.join(kit, 'entry.tsx'), 'BoardFrame', 'frame.js', app => [
			workbenchPlugin(host()),
			boardResolver(dir),
			...app,
		])
	);
	await build(await kitBuild(dir, path.join(kit, 'page.ts'), 'Board', 'board.js', () => []));
	writeFileSync(path.join(dir, 'frame.html'), frameHtml(sheetHrefs(host().head)), 'utf8');
	const seconds = ((Date.now() - started) / 1000).toFixed(1);

	const what: Record<(typeof KIT_FILES)[number], string> = {
		'frame.js': `the kit, every frame the workbench stages, and ${
			drawings.length === 0 ? 'no drawings of its own' : drawings.join(', ')
		}`,
		'frame.css': "the app's own sheet, with the board's classes",
		'frame.html': 'what every frame on the board opens',
		'board.js': 'turns a [data-frame] box into that frame',
	};

	process.stdout.write(`board kit  ${dir}\n`);
	for (const file of KIT_FILES) {
		process.stdout.write(
			`  ${file.padEnd(11)} ${size(path.join(dir, file)).padStart(7)}   ${what[file]}\n`
		);
	}
	process.stdout.write(
		`\n  ${seconds}s\n\n` +
			`  A frame on the page, at its true size, then the script once at the end:\n` +
			`    <div data-frame="people:person" data-width="390" data-height="844"></div>\n` +
			`    <script src="board.js"></script>\n\n` +
			`  Publish these beside the page:\n` +
			`    ${JSON.stringify(Object.fromEntries(KIT_FILES.map(file => [file, file])))}\n\n` +
			`  Run it again after changing a drawing — the kit is built from the working tree.\n`
	);

	return 0;
}
