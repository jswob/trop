/**
 * The steps that shape the test and e2e databases: each runs from the app's folder, and each
 * reaches its database by name, never through `DATABASE_URL`.
 */

import { execSync } from 'child_process';
import type { DbTarget } from './config';
import { run } from './docker';
import { DRIZZLE_KIT } from './drizzle-kit';

/**
 * Whether a container publishes the target's port.
 * @param target - The database
 */
export function isPublished(target: DbTarget): boolean {
	try {
		execSync(`docker ps --filter "publish=${target.port}" --format "{{.Names}}" | grep -q .`, {
			stdio: 'pipe',
		});
		return true;
	} catch {
		return false;
	}
}

/**
 * Drop and create the target's database inside its compose container.
 * @param target - The database
 * @param container - The container that serves it
 */
export function recreateDatabase(target: DbTarget, container: string): void {
	const user = decodeURIComponent(new URL(target.url).username) || 'postgres';
	run(
		`docker exec ${container} psql -U ${user} -c "DROP DATABASE IF EXISTS ${target.name};"`,
		`drop ${target.name}`
	);
	run(
		`docker exec ${container} psql -U ${user} -c "CREATE DATABASE ${target.name};"`,
		`create ${target.name}`
	);
}

/**
 * Run one of the app's scripts (its migration, its seed) against the test or e2e database.
 * @param target - `DB_TARGET` for the script
 * @param script - The script, from the app's folder
 * @param description - What it does, for the error message
 */
export function runAgainst(target: 'test' | 'e2e', script: string, description: string): void {
	try {
		execSync(`bun ${script}`, {
			stdio: 'inherit',
			cwd: process.cwd(),
			env: { ...process.env, DB_TARGET: target },
		});
	} catch {
		throw new Error(`Failed to ${description}`);
	}
}

/**
 * Push the schema with the target's own drizzle config, `drizzle.config.<target>.ts`.
 * @param target - Which database
 */
export function pushSchema(target: 'test' | 'e2e'): void {
	try {
		execSync(`${DRIZZLE_KIT} push --force --config=drizzle.config.${target}.ts`, {
			stdio: 'inherit',
			cwd: process.cwd(),
		});
	} catch {
		throw new Error('Failed to push schema changes');
	}
}
