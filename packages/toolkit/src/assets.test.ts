import { describe, expect, it } from 'vitest';

import { EMPTY_LOCK, type LockEntry, lockText, parseLock, staleness, syncPlan } from './assets';

const PKG = { name: '@acme/kit', version: '1.2.0' };

const source = { 'skills/kit/SKILL.md': '# Kit\n', 'skills/kit/BUILD.md': '# Build\n' };
const entry: LockEntry = {
	version: PKG.version,
	files: ['skills/kit/BUILD.md', 'skills/kit/SKILL.md'],
};

describe('lock', () => {
	it('reads back what it wrote', () => {
		const lock = { version: 1 as const, packages: { [PKG.name]: entry } };

		expect(parseLock(lockText(lock))).toEqual(lock);
	});

	it('is empty when there is no file, and throws on one that is not a lock', () => {
		expect(parseLock(undefined)).toEqual(EMPTY_LOCK);
		expect(() => parseLock('{"version":2}')).toThrow(/not a toolkit lock/);
	});
});

describe('syncPlan', () => {
	it('writes what is new or changed and leaves what is the same', () => {
		const plan = syncPlan(
			source,
			{ 'skills/kit/SKILL.md': '# Kit\n', 'skills/kit/BUILD.md': '# Old\n' },
			entry.files
		);

		expect(plan).toEqual({
			write: ['skills/kit/BUILD.md'],
			same: ['skills/kit/SKILL.md'],
			remove: [],
		});
	});

	it('removes a file the package wrote before and no longer ships', () => {
		const written = [...entry.files, 'skills/kit/GONE.md'];
		const plan = syncPlan(source, { ...source, 'skills/kit/GONE.md': 'x' }, written);

		expect(plan.remove).toEqual(['skills/kit/GONE.md']);
	});

	it('never removes a file the package did not write: the repo’s own stay', () => {
		const copy = { ...source, 'skills/kit/PROJECT.md': 'ours', 'skills/kit/NOTES.md': 'ours' };

		expect(syncPlan(source, copy, entry.files).remove).toEqual([]);
	});

	it('refuses a package that ships a skill’s PROJECT.md', () => {
		expect(() => syncPlan({ ...source, 'skills/kit/PROJECT.md': 'theirs' }, {}, [])).toThrow(
			/PROJECT\.md/
		);
	});
});

describe('staleness', () => {
	it('finds nothing wrong with a copy that is the package’s files as they stand', () => {
		const copy = { ...source, 'skills/kit/PROJECT.md': 'ours' };

		expect(staleness(PKG, source, copy, entry)).toEqual([]);
	});

	it('names a package that was never synced', () => {
		expect(staleness(PKG, source, source, undefined)).toEqual(['@acme/kit was never synced']);
	});

	it('names the version it was synced at when another is installed', () => {
		expect(staleness(PKG, source, source, { ...entry, version: '1.1.0' })).toEqual([
			'@acme/kit was synced at 1.1.0, 1.2.0 is installed',
		]);
	});

	it('names each file that is missing, differs, or was dropped', () => {
		const copy = { 'skills/kit/SKILL.md': '# Edited here\n', 'skills/kit/GONE.md': 'x' };
		const earlier = { ...entry, files: [...entry.files, 'skills/kit/GONE.md'] };

		expect(staleness(PKG, source, copy, earlier)).toEqual([
			'skills/kit/BUILD.md is missing',
			'skills/kit/SKILL.md differs from @acme/kit’s',
			'skills/kit/GONE.md is no longer part of @acme/kit',
		]);
	});
});
