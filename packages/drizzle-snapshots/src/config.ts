/**
 * What an app tells the CLI, in `drizzle-snapshots.config.ts` beside its `drizzle.config.ts`. The
 * CLI runs from that folder, and every path here is relative to it.
 */

import { existsSync } from 'fs';
import { join } from 'path';

export type DbTargetName = 'app' | 'test' | 'e2e';

export interface DbTarget {
	/** Database name. */
	name: string;
	port: string;
	/** Full connection URL. */
	url: string;
}

/** A constraint Drizzle's schema builder cannot express, such as `EXCLUDE`. */
export interface RawConstraint {
	table: string;
	name: string;
	/** What follows `ADD CONSTRAINT <name>`. */
	definition: string;
	/** Why it exists, in a few words. */
	purpose: string;
}

export interface SnapshotsConfig {
	/** The app's database connections; `createDbTargets` in `@jswob/drizzle-utils` makes one. */
	dbTarget: (target: DbTargetName) => DbTarget;
	/** The script that runs the committed migrations against the database `DB_TARGET` names. */
	migrate: string;
	/** The script that seeds the database `DB_TARGET` names; `e2e-setup` runs it. */
	seed: string;
	/** Re-applied after every push and test setup, since a push that recreates a table drops them. */
	rawConstraints?: RawConstraint[];
	/** Where the schema lives: a staged change here makes `schema-check` run. */
	schema?: string[];
	/** drizzle-kit's migrations folder. */
	migrations?: string;
}

export type ResolvedConfig = Required<SnapshotsConfig>;

export const CONFIG_FILE = 'drizzle-snapshots.config.ts';

/**
 * Type the app's config.
 * @param config - The app's config
 */
export function defineConfig(config: SnapshotsConfig): SnapshotsConfig {
	return config;
}

/**
 * The config in `cwd`, with every default filled in.
 * @param cwd - The app's folder
 */
export async function loadConfig(cwd: string): Promise<ResolvedConfig> {
	const path = join(cwd, CONFIG_FILE);
	if (!existsSync(path)) {
		throw new Error(`No ${CONFIG_FILE} in ${cwd}; run the CLI from the app's folder`);
	}
	const { default: config } = (await import(path)) as { default: SnapshotsConfig };
	return {
		rawConstraints: [],
		schema: ['src/db/schema/'],
		migrations: 'drizzle/',
		...config,
	};
}
