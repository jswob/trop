import { spawnSync } from 'node:child_process';
import {
	existsSync,
	mkdirSync,
	readFileSync,
	readdirSync,
	renameSync,
	statSync,
	writeFileSync,
} from 'node:fs';
import path from 'node:path';

import { nameOf } from '../sources';
import type { Argv } from './args';
import { codeOf, exportedSymbols } from './callers';
import { host } from './host';
import { addKitExport } from './promote';

/** The two workbench spaces whose settled modules move into the app. */
export type AppSpace = 'kit' | 'views';

/** One import or re-export statement: where it points and the names it takes. */
export type ImportLine = { specifier: string; names: string[] };

/** Where one import of a moving module points in the app, or why it cannot point anywhere yet. */
export type Verdict = { specifier: string; to: string } | { specifier: string; blocked: string };

/** What the app already holds, so a moving module's imports can be judged against it. */
export type AppState = {
	/** The specifier the app imports its settled code by: `@/ui`. */
	alias: string;
	/** Whether `<space>/<name>` exists where settled code lives. */
	inApp: (space: AppSpace, name: string) => boolean;
	/** Every name the settled kit's `index.ts` exports, through the barrels it re-exports whole. */
	appKitExports: Set<string>;
};

const STATEMENT =
	/(?:^|\n)[ \t]*(?:import|export)\s+(?:type\s+)?(?:([\w$]+)\s*,?\s*)?(?:\{([^}]*)\}\s*|\*\s+as\s+[\w$]+\s*)?(?:from\s+)?'([^']+)'/g;

/** Every import and re-export in a source, in order, with the names each takes. Code only. */
export function importsOf(source: string): ImportLine[] {
	return [...codeOf(source).matchAll(STATEMENT)].map(match => ({
		specifier: match[3] ?? '',
		names: (match[2] ?? '')
			.split(',')
			.map(name =>
				name
					.trim()
					.replace(/^type\s+/, '')
					.split(/\s+as\s+/)[0]
					?.trim()
			)
			.filter((name): name is string => name !== undefined && name !== ''),
	}));
}

/**
 * Every name a barrel exports: its own `export { … } from` lines, and every name of a sibling it
 * re-exports whole (`export * from './core'`), followed as deep as they go and each once.
 * @param sibling - A sibling module's source by its name (`core`), or `undefined` where none is
 */
export function barrelNames(
	source: string,
	sibling: (name: string) => string | undefined,
	seen: Set<string> = new Set()
): Set<string> {
	const names = new Set(importsOf(source).flatMap(line => line.names));
	for (const [, name] of codeOf(source).matchAll(/^[ \t]*export\s+\*\s+from\s+'\.\/([^']+)'/gm)) {
		if (name === undefined || seen.has(name)) continue;
		seen.add(name);
		const next = sibling(name);
		if (next !== undefined) for (const one of barrelNames(next, sibling, seen)) names.add(one);
	}
	return names;
}

/**
 * How each import of a module moving out of the design root's `<space>/` fares once it sits in the
 * settled `<space>/`.
 *
 * Packages and the app's own aliases resolve the same from both places. A sibling resolves only if
 * it has already moved. The kit, reached from a view, resolves through the settled alias only for
 * the names that have moved. **Everything else relative is workbench-only** — fixtures, places,
 * kit-docs — and that is a split made by hand: the verb does not guess which constant is a
 * fixture.
 */
export function judgeImports(lines: ImportLine[], space: AppSpace, app: AppState): Verdict[] {
	return lines.map(({ specifier, names }) => {
		if (!specifier.startsWith('.')) return { specifier, to: specifier };

		const sibling = /^\.\/([^/]+)$/.exec(specifier);
		if (sibling?.[1] !== undefined) {
			return app.inApp(space, sibling[1])
				? { specifier, to: specifier }
				: { specifier, blocked: `${space}/${sibling[1]} has not moved` };
		}

		const deep = /^\.\.\/(kit|views)\/([^/]+)$/.exec(specifier);
		if (deep?.[1] !== undefined && deep[2] !== undefined) {
			const [, other, name] = deep as unknown as [string, AppSpace, string];
			return app.inApp(other, name)
				? { specifier, to: `${app.alias}/${other}/${name}` }
				: { specifier, blocked: `${other}/${name} has not moved` };
		}

		if (specifier === '../kit') {
			const missing = names.filter(name => !app.appKitExports.has(name));
			return missing.length === 0
				? { specifier, to: `${app.alias}/kit` }
				: { specifier, blocked: `kit has not moved: ${missing.join(', ')}` };
		}

		return { specifier, blocked: 'workbench-only' };
	});
}

/** Replaces each `from '<old>'` / `import '<old>'` specifier with its new one. */
function rewriteSpecifiers(source: string, to: Map<string, string>): string {
	return source.replace(
		/(\bfrom\s+|^[ \t]*import\s+)'([^']+)'/gm,
		(whole, lead: string, specifier: string) => {
			const next = to.get(specifier);
			return next === undefined ? whole : `${lead}'${next}'`;
		}
	);
}

