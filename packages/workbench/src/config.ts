import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

/**
 * The terminal's half of the binding between the workbench and the app it draws.
 *
 * An app is bound by two files. `workbench.config.ts` is this one: plain data, read by the verbs
 * and by the server before anything is bundled, so it names files and never imports them.
 * `workbench.setup.tsx` is the browser's half (`setup.ts`): the app's own modules, handed over.
 *
 * Every path is written relative to the config file. The folder the config stands in is the
 * **design root**: where a design round may write, and where `views/`, `proto/`, `kit/` and
 * `theses/` are looked for.
 */

/** One tag in the document's head, in the shape Vite's `transformIndexHtml` takes. */
export type HeadTag = {
	tag: string;
	attrs?: Record<string, string | boolean>;
	children?: string;
};

/** One command `check` runs, and what it is called in the report. */
export type GateCommand = {
	name: string;
	/** Where it runs. The app's root when left out. */
	cwd?: string;
	run: string[];
};

export type WorkbenchConfig = {
	/** What the app is called, beside the workbench's own name. */
	name: string;
	/** The port the workbench is served on, and the only one: it refuses to start on another. */
	port?: number;
	app: {
		/** The app's root: where its commands run, and what its files are named from. */
		root: string;
		/** The app's own Vite config. The workbench takes its plugins and its resolution. */
		vite: string;
		/** Plugins of the app's that the workbench leaves out, by the start of their name. */
		without?: string[];
	};
	/** The browser's half of the binding. */
	setup: string;
	/** The stylesheet every frame loads. */
	styles: string;
	/** Tags for the document's head, where an app names the faces it loads. */
	head?: HeadTag[];
	/** Modules of the app's the workbench replaces, by the specifier they are imported with. */
	stubs?: Record<string, string>;
	theme: {
		/** The stylesheet that declares the tokens. */
		css: string;
		/** The token a frame's ground is drawn in, which every colour's contrast is measured against. */
		ground: string;
		/** The token a line of specimen type is set in. */
		face: string;
		/** The files that may spend a token, as globs. */
		spentIn: string[];
	};
	/** Where settled code lives once it is the app's own. */
	settled: {
		root: string;
		/** The specifier the app imports it by. */
		alias: string;
		/** The spaces under it a design round may write in. */
		spaces: string[];
		/** The stylesheets under it a design round may write. */
		styles?: string[];
	};
	/** The accumulated rejections: the file, or the file and how `ban find` reads it. */
	bans: string | BansConfig;
	/** What the state census reads: the union every state is a member of, and the hue tokens' prefix. */
	census?: {
		/** The files the union may be declared in. The first that exists is read. */
		states: string[];
		type: string;
		hues: string;
	};
	/** The floor and the sweep's held findings. Beside the config, in `probe/`, when left out. */
	baselines?: { probes?: string; sweep?: string };
	/** What `check` runs before the census and the blast radius. */
	gates?: GateCommand[];
	/** The design skill's copy in this repo: the one place outside the design root `check --skill` lets a round write. */
	skill?: string;
};

/** The bans file, and what `ban find` needs to know about it beyond its headings. */
export type BansConfig = {
	file: string;
	/**
	 * The words a round uses for a subject, mapped to words in the titles of the sections that
	 * hold its rules. A word not listed here is looked for in the titles as it is.
	 */
	subjects?: Record<string, string[]>;
	/** Sections printed whatever was asked for, by number: the ones about every round. */
	always?: string[];
};

/** A config with every path absolute and every default filled in. */
export type ResolvedConfig = {
	/** The config file itself. */
	file: string;
	/** The design root: the folder the config stands in. */
	root: string;
	/** The repository the design root is in, or the app's root where there is none. */
	repo: string;
	name: string;
	port: number;
	/** Where the workbench is served. */
	url: string;
	app: { root: string; vite: string; without: string[] };
	setup: string;
	styles: string;
	head: HeadTag[];
	stubs: { specifier: string; file: string }[];
	theme: { css: string; ground: string; face: string; spentIn: string[] };
	settled: { root: string; alias: string; spaces: string[]; styles: string[] };
	bans: { file: string; subjects: Record<string, string[]>; always: string[] };
	census: { states: string[]; type: string; hues: string } | undefined;
	baselines: { probes: string; sweep: string };
	gates: { name: string; cwd: string; run: string[] }[];
	skill: string | undefined;
};

