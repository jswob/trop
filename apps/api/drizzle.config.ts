import { databaseSsl } from '@jswob/drizzle-utils';
import { defineConfig } from 'drizzle-kit';
import { dbTarget } from './src/db/target';

const { url } = dbTarget('app');

export default defineConfig({
	dialect: 'postgresql',
	schema: './src/db/schema/*.ts',
	out: './drizzle',
	dbCredentials: { url, ssl: databaseSsl(url) },
});
