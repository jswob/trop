import { defineConfig } from 'drizzle-kit';
import { dbTarget } from './src/db/target';

export default defineConfig({
	dialect: 'postgresql',
	schema: './src/db/schema/*.ts',
	out: './drizzle',
	dbCredentials: { url: dbTarget('e2e').url, ssl: false },
});