/** Every blocked verdict as `<specifier>: <reason>`. */
function reasons(verdicts: Verdict[]): string[] {
	return verdicts.flatMap(verdict =>
		'blocked' in verdict ? [`${verdict.specifier}: ${verdict.blocked}`] : []
	);
}

/** The moved imports as a specifier map, for `rewriteSpecifiers`. */
function targets(verdicts: Verdict[]): Map<string, string> {
	return new Map(
		verdicts.flatMap(verdict => ('to' in verdict ? [[verdict.specifier, verdict.to]] : []))
	);
}

export type AppMovePlan = {
	/** Imports that keep the module in the workbench. Empty means it can move. */
	blocked: string[];
	/** The module's source as it will read in the app. */
	source: string;
	/**
	 * Its collocated test: moved beside it when every import resolves from the app; otherwise left in
	 * the workbench — a fixture-driven test of an app module, reaching it through the alias.
	 */
	test: { moves: true; source: string } | { moves: false; because: string[] } | null;
};

/** What moving `<space>/<name>` out of the design root into the settled space would do. Pure. */
export function planAppMove(
	module: { space: AppSpace; name: string },
	source: string,
	testSource: string | null,
	app: AppState
): AppMovePlan {
	const verdicts = judgeImports(importsOf(source), module.space, app);

	let test: AppMovePlan['test'] = null;
	if (testSource !== null) {
		const withModule: AppState = {
			...app,
			inApp: (space, name) =>
				(space === module.space && name === module.name) || app.inApp(space, name),
		};
		const testVerdicts = judgeImports(importsOf(testSource), module.space, withModule);
		const because = reasons(testVerdicts);
		test =
			because.length === 0
				? { moves: true, source: rewriteSpecifiers(testSource, targets(testVerdicts)) }
				: { moves: false, because };
	}

	return {
		blocked: reasons(verdicts),
		source: rewriteSpecifiers(source, targets(verdicts)),
		test,
	};
}

/**
 * A workbench file, re-pointed at a module that moved: every relative import resolving to
 * `<space>/<name>` becomes `<alias>/<space>/<name>`, and every place the file names
 * `'<space>/<name><ext>'` as data — `kit-docs/index.ts` lists each entry's own files so the caller
 * count can leave them out — names the app path instead, the way `sources` does.
 */
export function rewriteCaller(
	source: string,
	file: string,
	moving: {
		space: AppSpace;
		name: string;
		ext: string;
		/** The specifier settled code is imported by: `@/ui`. */
		alias: string;
		/** What the settled root is called from the repo's root: `packages/ui/src`. */
		settled: string;
	}
): string {
	const home = `${moving.space}/${moving.name}`;
	const dir = path.posix.dirname(file);

	const imports = source.replace(
		/(\bfrom\s+|^[ \t]*import\s+)'(\.{1,2}\/[^']+)'/gm,
		(whole, lead: string, specifier: string) =>
			path.posix.normalize(path.posix.join(dir, specifier)) === home
				? `${lead}'${moving.alias}/${home}'`
				: whole
	);

	return imports.split(`'${home}${moving.ext}'`).join(`'${moving.settled}/${home}${moving.ext}'`);
}

/**
 * Directories a caller rewrite never touches: dependencies, and the workbench's own tooling, whose
 * tests spell module paths inside string fixtures that are data, not imports.
 */
const NOT_CALLERS = new Set(['node_modules', 'cli', 'probe']);

/** Every `.ts`/`.tsx` in the workbench that could import a module, relative to the design root. */
function workbenchFiles(dir = ''): string[] {
	return readdirSync(path.join(host().root, dir)).flatMap(entry => {
		if (dir === '' && NOT_CALLERS.has(entry)) return [];
		if (entry === 'node_modules') return [];
		const relative = path.posix.join(dir, entry);
		if (statSync(path.join(host().root, relative)).isDirectory()) return workbenchFiles(relative);
		return /\.tsx?$/.test(entry) ? [relative] : [];
	});
}

/** The first of `<base>.ts` / `<base>.tsx` that exists. */
function withExt(base: string): string | undefined {
	return ['.tsx', '.ts'].map(ext => `${base}${ext}`).find(existsSync);
}

/** The head of the settled kit's index, written with the first module that moves. */
function appKitHeader(app: string): string {
	return `/**
 * The ${app} kit, as far as it has moved into the app. A module lands here through
 * \`bun workbench promote kit/<Name> --to app\`; the design root's \`kit/index.ts\` re-exports it
 * from here, so there is one copy.
 */
`;
}

/**
 * Moves a file, through git where git knows it so the history follows. A module written in the same
 * slice that moves it — a half split off by hand — is not tracked yet, and is renamed.
 */
