/**
 * `bun setup <name>`: the half of `scripts/setup.ts` that only the template has. It turns a clone of
 * the template into an app, and deletes this folder with the rest of what only the template has.
 *
 * - **Names.** The root `package.json`, the workspaces' scope (`@app/` becomes `@<name>/`), the
 *   database names and `COMPOSE_PROJECT_NAME`, the Fly app, the page title and the README heading.
 * - **Ports.** The databases take three ports from the name, and the dev servers four (the API, the
 *   web, the web under e2e, the workbench), so apps on one machine do not meet: a second Bun server on
 *   a port shares it with the first, without an error, and each answers half the requests.
 * - **The packages by version.** Every `@jswob/*` workspace is deleted, and every dependency on one
 *   becomes a caret range on the version it has here; the scripts that ran it go too.
 * - **What only the template has.** This folder, `TEMPLATE.md`, the packages' release (changesets,
 *   its workflow, its script and their dependencies), the Renovate presets, and every block between
 *   a `template:start` and a `template:end` marker line.
 * - **The app's settings.** `renovate.json` on one of the presets, `toolkit.home` for the toolkit's
 *   checkout, `toolkit.agnostic` on the app's patches, and the template commit it came from (found
 *   in that checkout by its tree, since GitHub's "Use this template" makes a commit of its own).
 *
 * Then it installs, syncs `.claude/`, and formats what it changed.
 */
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { parseArgs } from 'node:util';

/** The template's repository, which the app's `renovate.json` and `template` field name. */
export const TEMPLATE_REPO = 'jswob/bun-monorepo';

export const PRESETS = ['auto', 'manual', 'next', 'off'] as const;
export type Preset = (typeof PRESETS)[number];

export type Options = {
	/** Lowercase letters, digits and hyphens, starting with a letter. */
	name: string;
	title: string;
	/** The dev database's port; the test and e2e ones are the next two. */
	dbPort: number;
	/** The API's dev port; the web, the web under e2e and the workbench take the next three. */
	port: number;
	renovate: Preset;
	/** The toolkit's checkout, relative to the app. */
	home: string;
};

/** What only the template has, deleted whole. */
const TEMPLATE_ONLY = [
	'template',
	'TEMPLATE.md',
	'.changeset',
	'renovate',
	'scripts/release.ts',
	'scripts/release.test.ts',
	'.github/workflows/release.yml',
];

/** Root scripts only the template runs. */
const TEMPLATE_SCRIPTS = [
	'changeset',
	'release:publish',
	'release:version',
	'scripts:test',
	'template:test',
	'template:typecheck',
];

/** Root devDependencies only the packages' release uses. */
const TEMPLATE_DEPENDENCIES = ['@changesets/cli', 'semver', '@types/semver'];

const DEPENDENCY_FIELDS = [
	'dependencies',
	'devDependencies',
	'optionalDependencies',
	'peerDependencies',
] as const;

const NAME = /^[a-z][a-z0-9-]*$/;

/** `my-app` → `My App`. */
export function defaultTitle(name: string): string {
	return name
		.split('-')
		.filter(part => part !== '')
		.map(part => part.charAt(0).toUpperCase() + part.slice(1))
		.join(' ');
}

/** One of a hundred slots, from the name. */
function slot(name: string): number {
	return createHash('sha256').update(name).digest().readUInt16BE(0) % 100;
}

/** A port for the dev database from the name: one of a hundred, three apart, from 5500. */
export function defaultDbPort(name: string): number {
	return 5500 + 3 * slot(name);
}

/**
 * A port for the API from the name, in the same slot as the database's: one of a hundred, four
 * apart, from 3100. Clear of the template's own (3001, 5173 to 5175) and of every port a browser
 * refuses.
 */
export function defaultPort(name: string): number {
	return 3100 + 4 * slot(name);
}

/** The template's dev ports, in the order an app's take their place: API, web, e2e web, workbench. */
export const TEMPLATE_PORTS = [3001, 5173, 5174, 5175] as const;
const PORT_WORD = new RegExp(`\\b(?:${TEMPLATE_PORTS.join('|')})\\b`, 'g');

const USAGE = `Usage: bun setup <name> [options]

Turns this clone of the template into the app <name> (lowercase letters, digits and hyphens).

  --title <title>                    The app's name in words (default: from <name>)
  --db-port <port>                   The dev database's port; test and e2e take the next two
                                     (default: one from <name>)
  --port <port>                      The API's dev port; the web, the e2e web and the workbench
                                     take the next three (default: one from <name>)
  --renovate auto|manual|next|off    The toolkit's Renovate preset (default: auto)
  --home <dir>                       The toolkit's checkout (default: ../bun-monorepo)
  --no-install                       Leave out bun install, the sync and the formatting`;

