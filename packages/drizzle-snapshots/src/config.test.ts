import { mkdtempSync, rmSync, writeFileSync } from 'fs';
import { tmpdir } from 'os';
import { join } from 'path';
import { afterEach, beforeEach, describe, expect, test } from 'vitest';
import { CONFIG_FILE, loadConfig } from './config';

let dir: string;

beforeEach(() => {
	dir = mkdtempSync(join(tmpdir(), 'drizzle-snapshots-'));
});

afterEach(() => {
	rmSync(dir, { recursive: true, force: true });
});

describe('loadConfig', () => {
	test('fills in what the app leaves out', async () => {
		writeFileSync(
			join(dir, CONFIG_FILE),
			`export default {
				dbTarget: () => ({ name: 'n', port: '1', url: 'postgresql://u@h:1/n' }),
				migrate: 'migrate.ts',
				seed: 'seed.ts',
			};`
		);

		const config = await loadConfig(dir);

		expect(config).toMatchObject({
			migrate: 'migrate.ts',
			seed: 'seed.ts',
			rawConstraints: [],
			schema: ['src/db/schema/'],
			migrations: 'drizzle/',
		});
		expect(config.dbTarget('app').name).toBe('n');
	});

	test('keeps what the app gives', async () => {
		writeFileSync(
			join(dir, CONFIG_FILE),
			`export default {
				dbTarget: () => ({ name: 'n', port: '1', url: '' }),
				migrate: 'm.ts',
				seed: 's.ts',
				schema: ['db/'],
				migrations: 'migrations/',
			};`
		);

		expect(await loadConfig(dir)).toMatchObject({ schema: ['db/'], migrations: 'migrations/' });
	});

	test('names the file it could not find', async () => {
		await expect(loadConfig(dir)).rejects.toThrow(CONFIG_FILE);
	});
});
