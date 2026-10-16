import { join } from 'path';
import { loadEnv } from 'vite';

const rootDir = join(__dirname, '../..');
const env = loadEnv('test', rootDir, '');

Object.assign(process.env, env);
