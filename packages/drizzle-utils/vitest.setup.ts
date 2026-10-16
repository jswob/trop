import { join } from 'path';
import { loadEnv } from 'vite';

/** The test database's credentials come from the repo's `.env`, as the app's own tests read them. */
Object.assign(process.env, loadEnv('test', join(__dirname, '../..'), ''));