/** The options a command line gives, or the reason it gives none. */
export function parse(argv: string[]): (Options & { install: boolean }) | { error: string } {
	let parsed;
	try {
		parsed = parseArgs({
			args: argv,
			allowPositionals: true,
			options: {
				title: { type: 'string' },
				'db-port': { type: 'string' },
				port: { type: 'string' },
				renovate: { type: 'string', default: 'auto' },
				home: { type: 'string', default: '../bun-monorepo' },
				'no-install': { type: 'boolean', default: false },
			},
		});
	} catch (error) {
		return { error: (error as Error).message };
	}
	const { values, positionals } = parsed;
	const [name, ...rest] = positionals;
	if (name === undefined || rest.length > 0) return { error: 'name the app, once' };
	if (!NAME.test(name))
		return { error: `${name}: lowercase letters, digits and hyphens, from a letter` };
	if (name === 'app') return { error: 'app is the template’s own name: pick the app’s' };
	if (!PRESETS.includes(values.renovate as Preset)) {
		return { error: `--renovate ${values.renovate}: one of ${PRESETS.join(', ')}` };
	}
	const dbPort = values['db-port'] === undefined ? defaultDbPort(name) : Number(values['db-port']);
	if (!Number.isInteger(dbPort) || dbPort < 1024 || dbPort > 65533) {
		return {
			error: `--db-port ${String(values['db-port'])}: a port from 1024, with two free after it`,
		};
	}
	const port = values.port === undefined ? defaultPort(name) : Number(values.port);
	if (!Number.isInteger(port) || port < 1024 || port > 65532) {
		return { error: `--port ${String(values.port)}: a port from 1024, with three free after it` };
	}
	return {
		name,
		title: values.title ?? defaultTitle(name),
		dbPort,
		port,
		renovate: values.renovate as Preset,
		home: values.home,
		install: !values['no-install'],
	};
}

