import { readFileSync, readdirSync, statSync } from 'node:fs';
import { dirname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

import { describe, expect, it } from 'vitest';

const SRC_DIR = dirname(fileURLToPath(import.meta.url));
const MANIFEST = JSON.parse(readFileSync(join(SRC_DIR, '../package.json'), 'utf8')) as {
	name: string;
	dependencies: Record<string, string>;
	peerDependencies: Record<string, string>;
	devDependencies: Record<string, string>;
};

/** Every `.ts`/`.tsx` file under `dir`, recursively, relative to `src/`. */
function sourcesUnder(dir: string): string[] {
	return readdirSync(dir).flatMap(entry => {
		const path = join(dir, entry);
		if (statSync(path).isDirectory()) return sourcesUnder(path);
		return /\.tsx?$/.test(entry) ? [relative(SRC_DIR, path)] : [];
	});
}

/** The specifiers a file imports, mocks or loads, with its prose left out. */
function specifiersOf(file: string): string[] {
	const code = readFileSync(join(SRC_DIR, file), 'utf8')
		.replace(/\/\*[\s\S]*?\*\//g, '')
		.replace(/^[ \t]*\/\/.*$/gm, '');
	const statements =
		/(?:^\s*(?:import|export)\b[^;'"]*?\bfrom\s*|^\s*import\s*|\bimport\(\s*|\bvi\.mock\(\s*)'([^']+)'/gm;
	return [...code.matchAll(statements)].map(match => match[1]!);
}

/** The package a bare specifier names: `@scope/name` or `name`. */
function packageOf(specifier: string): string {
	const parts = specifier.split('/');
	return specifier.startsWith('@') ? parts.slice(0, 2).join('/') : parts[0]!;
}

const isTest = (file: string) => /\.test\.tsx?$/.test(file) || file.startsWith('test/');
/** A component's page on the workbench, beside it: `Button.specimen.tsx`. */
const isSpecimen = (file: string) => file.includes('.specimen.');
const FILES = sourcesUnder(SRC_DIR);
const SHIPPED = FILES.filter(file => !isTest(file) && !isSpecimen(file));
const RUNTIME = new Set([
	MANIFEST.name,
	...Object.keys(MANIFEST.dependencies),
	...Object.keys(MANIFEST.peerDependencies),
]);
const TESTING = new Set([...RUNTIME, ...Object.keys(MANIFEST.devDependencies)]);

/**
 * The walls around the UI. The package knows no app: it imports no app code, no API and no
 * workbench, only itself and the packages its manifest names, so the web and the workbench can both
 * draw it. Inside it the kit knows nothing of the views, so a kit component stays a kit component.
 *
 * A specimen is the one file here the web never loads: a component's page on the workbench, which
 * draws it with the engine the manifest names as a dev dependency. Nothing shipped imports one.
 */
describe('@app/ui boundaries', () => {
	it('has files to check', () => {
		expect(SHIPPED.length).toBeGreaterThan(40);
	});

	it.each(FILES)('%s imports only itself and the packages its manifest names', file => {
		const allowed = isTest(file) || isSpecimen(file) ? TESTING : RUNTIME;
		const outside = specifiersOf(file)
			.filter(specifier => !specifier.startsWith('.'))
			.filter(specifier => !(isTest(file) && specifier.startsWith('node:')))
			.filter(specifier => !allowed.has(packageOf(specifier)));
		expect(outside).toEqual([]);
	});

	it.each(SHIPPED)('%s imports no specimen', file => {
		expect(specifiersOf(file).filter(specifier => specifier.includes('.specimen'))).toEqual([]);
	});

	it.each(FILES.filter(file => file.startsWith('kit/') && !isTest(file)))(
		'%s imports nothing from the views',
		file => {
			expect(specifiersOf(file).filter(specifier => /(^|\/)views(\/|$)/.test(specifier))).toEqual(
				[]
			);
		}
	);

	it.each(SHIPPED.filter(file => file.startsWith('lib/')))(
		'%s imports nothing from the kit or the views',
		file => {
			expect(
				specifiersOf(file).filter(specifier => /(^|\/)(kit|views)(\/|$)/.test(specifier))
			).toEqual([]);
		}
	);
});
