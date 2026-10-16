import { describe, expect, test } from 'vitest';

import { isPublished, type Manifest, packedProblems, publishOrder } from './release';

const versions = new Map([
	['@acme/kit', '0.3.1'],
	['@acme/engine', '1.2.0'],
]);

describe('packedProblems', () => {
	test('passes a manifest whose ranges admit the versions here', () => {
		const packed: Manifest = {
			name: '@acme/engine',
			version: '1.2.0',
			dependencies: { '@acme/kit': '^0.3.1', zod: '^3.24.2' },
		};
		expect(packedProblems(packed, versions)).toEqual([]);
	});

	test('names a range that leaves out the version here, as a stale lock packs it', () => {
		const packed: Manifest = {
			name: '@acme/engine',
			version: '1.2.0',
			dependencies: { '@acme/kit': '0.3.0' },
		};
		expect(packedProblems(packed, versions)).toEqual([
			'dependencies @acme/kit is "0.3.0", which leaves out 0.3.1',
		]);
	});

	test('names a protocol the pack did not resolve', () => {
		const packed: Manifest = {
			name: '@acme/engine',
			version: '1.2.0',
			peerDependencies: { '@acme/kit': 'workspace:^' },
			optionalDependencies: { local: 'file:../local' },
		};
		expect(packedProblems(packed, versions)).toEqual([
			'peerDependencies @acme/kit is still "workspace:^"',
			'optionalDependencies local is still "file:../local"',
		]);
	});

	test('names a version that is not the package’s', () => {
		const packed: Manifest = { name: '@acme/kit', version: '0.3.0' };
		expect(packedProblems(packed, versions)).toEqual(['packed as 0.3.0, but the package is 0.3.1']);
	});
});

describe('publishOrder', () => {
	test('puts each package after the packages here it installs', () => {
		const order = publishOrder<Manifest>([
			{ name: 'app', version: '1.0.0', dependencies: { engine: '^1.0.0' } },
			{ name: 'engine', version: '1.0.0', peerDependencies: { kit: '^1.0.0' } },
			{ name: 'kit', version: '1.0.0', dependencies: { zod: '^3.0.0' } },
		]);
		expect(order.map(manifest => manifest.name)).toEqual(['kit', 'engine', 'app']);
	});

	test('refuses a cycle', () => {
		expect(() =>
			publishOrder<Manifest>([
				{ name: 'a', version: '1.0.0', dependencies: { b: '^1.0.0' } },
				{ name: 'b', version: '1.0.0', dependencies: { a: '^1.0.0' } },
			])
		).toThrow('Dependency cycle: a → b → a');
	});
});

describe('isPublished', () => {
	const kit: Manifest = { name: '@acme/kit', version: '0.3.1' };
	const answer = (status: number, body: unknown = {}) => {
		const urls: string[] = [];
		const get = (url: string) => {
			urls.push(url);
			return Promise.resolve(new Response(JSON.stringify(body), { status }));
		};
		return { get, urls };
	};

	test('asks the registry past its CDN, for the scoped name', async () => {
		const { get, urls } = answer(200, { versions: { '0.3.1': {} } });
		expect(await isPublished('https://registry.example/', kit, get)).toBe(true);
		expect(urls).toEqual(['https://registry.example/@acme%2fkit?write=true']);
	});

	test('is false for a version the package does not have yet', async () => {
		const { get } = answer(200, { versions: { '0.3.0': {} } });
		expect(await isPublished('https://registry.example', kit, get)).toBe(false);
	});

	test('is false for a package the registry does not have', async () => {
		const { get } = answer(404);
		expect(await isPublished('https://registry.example', kit, get)).toBe(false);
	});

	test('throws when the registry cannot answer', async () => {
		const { get } = answer(503);
		await expect(isPublished('https://registry.example', kit, get)).rejects.toThrow(
			'https://registry.example/@acme%2fkit?write=true answered 503'
		);
	});
});
