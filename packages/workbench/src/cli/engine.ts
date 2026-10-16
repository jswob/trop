import { readFileSync } from 'node:fs';
import path from 'node:path';

/** This package's folder, wherever it is installed. */
export const ENGINE_ROOT = path.resolve(import.meta.dirname, '../..');

/**
 * What this package is called and which version it is, read off its own manifest.
 *
 * The name is the specifier a file the verbs write imports the package by, so a renamed package
 * scaffolds files that resolve. The version is what a copy of the skill is stamped with.
 */
export function engine(): { name: string; version: string } {
	const manifest = JSON.parse(readFileSync(path.join(ENGINE_ROOT, 'package.json'), 'utf8')) as {
		name: string;
		version: string;
	};

	return { name: manifest.name, version: manifest.version };
}

/** What this package is called where it is installed. */
export function engineName(): string {
	return engine().name;
}