export const CONFIG_NAME = 'workbench.config.ts';
export const DEFAULT_PORT = 5175;

/** Types a config where it is written. It returns what it is given. */
export function defineWorkbench(config: WorkbenchConfig): WorkbenchConfig {
	return config;
}

/**
 * A config as the engine uses it: every path absolute, every default filled in.
 *
 * `repo` is passed in so this stays a function of its arguments; `loadConfig` asks git for it.
 */
export function resolveConfig(
	config: WorkbenchConfig,
	file: string,
	repo?: string
): ResolvedConfig {
	const root = path.dirname(path.resolve(file));
	const at = (relative: string): string => path.resolve(root, relative);
	const appRoot = at(config.app.root);
	const port = config.port ?? DEFAULT_PORT;
	const bans = typeof config.bans === 'string' ? { file: config.bans } : config.bans;

	return {
		file: path.resolve(file),
		root,
		repo: repo ?? appRoot,
		name: config.name,
		port,
		url: `http://localhost:${String(port)}`,
		app: { root: appRoot, vite: at(config.app.vite), without: config.app.without ?? [] },
		setup: at(config.setup),
		styles: at(config.styles),
		head: config.head ?? [],
		stubs: Object.entries(config.stubs ?? {}).map(([specifier, stub]) => ({
			specifier,
			file: at(stub),
		})),
		theme: {
			css: at(config.theme.css),
			ground: config.theme.ground,
			face: config.theme.face,
			spentIn: config.theme.spentIn.map(at),
		},
		settled: {
			root: at(config.settled.root),
			alias: config.settled.alias,
			spaces: config.settled.spaces,
			styles: config.settled.styles ?? [],
		},
		bans: { file: at(bans.file), subjects: bans.subjects ?? {}, always: bans.always ?? [] },
		census:
			config.census === undefined
				? undefined
				: { ...config.census, states: config.census.states.map(at) },
		baselines: {
			probes: at(config.baselines?.probes ?? 'probe/baseline.json'),
			sweep: at(config.baselines?.sweep ?? 'probe/sweep-baseline.json'),
		},
		gates: (config.gates ?? []).map(gate => ({
			name: gate.name,
			cwd: gate.cwd === undefined ? appRoot : at(gate.cwd),
			run: gate.run,
		})),
		skill: config.skill === undefined ? undefined : at(config.skill),
	};
}

/**
 * Which config a command is about: the one it was pointed at, the one the environment names, or
 * the one in the folder it was run from.
 */
export function findConfig(
	cwd: string,
	explicit: string | undefined,
	env: Record<string, string | undefined> = process.env
): string {
	const named = explicit ?? env['WORKBENCH_CONFIG'];
	const file = path.resolve(cwd, named ?? CONFIG_NAME);

	if (!existsSync(file)) {
		throw new Error(
			named === undefined
				? `no ${CONFIG_NAME} in ${cwd} — run this from the design root, or name it: --config <file>`
				: `no workbench config at ${file}`
		);
	}

	return file;
}

/** The repository a folder is in, as git names it. */
function repoOf(dir: string): string | undefined {
	const result = spawnSync('git', ['rev-parse', '--show-toplevel'], { cwd: dir, encoding: 'utf8' });
	const top = result.status === 0 ? result.stdout.trim() : '';

	return top === '' ? undefined : top;
}

/** Reads a config file and resolves it. */
export async function loadConfig(file: string): Promise<ResolvedConfig> {
	const module = (await import(pathToFileURL(file).href)) as { default?: WorkbenchConfig };
	if (module.default === undefined) {
		throw new Error(`${file} has no default export — export default defineWorkbench({ … })`);
	}

	return resolveConfig(module.default, file, repoOf(path.dirname(file)));
}
