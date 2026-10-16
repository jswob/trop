// The harness has one door. Core knows no adapter and nothing of the app; the app's tasks reach the
// harness through api.ts and testing.ts, the only paths the package exports, and the self-test's
// task, the one to copy, keeps to the same door.
import { readdirSync, readFileSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';

import { describe, expect, test } from 'vitest';

const SRC = import.meta.dirname;

/** Every relative import in the package, as [the importing file, the imported file], from `src/`. */
function imports(): [string, string][] {
	const files = (readdirSync(SRC, { recursive: true }) as string[]).filter(f => f.endsWith('.ts'));
	return files.flatMap(file => {
		const text = readFileSync(join(SRC, file), 'utf8');
		return [...text.matchAll(/from '(\.{1,2}\/[^']+)'/g)].map((m): [string, string] => [
			file,
			relative(SRC, resolve(dirname(join(SRC, file)), m[1] ?? '')),
		]);
	});
}

const inDir = (dir: string) => (file: string) => file.startsWith(`${dir}/`);
const broken = (from: (file: string) => boolean, to: (file: string) => boolean) =>
	imports()
		.filter(([file, target]) => from(file) && to(target))
		.map(([file, target]) => `${file} imports ${target}`);

describe('the module boundary', () => {
	test('core imports only core', () => {
		expect(broken(inDir('core'), target => !inDir('core')(target))).toEqual([]);
	});

	test('the adapter knows no core', () => {
		expect(broken(inDir('adapters'), inDir('core'))).toEqual([]);
	});

	test('the self-test’s task reaches the harness only through api.ts and testing.ts', () => {
		const inside = (file: string) => inDir('core')(file) || inDir('adapters')(file);
		expect(broken(file => file === 'selftest/task.ts', inside)).toEqual([]);
	});
});
