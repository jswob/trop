import { readFileSync, readdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { parseProposedShape } from '@jswob/design-workbench';
import { describe, expect, it } from 'vitest';

import { PROTOS } from './index';

const PROTO_DIR = dirname(fileURLToPath(import.meta.url));

const tsxFiles = readdirSync(PROTO_DIR).filter(file => file.endsWith('.tsx'));

/**
 * Files in this space that are **not** prototypes: the shared harness a question's candidates are all
 * drawn in. `BUILD.md` requires one author to write the subject, the fixtures and the harness
 * before the fan-out, and a harness carrying JSX has to live in a `.tsx` like anything else. It is
 * held to the same no-API rule and is asserted to stay unregistered — a harness that appeared in
 * the prototype list would be a candidate nobody proposed.
 *
 * Empty while nothing is being argued about, like the space itself. A harness is named here while
 * its candidates are drawn, and is deleted with them when the question settles.
 */
const SHARED: string[] = [];

const protoFiles = tsxFiles.filter(file => !SHARED.includes(file));

/** Reading from disk, not `?raw`: Vitest stubs raw CSS/source imports out to an empty string. */
function sourceOf(file: string): string {
	return readFileSync(resolve(PROTO_DIR, file), 'utf8');
}

describe('proto registry', () => {
	it('registers every prototype file', () => {
		const registered = PROTOS.map(proto => `${proto.id}.tsx`).sort();
		expect(registered).toEqual(protoFiles.sort());
	});

	it('leaves the shared harness out of the prototype list', () => {
		const registered = PROTOS.map(proto => `${proto.id}.tsx`);
		for (const file of SHARED) expect(registered).not.toContain(file);
	});

	it('gives every prototype at least one variant and a unique id', () => {
		expect(new Set(PROTOS.map(proto => proto.id)).size).toBe(PROTOS.length);

		for (const proto of PROTOS) {
			expect(proto.variants.length).toBeGreaterThan(0);
			expect(new Set(proto.variants.map(variant => variant.id)).size).toBe(proto.variants.length);
		}
	});

	it.skipIf(protoFiles.length === 0).each(protoFiles)('%s declares a PROPOSED_SHAPE', file => {
		expect(parseProposedShape(sourceOf(file))).not.toBeNull();
	});

	it.skipIf(tsxFiles.length === 0).each(tsxFiles)('%s imports nothing from the API layer', file => {
		const source = sourceOf(file);

		expect(source).not.toMatch(/from '@\/lib\/api/);
		expect(source).not.toMatch(/from '@\/hooks\/api/);
		expect(source).not.toMatch(/from '@app\/api/);
	});
});
