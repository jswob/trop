/**
 * What keeps every install the one CI tested: Bun pinned to one version in `packageManager`, which
 * CI installs (`bun-version-file`) and which decides how `bun.lock` is written, and no dependency on
 * a folder (`link:`, `file:`), which no other clone has. `toolkit link` tries a checkout instead,
 * and `check` names it while it is linked.
 */
import { realpathSync } from 'node:fs';
import path from 'node:path';

import { readManifest, rootManifest, workspaceDirs } from './repo';

const FIELDS = ['dependencies', 'devDependencies', 'optionalDependencies'] as const;

/**
 * Why an install here may not be the one CI tested. Empty when it is.
 * @param repo - The repo's root
 */
export function pinProblems(repo: string): string[] {
	const root = rootManifest(repo);
	const pinned = root.packageManager;
	const bun =
		pinned !== undefined && /^bun@\d+\.\d+\.\d+(\+\S+)?$/.test(pinned)
			? []
			: [
					`package.json: packageManager is ${pinned === undefined ? 'not set' : `"${pinned}"`}; ` +
						'pin Bun to one version, "bun@<x.y.z>"',
				];

	const real = realpathSync(repo);
	const folders = [repo, ...workspaceDirs(repo, root)].flatMap(dir => {
		const manifest = readManifest(dir);
		if (manifest === null) return [];
		const where = path.join(path.relative(real, realpathSync(dir)), 'package.json');
		return FIELDS.flatMap(field =>
			Object.entries(manifest[field] ?? {})
				.filter(([, spec]) => /^(link|file):/.test(spec))
				.map(
					([name, spec]) =>
						`${where}: ${name} is "${spec}", a folder no other clone has: depend on a version ` +
						'(bun toolkit link runs the app on a checkout)'
				)
		);
	});

	return [...bun, ...folders];
}
