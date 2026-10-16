/**
 * `toolkit try`: would the app's tests still pass on the toolkit's checkout? Each workspace that
 * depends on the packages runs its tests on what it installs, then on the checkout (linked), and
 * the packages are unlinked whatever happened. Only a test that passed before and fails now is a
 * failure: the app's own broken tests are not the change's.
 */
import { existsSync, mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

import { link, readLinks, unlink } from './link';
import { runCommand, type Run } from './patches';
import { readManifest, workspaceDirs } from './repo';
import { exec, slug, type Exec } from './upstream';

/** Vitest's JSON report, as far as it is read. */
type Report = {
	testResults: {
		name: string;
		status: string;
		assertionResults: { fullName: string; status: string }[];
	}[];
};

export type Tried = {
	/** The workspace, from the repo's root (`.` for the root). */
	workspace: string;
	/** Tests that passed before and fail on the checkout. */
	broken: string[];
	/** Tests that failed before and pass on the checkout. */
	fixed: string[];
	/** Tests that failed both times. */
	failing: string[];
};

/**
 * The workspaces whose tests reach the packages: each that depends on one and has a `test` script.
 * The root only when no workspace does, since its tests are usually every workspace's.
 */
export function dependents(repo: string, names: readonly string[]): string[] {
	const reaches = (dir: string) => {
		const manifest = readManifest(dir);
		const deps = { ...manifest?.dependencies, ...manifest?.devDependencies };
		return manifest?.scripts?.test !== undefined && names.some(name => name in deps);
	};
	const found = workspaceDirs(repo).filter(reaches);
	if (found.length === 0 && reaches(repo)) return [repo];
	return found;
}

/**
 * The failing tests in a Vitest JSON report, each `<file> > <full name>`; a file that failed with
 * no failing test (it did not load) is the file alone.
 */
export function failures(report: Report, root: string): string[] {
	return report.testResults.flatMap(file => {
		const name = path.relative(root, file.name);
		const failed = file.assertionResults
			.filter(test => test.status === 'failed')
			.map(test => `${name} > ${test.fullName}`);
		return failed.length === 0 && file.status === 'failed' ? [name] : failed;
	});
}

/** Run a workspace's tests with the JSON reporter, and read its failures. `null`: no report. */
function runTests(run: Exec, repo: string, dir: string, out: string): string[] | null {
	const script = readManifest(dir)?.scripts?.test ?? '';
	rmSync(out, { force: true });
	run(
		'bun',
		[
			'run',
			'test',
			...(script.includes('--run') ? [] : ['--run']),
			'--reporter=json',
			`--outputFile=${out}`,
		],
		dir
	);
	if (!existsSync(out)) return null;
	return failures(JSON.parse(readFileSync(out, 'utf8')) as Report, repo);
}

/**
 * Run the dependents' tests before and after linking the packages to the toolkit's checkout.
 * @param repo - The app's root
 * @param home - The toolkit's checkout
 * @param names - The packages to try
 * @param run - Runs the tests
 * @param install - Runs `bun install` when unlinking
 */
export function tryPackages(
	repo: string,
	home: string,
	names: readonly string[],
	run: Exec = exec,
	install: Run = runCommand
): Tried[] {
	if (names.length === 0) throw new Error('which packages? toolkit try <package...>');
	const already = names.filter(name => readLinks(repo)[name] !== undefined);
	if (already.length > 0) {
		throw new Error(
			`${already.join(', ')} ${already.length === 1 ? 'is' : 'are'} linked already, so the tests ` +
				`cannot run on what the app installs: bun toolkit unlink ${already.join(' ')}`
		);
	}

	const workspaces = dependents(repo, names);
	if (workspaces.length === 0) return [];
	const reports = mkdtempSync(path.join(tmpdir(), 'toolkit-try-'));
	const out = (dir: string, when: string) =>
		path.join(reports, `${slug(path.relative(repo, dir) || 'root')}-${when}.json`);

	try {
		const before = workspaces.map(dir => {
			const failed = runTests(run, repo, dir, out(dir, 'before'));
			if (failed === null) {
				throw new Error(
					`the tests in ${path.relative(repo, dir) || '.'} gave no report before linking: run them there`
				);
			}
			return failed;
		});

		link(repo, home, names);
		try {
			return workspaces.map((dir, i) => {
				const was = new Set(before[i]);
				const now = runTests(run, repo, dir, out(dir, 'after')) ?? [
					`${path.relative(repo, dir) || '.'}: the tests gave no report on the checkout`,
				];
				const is = new Set(now);
				return {
					workspace: path.relative(repo, dir) || '.',
					broken: now.filter(test => !was.has(test)),
					fixed: [...was].filter(test => !is.has(test)),
					failing: now.filter(test => was.has(test)),
				};
			});
		} finally {
			unlink(repo, names, install);
		}
	} finally {
		rmSync(reports, { recursive: true, force: true });
	}
}
