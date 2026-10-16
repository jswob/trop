import { readFileSync, readdirSync, statSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { describe, expect, it } from 'vitest';

const FRAME_DIR = dirname(fileURLToPath(import.meta.url));
const SRC_DIR = resolve(FRAME_DIR, '..');

/** Every `.ts`/`.tsx` file under `dir`, recursively, as absolute paths. */
function sourcesUnder(dir: string): string[] {
	let entries: string[];
	try {
		entries = readdirSync(dir);
	} catch {
		return [];
	}
	return entries.flatMap(entry => {
		const path = join(dir, entry);
		if (statSync(path).isDirectory()) return sourcesUnder(path);
		return /\.tsx?$/.test(entry) ? [path] : [];
	});
}

/** The code without the prose — a doc comment naming a path is not a dependency. */
function codeOf(path: string): string {
	return readFileSync(path, 'utf8')
		.replace(/\/\*[\s\S]*?\*\//g, '')
		.replace(/^[ \t]*\/\/.*$/gm, '');
}

/** Paths relative to `src/`, for readable test names. */
function short(path: string): string {
	return relative(SRC_DIR, path);
}

const APP_FILES = sourcesUnder(SRC_DIR)
	.filter(path => !path.endsWith('routeTree.gen.ts'))
	.filter(path => path !== fileURLToPath(import.meta.url))
	.map(short);

/**
 * The walls around the app. It never reaches into the workbench, whose fixtures would otherwise
 * ship, nor into the specimens the workbench keeps beside the UI package's components. The walls
 * around the kit and the views are the UI package's own (`@app/ui`).
 */
describe('src boundaries', () => {
	it('has app files to check', () => {
		expect(APP_FILES.length).toBeGreaterThan(20);
	});

	it.each(APP_FILES)('%s imports nothing from the design workbench', file => {
		expect(codeOf(resolve(SRC_DIR, file))).not.toMatch(/from '[^']*\bworkbench(-app)?[/']/);
	});

	it.each(APP_FILES)('%s imports no specimen, the workbench’s page for a component', file => {
		expect(codeOf(resolve(SRC_DIR, file))).not.toMatch(/from '[^']*\.specimen'/);
	});
});
