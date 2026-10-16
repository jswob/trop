import { databaseSsl, withPreparedStatements } from '@jswob/drizzle-utils';
import { drizzle } from 'drizzle-orm/postgres-js';
import postgres from 'postgres';
import * as schema from './schema';
import { selectedDbTarget } from './target';

if (process.env.NODE_ENV === 'production' && !process.env.DATABASE_URL) {
	throw new Error('DATABASE_URL environment variable is required in production');
}

const connectionString = selectedDbTarget().url;

const client = postgres(connectionString, {
	ssl: databaseSsl(connectionString),
	idle_timeout: 20,
	max_lifetime: 60 * 30,
	connect_timeout: 15,
});

/**
 * Queries are sent as prepared statements (`withPreparedStatements`). `DATABASE_PREPARE=false`
 * turns that off without a new build, for a pooler that does not carry them.
 */
const prepare = process.env.DATABASE_PREPARE !== 'false';

export const db = drizzle(prepare ? withPreparedStatements(client) : client, { schema });
