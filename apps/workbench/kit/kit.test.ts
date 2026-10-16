import { readFileSync, readdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { describe, expect, it } from 'vitest';

const KIT_DIR = dirname(fileURLToPath(import.meta.url));

/** Reading from disk, not `?raw`: Vitest stubs raw source imports out to an empty string. */
function sourceOf(file: string): string {
	return readFileSync(resolve(KIT_DIR, file), 'utf8');
}

/**
 * The code, without the prose. A doc comment naming `views/place-frame` is how these files explain
 * where they came from; only a reference the compiler sees is a dependency.
 */
function codeOf(file: string): string {
	return sourceOf(file)
		.replace(/\/\*[\s\S]*?\*\//g, '')
		.replace(/^[ \t]*\/\/.*$/gm, '');
}

const kitFiles = readdirSync(KIT_DIR).filter(
	file => (file.endsWith('.ts') || file.endsWith('.tsx')) && !file.includes('.test.')
);

/**
 * **The kit knows nothing about this app.**
 *
 * It is the one rule that makes a kit a kit rather than a folder. A component that reached back
 * for a place's fixture would draw one screen correctly and every other caller's screen wrong, and
 * nothing on any screenshot would say so. A kit component that imported a fixture, a subject, a
 * place or its own documentation would be a component with content in it.
 */
describe('the kit', () => {
	/* Only that the directory was read: the barrel is here even while every component is settled. */
	it('has files to check', () => {
		expect(kitFiles.length).toBeGreaterThan(0);
	});

	it.each(kitFiles)('%s imports nothing from views, subjects, fixtures or kit-docs', file => {
		expect(codeOf(file)).not.toMatch(/from '\.\.\/(views|subjects|fixtures|kit-docs)/);
	});

	it.each(kitFiles)('%s imports nothing from the API layer', file => {
		const code = codeOf(file);
		expect(code).not.toMatch(/from '@\/lib\/api/);
		expect(code).not.toMatch(/from '@\/hooks\/api/);
		expect(code).not.toMatch(/from '@app\/api/);
	});

	/*
	 * A settled kit module is reached as `@app/ui/kit/<Name>`, and that is still the kit. The web's
	 * frame and routes, which fill the views with the app's records, are not.
	 */
	it.each(kitFiles)('%s imports nothing from the web’s frame or routes', file => {
		expect(codeOf(file)).not.toMatch(/from '@\/(frame|routes)\//);
	});
});
