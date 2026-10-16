import { join } from 'path';
import { loadEnv } from 'vite';
import { afterAll } from 'vitest';
import { closeTestDb } from './src/test/db';

const rootDir = join(__dirname, '../..');
const env = loadEnv('test', rootDir, '');
Object.assign(process.env, env);

// Anything that opens the app's own connection during a test reaches the test database.
process.env.DB_TARGET = 'test';

afterAll(async () => {
	await closeTestDb();
});