const MARKER = /^\s*(?:<!--|#|\/\/)\s*template:(start|end)\s*(?:-->)?\s*$/;

/**
 * The text without its template-only blocks: every line from a `template:start` marker line through
 * the next `template:end` one. Where a block stood between two blank lines, one of them goes too.
 * @param text - The file's text
 * @param file - Its path, for the error on an unmatched marker
 */
export function stripTemplate(text: string, file: string): string {
	const lines = text.split('\n');
	const kept: string[] = [];
	for (let at = 0; at < lines.length; at++) {
		const line = lines[at] ?? '';
		const marker = MARKER.exec(line)?.[1];
		if (marker === undefined) {
			kept.push(line);
			continue;
		}
		if (marker === 'end') throw new Error(`${file}:${String(at + 1)}: template:end with no start`);
		const end = lines.findIndex((line, index) => index > at && MARKER.test(line));
		if (end < 0 || MARKER.exec(lines[end] ?? '')?.[1] !== 'end') {
			throw new Error(
				`${file}:${String(at + 1)}: template:start with no end before the next marker`
			);
		}
		at = end;
		if (kept.at(-1)?.trim() === '' && lines[at + 1]?.trim() === '') at++;
	}
	return kept.join('\n');
}

type Manifest = {
	name?: string;
	version?: string;
	scripts?: Record<string, string>;
	toolkit?: { agnostic?: { paths?: string[]; words?: string[] }; home?: string };
	template?: { repo: string; commit?: string; tree?: string };
} & Partial<Record<(typeof DEPENDENCY_FIELDS)[number], Record<string, string>>>;

const readJson = (file: string) => JSON.parse(readFileSync(file, 'utf8')) as Manifest;
const writeJson = (file: string, value: unknown) =>
	writeFileSync(file, `${JSON.stringify(value, null, '\t')}\n`, 'utf8');

function git(root: string, args: string[]): string | null {
	const run = spawnSync('git', args, { cwd: root, encoding: 'utf8' });
	return run.status === 0 ? run.stdout.trim() : null;
}

/**
 * The template's commit with this tree, from the toolkit's checkout; none when the checkout is not
 * there or does not have it. A clone's own commit is one, and a repo made with GitHub's "Use this
 * template" has a commit of its own with the same tree.
 * @param home - The toolkit's checkout, absolute
 * @param tree - The clone's tree
 */
export function templateCommit(home: string, tree: string): string | undefined {
	if (!existsSync(home)) return undefined;
	const log = git(home, ['log', '--all', '--format=%H %T']) ?? '';
	return log
		.split('\n')
		.find(line => line.endsWith(` ${tree}`))
		?.split(' ')[0];
}

/** The workspace manifests under `apps/` and `packages/`, relative to the root. */
function workspaceManifests(root: string): string[] {
	return ['apps', 'packages'].flatMap(parent => {
		const dir = path.join(root, parent);
		if (!existsSync(dir)) return [];
		return readdirSync(dir, { withFileTypes: true })
			.filter(
				entry => entry.isDirectory() && existsSync(path.join(dir, entry.name, 'package.json'))
			)
			.map(entry => `${parent}/${entry.name}/package.json`);
	});
}

function renovateJson(preset: Preset): string {
	return `{
	"$schema": "https://docs.renovatebot.com/renovate-schema.json",
	"extends": [
		"config:best-practices",
		"github>${TEMPLATE_REPO}//renovate/third-party",
		"github>${TEMPLATE_REPO}//renovate/toolkit-${preset}"
	],
	"packageRules": [
		{
			"description": "Bun's pin in package.json and its image in the Dockerfile move together",
			"matchDepNames": ["bun", "oven/bun"],
			"groupName": "bun"
		}
	]
}
`;
}

/** What `makeApp` did, for the summary and for the formatting after the install. */
export type Made = {
	packages: { name: string; dir: string; range: string }[];
	deleted: string[];
	changed: string[];
};

/**
 * Turns the clone at `root` into the app. Files only: no install, no formatting.
 * @param root - The clone's root
 * @param options - The app's name, title, ports, preset and the toolkit's home
 */
export function makeApp(root: string, options: Options): Made {
	if (!existsSync(path.join(root, 'template', 'setup.ts'))) {
		throw new Error(`${root} is not a clone of the template: it has no template/setup.ts`);
	}
	const status = git(root, ['status', '--porcelain']);
	if (status === null) throw new Error(`${root} is not a git checkout`);
	if (status !== '')
		throw new Error('commit or stash the changes first: setup changes most of the files');
	const tree = git(root, ['rev-parse', 'HEAD^{tree}']) ?? undefined;
	const commit =
		tree === undefined ? undefined : templateCommit(path.resolve(root, options.home), tree);

	const lock = path.join(root, '.claude', 'toolkit-lock.json');
	const copies = new Set(
		existsSync(lock)
			? Object.values(
					(
						JSON.parse(readFileSync(lock, 'utf8')) as {
							packages: Record<string, { files: string[] }>;
						}
					).packages
				).flatMap(entry => entry.files.map(file => `.claude/${file}`))
			: []
	);

	// The packages, by the versions they have here
	const packages = workspaceManifests(root)
		.map(file => ({ file, manifest: readJson(path.join(root, file)) }))
		.flatMap(({ file, manifest: { name, version } }) =>
			name?.startsWith('@jswob/') === true && version !== undefined
				? [{ name, dir: path.dirname(file), range: `^${version}` }]
				: []
		);
	const deleted = [...packages.map(entry => entry.dir), ...TEMPLATE_ONLY].filter(entry =>
		existsSync(path.join(root, entry))
	);
	for (const entry of deleted) rmSync(path.join(root, entry), { recursive: true, force: true });

	const changed = new Set<string>();
	const ranges = new Map(packages.map(entry => [entry.name, entry.range]));
	for (const file of ['package.json', ...workspaceManifests(root)]) {
		const manifest = readJson(path.join(root, file));
		for (const field of DEPENDENCY_FIELDS) {
			const dependencies = manifest[field] ?? {};
			for (const [name, spec] of Object.entries(dependencies)) {
				const range = ranges.get(name);
				if (range !== undefined && spec.startsWith('workspace:')) dependencies[name] = range;
			}
		}
		if (file === 'package.json') {
			manifest.name = options.name;
			// A script goes with what it runs: a word of its command that is a deleted path, or in one
			const words = (command: string) => command.split(/[\s'"=]+/);
			const scripts = manifest.scripts ?? {};
			for (const [script, command] of Object.entries(scripts)) {
				const gone =
					TEMPLATE_SCRIPTS.includes(script) ||
					words(command).some(word =>
						deleted.some(entry => word === entry || word.startsWith(`${entry}/`))
					);
				if (gone) delete scripts[script];
			}
			for (const name of TEMPLATE_DEPENDENCIES) delete manifest.devDependencies?.[name];
			manifest.toolkit = {
				...manifest.toolkit,
				agnostic: { paths: ['patches'], words: manifest.toolkit?.agnostic?.words ?? [] },
				home: options.home,
			};
			manifest.template = { repo: TEMPLATE_REPO, commit, tree };
		}
		writeJson(path.join(root, file), manifest);
		changed.add(file);
	}

	// Every text file: the names, the template-only blocks, then the dev ports (every mention of one
	// outside the lock is one: the configs, their fallbacks, the docs)
	const snake = options.name.replaceAll('-', '_');
	const ports = new Map<number, number>(
		TEMPLATE_PORTS.map((port, index) => [port, options.port + index])
	);
	const listed = git(root, ['ls-files', '-z', '--cached', '--others', '--exclude-standard']) ?? '';
	for (const file of listed.split('\0')) {
		const full = path.join(root, file);
		if (file === '' || copies.has(file) || !existsSync(full)) continue;
		const text = readFileSync(full, 'utf8');
		if (text.includes('\0')) continue;
		let next = text
			.replaceAll('@app/', `@${options.name}/`)
			.replaceAll('@app\\/', `@${options.name}\\/`)
			.replace(/\bapp_(test_|e2e_)?db\b/g, `${snake}_$1db`)
			.replace(/\bapp-api\b/g, `${options.name}-api`);
		if (file !== 'bun.lock') {
			next = stripTemplate(next, file).replace(PORT_WORD, word => String(ports.get(Number(word))));
		}
		if (next !== text) {
			writeFileSync(full, next, 'utf8');
			changed.add(file);
		}
	}

	const edit = (file: string, change: (text: string) => string) => {
		const full = path.join(root, file);
		if (!existsSync(full)) return;
		writeFileSync(full, change(readFileSync(full, 'utf8')), 'utf8');
		changed.add(file);
	};
	edit('.env.example', text =>
		text
			.replace(/^DB_PORT=.*$/m, `DB_PORT=${String(options.dbPort)}`)
			.replace(/^TEST_DB_PORT=.*$/m, `TEST_DB_PORT=${String(options.dbPort + 1)}`)
			.replace(/^E2E_DB_PORT=.*$/m, `E2E_DB_PORT=${String(options.dbPort + 2)}`)
			.replace(/^COMPOSE_PROJECT_NAME=.*$/m, `COMPOSE_PROJECT_NAME=${options.name}`)
	);
	const html = options.title.replace(/[&<>]/g, char => `&#${String(char.charCodeAt(0))};`);
	edit('apps/web/index.html', text =>
		text.replace(/<title>[^<]*<\/title>/, `<title>${html}</title>`)
	);
	edit('README.md', text => text.replace(/^# .*$/m, `# ${options.title}`));
	writeFileSync(path.join(root, 'renovate.json'), renovateJson(options.renovate), 'utf8');
	changed.add('renovate.json');

	return { packages, deleted, changed: [...changed].sort() };
}

function run(root: string, [command, ...args]: [string, ...string[]]): boolean {
	return spawnSync(command, args, { cwd: root, stdio: 'inherit' }).status === 0;
}

/** Installs, syncs `.claude/` and formats what setup changed; `false` at the first step that fails. */
function finish(root: string, made: Made): boolean {
	if (!run(root, ['bun', 'install'])) return false;
	if (!run(root, ['bun', 'toolkit', 'sync'])) return false;
	const files = made.changed.filter(
		file => file !== 'bun.lock' && existsSync(path.join(root, file))
	);
	const scripts = files.filter(file => /\.(?:[cm]?[jt]sx?)$/.test(file));
	// The scope's new name sorts its imports elsewhere, and a longer one can wrap a line
	if (
		scripts.length > 0 &&
		!run(root, ['bun', 'x', 'eslint', '--fix', '--no-warn-ignored', ...scripts])
	) {
		return false;
	}
	return run(root, [
		'bun',
		'x',
		'prettier',
		'--write',
		'--ignore-unknown',
		'--log-level',
		'warn',
		...files,
	]);
}

/**
 * `bun setup <name> [options]`, from `scripts/setup.ts`.
 * @returns The exit code
 */
export function main(argv: string[], root: string): number {
	const options = parse(argv);
	if ('error' in options) {
		console.error(`bun setup: ${options.error}\n\n${USAGE}`);
		return 1;
	}
	let made: Made;
	try {
		made = makeApp(root, options);
	} catch (error) {
		console.error(`bun setup: ${(error as Error).message}`);
		return 1;
	}
	console.log(`✓ ${options.title} (${options.name}) is set up from ${TEMPLATE_REPO}.`);
	console.log(
		`  Databases on ${String(options.dbPort)}, ${String(options.dbPort + 1)} and ${String(options.dbPort + 2)}; the API on ${String(options.port)}, the web on ${String(options.port + 1)}, the workbench on ${String(options.port + 3)}; Renovate preset toolkit-${options.renovate}; the toolkit's checkout at ${options.home}.`
	);
	for (const entry of made.packages) console.log(`  ${entry.name} ${entry.range} from npm`);

	if (options.install && !finish(root, made)) {
		console.error(
			'bun setup: the files are set up, and a step after them failed. Run it again by hand: bun install, bun toolkit sync, then bun lint:fix.'
		);
		return 1;
	}
	if (existsSync(path.join(root, '.env'))) {
		console.log(
			'  .env was there before: its database names and ports are the template’s. Compare it with .env.example.'
		);
	}
	console.log(`
  Then: review the change and commit it, and see README.md § Making it the app's.`);
	return 0;
}
