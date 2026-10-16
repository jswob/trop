import type { Env } from './db-target';

/**
 * Whether a connection to `url` uses TLS: `DATABASE_SSL` (`true` or `false`) when it is set,
 * otherwise whatever the URL asks for with `sslmode=require`. The app's connection and drizzle-kit
 * both ask here, so the two can never disagree about one database.
 * @param url - The connection URL
 * @param env - Environment to read, `process.env` by default
 */
export function databaseSsl(url: string, env: Env = process.env): 'require' | false {
	if (env.DATABASE_SSL) {
		return env.DATABASE_SSL === 'true' ? 'require' : false;
	}
	return url.includes('sslmode=require') ? 'require' : false;
}