function gitMv(from: string, to: string): void {
	const tracked = spawnSync('git', ['ls-files', '--error-unmatch', from], { encoding: 'utf8' });
	if (tracked.status !== 0) {
		renameSync(from, to);
		return;
	}
	const result = spawnSync('git', ['mv', from, to], { encoding: 'utf8' });
	if (result.status !== 0) throw new Error(`git mv ${from} ${to}: ${result.stderr}`);
}

/**
 * `promote <kit|views>/<name> --to app` — a settled module moves out of the design root into the
 * app's settled root, where the app and the workbench both reach it.
 *
 * It refuses rather than guesses: any import that would not resolve from the app is printed, and
 * nothing moves until the module has been split by hand. A test that needs the workbench's
 * fixtures stays behind and reaches the moved module through the settled alias.
 */
export function promoteToApp(argv: Argv): number {
	const target = argv.positional[0] ?? '';
	const match = /^(kit|views)\/([\w-]+)$/.exec(target);
	if (match?.[1] === undefined || match[2] === undefined)
		throw new Error('name the module: workbench promote kit/<Name>|views/<name> --to app');

	const space = match[1] as AppSpace;
	const name = match[2];
	const { root, settled } = host();
	const settledName = nameOf(settled.root, host());
	const from = withExt(path.join(root, space, name));
	if (from === undefined) throw new Error(`no ${space}/${name}.ts(x) in the workbench`);
	const ext = path.extname(from);
	const testFrom = withExt(path.join(root, space, `${name}.test`));

	const appDir = path.join(settled.root, space);
	const appIndex = path.join(settled.root, 'kit/index.ts');
	const indexSource = existsSync(appIndex)
		? readFileSync(appIndex, 'utf8')
		: appKitHeader(host().name);
	const app: AppState = {
		alias: settled.alias,
		inApp: (other, otherName) => withExt(path.join(settled.root, other, otherName)) !== undefined,
		appKitExports: barrelNames(indexSource, sibling => {
			const file = withExt(path.join(settled.root, 'kit', sibling));
			return file === undefined ? undefined : readFileSync(file, 'utf8');
		}),
	};

	const source = readFileSync(from, 'utf8');
	const plan = planAppMove(
		{ space, name },
		source,
		testFrom === undefined ? null : readFileSync(testFrom, 'utf8'),
		app
	);

	const out = (line: string): void => void process.stdout.write(`${line}\n`);
	out(`promoting ${space}/${name}${ext} → ${settledName}/${space}/\n`);

	if (plan.blocked.length > 0) {
		out(`  blocked — these imports do not resolve from the app:`);
		for (const reason of plan.blocked) out(`    ${reason}`);
		out(`\n  Split the module by hand: fixture constants stay in the design root's fixtures/, and`);
		out(`  a sibling moves first. Nothing was moved.`);
		return 1;
	}

	const moving = { space, name, ext, alias: settled.alias, settled: settledName };
	const callers = workbenchFiles()
		.filter(file => file !== `${space}/${name}${ext}`)
		.filter(file => testFrom === undefined || file !== path.relative(root, testFrom))
		.map(file => {
			const before = readFileSync(path.join(root, file), 'utf8');
			return { file, before, after: rewriteCaller(before, file, moving) };
		})
		.filter(caller => caller.after !== caller.before);

	const testName = testFrom === undefined ? null : path.basename(testFrom);
	if (plan.test === null) out(`  test      none`);
	else if (plan.test.moves) out(`  test      ${testName} moves beside it`);
	else {
		out(`  test      ${testName} stays in the workbench:`);
		for (const reason of plan.test.because) out(`              ${reason}`);
	}
	out(`  callers   ${callers.length === 0 ? 'none' : callers.map(c => c.file).join(', ')}`);

	const { values, types } = exportedSymbols(source);
	if (space === 'kit')
		out(`  exports   ${settledName}/kit/index.ts ← ${[...values, ...types].join(', ')}`);

	if (argv.flags['dry'] !== undefined) return 0;

	mkdirSync(appDir, { recursive: true });
	const to = path.join(appDir, `${name}${ext}`);
	gitMv(from, to);
	writeFileSync(to, plan.source, 'utf8');

	if (testFrom !== undefined && plan.test?.moves === true) {
		const testTo = path.join(appDir, path.basename(testFrom));
		gitMv(testFrom, testTo);
		writeFileSync(testTo, plan.test.source, 'utf8');
	} else if (testFrom !== undefined) {
		const before = readFileSync(testFrom, 'utf8');
		writeFileSync(testFrom, rewriteCaller(before, path.relative(root, testFrom), moving));
	}

	for (const caller of callers) writeFileSync(path.join(root, caller.file), caller.after);

	if (space === 'kit') writeFileSync(appIndex, addKitExport(indexSource, name, values, types));

	out(`\nmoved. Then:`);
	out(`  the linter's fix      — an aliased import sorts above the relative ones it replaced`);
	out(`  bun workbench check   ·   the project's own lint and tests`);
	return 0;
}
