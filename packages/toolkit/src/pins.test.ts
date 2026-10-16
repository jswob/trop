import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import { pinProblems } from './pins';

let repo: string;

function put(file: string, value: unknown) {
	const full = path.join(repo, file);
	mkdirSync(path.dirname(full), { recursive: true });
	writeFileSync(full, `${JSON.stringify(value, null, '\t')}\n`, 'utf8');
}

beforeEach(() => {
	repo = mkdtempSync(path.join(tmpdir(), 'toolkit-pins-'));
	put('package.json', { workspaces: ['apps/*'], packageManager: 'bun@1.4.3' });
	put('apps/web/package.json', { name: 'web', dependencies: { '@acme/kit': '^1.0.0' } });
});

afterEach(() => {
	rmSync(repo, { recursive: true, force: true });
});

describe('pinProblems', () => {
	it('is empty when Bun is pinned and every dependency is a version', () => {
		expect(pinProblems(repo)).toEqual([]);
	});

	it('takes a pin with its hash', () => {
		put('package.json', { packageManager: 'bun@1.4.3+sha512.abc' });

		expect(pinProblems(repo)).toEqual([]);
	});

	it.for([
		[undefined, 'not set'],
		['bun@1', '"bun@1"'],
		['bun@^1.4.3', '"bun@^1.4.3"'],
		['pnpm@10.0.0', '"pnpm@10.0.0"'],
	] as const)('names a packageManager of %s', ([pinned, said]) => {
		put('package.json', { workspaces: ['apps/*'], packageManager: pinned });

		expect(pinProblems(repo)).toEqual([
			`package.json: packageManager is ${said}; pin Bun to one version, "bun@<x.y.z>"`,
		]);
	});

	it('names a dependency on a folder, in the root or a workspace', () => {
		put('package.json', {
			workspaces: ['apps/*'],
			packageManager: 'bun@1.4.3',
			devDependencies: { '@acme/tools': 'file:../tools' },
		});
		put('apps/web/package.json', {
			name: 'web',
			dependencies: { '@acme/kit': 'link:@acme/kit', '@acme/ui': 'workspace:*' },
		});

		expect(pinProblems(repo)).toEqual([
			'package.json: @acme/tools is "file:../tools", a folder no other clone has: depend on a ' +
				'version (bun toolkit link runs the app on a checkout)',
			'apps/web/package.json: @acme/kit is "link:@acme/kit", a folder no other clone has: ' +
				'depend on a version (bun toolkit link runs the app on a checkout)',
		]);
	});
});
