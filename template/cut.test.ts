import path from 'node:path';

import { describe, expect, it } from 'vitest';

import { renameText } from './cut';
import manifest from './manifest';

describe('the cut', () => {
	it('gives the packages the source takes by version back their workspace ranges', () => {
		const source = [
			'{',
			'\t"name": "@meetings-scheduler/web",',
			'\t"devDependencies": {',
			'\t\t"@jswob/style-guide": "^0.1.1",',
			'\t\t"@jswob/design-workbench": "0.2.0-next-20261015",',
			'\t\t"@playwright/test": "1.64.1"',
			'\t}',
			'}',
		].join('\n');

		const cut = renameText('apps/web/package.json', source, manifest.text);

		expect(cut).toContain('"name": "@app/web"');
		expect(cut).toContain('"@jswob/style-guide": "workspace:*"');
		expect(cut).toContain('"@jswob/design-workbench": "workspace:*"');
		expect(cut).toContain('"@playwright/test": "1.64.1"');
	});

	it('leaves a package range outside a package.json alone', () => {
		const text = 'bun add "@jswob/toolkit": "^0.1.1"';

		expect(renameText('docs/notes.md', text, manifest.text)).toBe(text);
	});

	it('never copies nor deletes the lockfile, and never copies the source’s patches', () => {
		const glob = (pattern: string, file: string) => path.matchesGlob(file, pattern);

		expect(manifest.keep.some(pattern => glob(pattern, 'bun.lock'))).toBe(true);
		expect(
			manifest.exclude.some(pattern => glob(pattern, 'patches/@jswob%2Ftoolkit@0.1.1.patch'))
		).toBe(true);
	});
});
