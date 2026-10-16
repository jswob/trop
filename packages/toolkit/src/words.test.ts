import { spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import { appWords, hitsIn, readAgnostic, wordPattern } from './words';

describe('wordPattern', () => {
	const found = (text: string) => [...text.matchAll(wordPattern(['ledger', 'order']))].length;

	it('finds a word whatever its case, and where it starts a longer one', () => {
		expect(found('Ledger')).toBe(1);
		expect(found('the ledgers')).toBe(1);
		expect(found('@acme/ledger-api')).toBe(1);
		expect(found('db_ledger_test')).toBe(1);
	});

	it('never finds a word inside another', () => {
		expect(found('a field border')).toBe(0);
		expect(found('reorder: true')).toBe(0);
		expect(found('subledger')).toBe(0);
	});

	it('reads a word as plain text, not as a pattern', () => {
		expect([...'a.b axb'.matchAll(wordPattern(['a.b']))]).toHaveLength(1);
	});
});

describe('hitsIn', () => {
	it('names each hit by its line and the word in lower case', () => {
		expect(
			hitsIn('notes.md', 'one\nthe Ledger\ntwo ledgers, an order', wordPattern(['ledger', 'order']))
		).toEqual([
			{ file: 'notes.md', line: 2, word: 'ledger' },
			{ file: 'notes.md', line: 3, word: 'ledger' },
			{ file: 'notes.md', line: 3, word: 'order' },
		]);
	});
});

describe('appWords', () => {
	let repo: string;

	function put(file: string, text: string) {
		const full = path.join(repo, file);
		mkdirSync(path.dirname(full), { recursive: true });
		writeFileSync(full, text, 'utf8');
	}

	beforeEach(() => {
		repo = mkdtempSync(path.join(tmpdir(), 'toolkit-words-'));
		spawnSync('git', ['init', '-q'], { cwd: repo });
		put(
			'package.json',
			JSON.stringify({ toolkit: { agnostic: { paths: ['packages/kit'], words: ['ledger'] } } })
		);
		put('.gitignore', 'packages/kit/dist/\n');
	});

	afterEach(() => {
		rmSync(repo, { recursive: true, force: true });
	});

	it('reads the paths and the words from the root manifest', () => {
		expect(readAgnostic(repo)).toEqual({ paths: ['packages/kit'], words: ['ledger'] });
	});

	it('finds the words in the named paths only, tracked or not', () => {
		put('packages/kit/src/index.ts', "export const name = 'ledger';\n");
		spawnSync('git', ['add', '.'], { cwd: repo });
		put('packages/kit/README.md', '# Kit\n\nBuilt for the Ledger.\n');
		put('apps/ledger/index.ts', "export const app = 'ledger';\n");

		expect(appWords(repo)).toEqual([
			{ file: 'packages/kit/README.md', line: 3, word: 'ledger' },
			{ file: 'packages/kit/src/index.ts', line: 1, word: 'ledger' },
		]);
	});

	it('skips what git ignores and files that are not text', () => {
		put('packages/kit/dist/index.js', "'ledger'\n");
		put('packages/kit/font.woff2', 'ledger\0\0');

		expect(appWords(repo)).toEqual([]);
	});

	it('finds nothing where the manifest names no words', () => {
		put('package.json', JSON.stringify({}));
		put('packages/kit/index.ts', "'ledger'\n");

		expect(readAgnostic(repo)).toBeNull();
		expect(appWords(repo)).toEqual([]);
	});
});
