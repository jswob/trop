/**
 * Publishes every package here whose version is not on the registry yet: `bun release:publish`.
 *
 * The release workflow runs it once the "Release packages" PR merges. A new package's first publish
 * is this, run by hand after `npm login`, since a trusted publisher can only be set up for a package
 * that exists. `--tag next` publishes a snapshot; `--dry-run` packs, checks and asks npm what it would
 * publish, and publishes nothing.
 *
 * `bun pm pack` packs each package, because it turns `workspace:` ranges into versions and npm's
 * pack does not. `npm publish <tarball>` publishes it, because npm has trusted publishing and Bun's
 * publish does not. Bun reads a workspace's version from `bun.lock`, and a Bun older than 1.4.3
 * leaves that stale, so every packed manifest is checked before anything is published: its version
 * is the package's, and each range on another package here admits that package's version.
 *
 * Under changesets/action, each publish is written to `CHANGESETS_OUTPUT` as a `git-tag` event, and
 * the action pushes the tag and writes the GitHub release.
 *
 * npm is slow to show a release. Its CDN keeps a package's metadata for five minutes, and a version
 * just published takes a minute or two to appear at all, so a push to master soon after a release
 * would find the version missing and publish it again. The check asks past the CDN, and a publish
 * that fails waits a few minutes for npm to show the version from an earlier run before it counts
 * as a failure.
 */
import { appendFileSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import semver from 'semver';

export type Manifest = {
	name: string;
	version: string;
	private?: boolean;
	dependencies?: Record<string, string>;
	peerDependencies?: Record<string, string>;
	optionalDependencies?: Record<string, string>;
};

/** The fields an install reads. A devDependency never reaches an app, so it is not checked. */
const INSTALLED = ['dependencies', 'peerDependencies', 'optionalDependencies'] as const;

/** What is wrong with a packed manifest, given the version of every package here. */
export function packedProblems(packed: Manifest, versions: ReadonlyMap<string, string>): string[] {
	const problems: string[] = [];
	const own = versions.get(packed.name);
	if (packed.version !== own) {
		problems.push(`packed as ${packed.version}, but the package is ${own ?? 'not here'}`);
	}
	for (const field of INSTALLED) {
		for (const [name, range] of Object.entries(packed[field] ?? {})) {
			if (/^(workspace|catalog|link|file):/.test(range)) {
				problems.push(`${field} ${name} is still "${range}"`);
				continue;
			}
			const version = versions.get(name);
			if (version !== undefined && !semver.satisfies(version, range)) {
				problems.push(`${field} ${name} is "${range}", which leaves out ${version}`);
			}
		}
	}
	return problems;
}

/** The manifests in an order where each comes after those it installs. */
export function publishOrder<T extends Manifest>(manifests: T[]): T[] {
	const byName = new Map(manifests.map(manifest => [manifest.name, manifest]));
	const order: T[] = [];
	const placed = new Set<string>();
	const place = (manifest: T, chain: string[]) => {
		if (placed.has(manifest.name)) return;
		if (chain.includes(manifest.name)) {
			throw new Error(`Dependency cycle: ${[...chain, manifest.name].join(' → ')}`);
		}
		for (const field of INSTALLED) {
			for (const name of Object.keys(manifest[field] ?? {})) {
				const dependency = byName.get(name);
				if (dependency) place(dependency, [...chain, manifest.name]);
			}
		}
		placed.add(manifest.name);
		order.push(manifest);
	};
	for (const manifest of manifests) place(manifest, []);
	return order;
}

type Workspace = Manifest & { dir: string };

function run(cmd: string[], cwd?: string) {
	const result = Bun.spawnSync(cmd, { cwd, stdout: 'pipe', stderr: 'pipe' });
	return {
		ok: result.exitCode === 0,
		stdout: result.stdout.toString(),
		stderr: result.stderr.toString(),
	};
}

function workspaces(root: string): Workspace[] {
	const { workspaces: globs } = JSON.parse(
		readFileSync(path.join(root, 'package.json'), 'utf8')
	) as {
		workspaces: string[];
	};
	return globs.flatMap(glob =>
		[...new Bun.Glob(`${glob}/package.json`).scanSync({ cwd: root })].sort().map(file => ({
			...(JSON.parse(readFileSync(path.join(root, file), 'utf8')) as Manifest),
			dir: path.join(root, path.dirname(file)),
		}))
	);
}

type Get = (url: string, init?: RequestInit) => Promise<Response>;

/**
 * Whether the registry has this version. `?write=true` is answered by the registry itself rather
 * than its CDN, as npm asks before it changes a package.
 */
export async function isPublished(
	registry: string,
	{ name, version }: Manifest,
	get: Get = fetch
): Promise<boolean> {
	const url = `${registry.replace(/\/+$/, '')}/${name.replace('/', '%2f')}?write=true`;
	const response = await get(url, { headers: { accept: 'application/json' } });
	if (response.status === 404) return false;
	if (!response.ok) throw new Error(`${url} answered ${response.status} ${response.statusText}`);
	const { versions = {} } = (await response.json()) as { versions?: Record<string, unknown> };
	return version in versions;
}

/** How long a failed publish waits for the registry to show the version, and how often it asks. */
const SETTLE_MS = 180_000;
const POLL_MS = 15_000;

async function appears(registry: string, workspace: Manifest): Promise<boolean> {
	for (const start = Date.now(); ; await Bun.sleep(POLL_MS)) {
		if (await isPublished(registry, workspace)) return true;
		if (Date.now() - start >= SETTLE_MS) return false;
	}
}

async function main(argv: string[]): Promise<number> {
	const dryRun = argv.includes('--dry-run');
	const tagAt = argv.indexOf('--tag');
	const tag = tagAt === -1 ? 'latest' : argv[tagAt + 1];
	if (!tag || tag.startsWith('--')) {
		console.error('Usage: bun release:publish [--tag <dist-tag>] [--dry-run]');
		return 1;
	}

	const root = path.resolve(import.meta.dirname, '..');
	const all = workspaces(root);
	const versions = new Map(all.map(workspace => [workspace.name, workspace.version]));
	const registry = run(['npm', 'config', 'get', 'registry']).stdout.trim();
	const candidates = all.filter(workspace => !workspace.private);
	const published = await Promise.all(
		candidates.map(workspace => isPublished(registry, workspace))
	);
	const pending = publishOrder(candidates.filter((_workspace, index) => !published[index]));
	const output = process.env.CHANGESETS_OUTPUT;
	if (output) appendFileSync(output, '');
	if (pending.length === 0) {
		console.log('Every package is on the registry already.');
		return 0;
	}

	const packs = mkdtempSync(path.join(tmpdir(), 'release-'));
	try {
		const tarballs: { workspace: Workspace; tarball: string }[] = [];
		let problems = 0;
		for (const workspace of pending) {
			const out = path.join(packs, String(tarballs.length));
			mkdirSync(out);
			const pack = run(['bun', 'pm', 'pack', '--destination', out], workspace.dir);
			const file = readdirSync(out).find(name => name.endsWith('.tgz'));
			if (!pack.ok || !file)
				throw new Error(`bun pm pack failed in ${workspace.dir}:\n${pack.stderr}`);
			const tarball = path.join(out, file);
			tarballs.push({ workspace, tarball });

			const packed = JSON.parse(
				run(['tar', '-xzOf', tarball, 'package/package.json']).stdout
			) as Manifest;
			for (const problem of packedProblems(packed, versions)) {
				console.error(`${workspace.name}: ${problem}`);
				problems++;
			}
		}
		if (problems > 0) {
			console.error(
				'\nNothing was published. A stale bun.lock is the usual cause: `bun install` with Bun 1.4.3 or later.'
			);
			return 1;
		}

		for (const { workspace, tarball } of tarballs) {
			const id = `${workspace.name}@${workspace.version}`;
			const flags = ['--access', 'public', '--tag', tag, ...(dryRun ? ['--dry-run'] : [])];
			// npm speaks to the terminal: a two-factor prompt, or its link to approve in a browser,
			// must reach whoever publishes by hand
			const publish = Bun.spawnSync(['npm', 'publish', tarball, ...flags], {
				stdin: 'inherit',
				stdout: 'inherit',
				stderr: 'inherit',
			});
			if (publish.exitCode !== 0) {
				if (!dryRun) {
					console.error(
						`npm publish ${id} failed. Is it on npm from an earlier run? Asking for 3 minutes.`
					);
					if (await appears(registry, workspace)) {
						// that run wrote its tag event, and a second one would fail on the existing tag
						console.log(`${id} is on npm already, from an earlier run; skipped.`);
						continue;
					}
				}
				console.error(`npm publish ${id} failed (npm's output is above).`);
				return 1;
			}
			console.log(`${dryRun ? 'Would publish' : 'Published'} ${id} (${tag})`);
			if (output && !dryRun) {
				appendFileSync(
					output,
					`${JSON.stringify({ type: 'git-tag', tag: id, packageName: workspace.name })}\n`
				);
			}
		}
		return 0;
	} finally {
		rmSync(packs, { recursive: true, force: true });
	}
}

if (import.meta.main) process.exit(await main(Bun.argv.slice(2)));
