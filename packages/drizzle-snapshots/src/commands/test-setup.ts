/**
 * Test Database Setup
 *
 * 1. Drop and recreate the database (clean slate)
 * 2. Run migrations (apply committed schema changes)
 * 3. Run push --force (apply uncommitted schema changes)
 * 4. Re-apply the raw constraints, which the push leaves off
 *
 * Every step reaches the database through `dbTarget('test')`; the migration runs with
 * `DB_TARGET=test`, so a `DATABASE_URL` in the environment is never touched. In CI the database
 * comes from a service container, so the docker steps are skipped.
 */
import type { ResolvedConfig } from '../config';
import { isPublished, pushSchema, recreateDatabase, runAgainst } from '../database';
import { applyRawConstraints } from '../raw-constraints';
import { containerFor } from '../volumes';

/**
 * Make the test database match the schema: migrations, then a push, then the raw constraints.
 * @param config - The app's config
 */
export async function testSetup(config: ResolvedConfig): Promise<void> {
	const target = config.dbTarget('test');
	console.log('🧪 Setting up test database...');

	if (process.env.CI === 'true') {
		console.log('🤖 CI environment detected — skipping Docker checks');
	} else {
		console.log(`🔍 Checking test database on port ${target.port}...`);
		if (!isPublished(target)) {
			throw new Error('Test database is not running. Please run "bun db:up" first.');
		}
		console.log('✅ Test database container is running');

		console.log(`🗄️ Creating fresh test database (${target.name})...`);
		recreateDatabase(target, containerFor('test'));
		console.log('✅ Test database created');
	}

	console.log('📦 Running migrations (committed schema)...');
	runAgainst('test', config.migrate, 'run migrations');
	console.log('✅ Migrations completed');

	console.log('🔍 Checking and applying schema changes...');
	pushSchema('test');
	console.log('✅ Schema push completed');
	console.log('✅ Test database schema setup complete');

	console.log('🔒 Applying raw constraints...');
	try {
		const created = await applyRawConstraints(target.url, config.rawConstraints);
		console.log(
			created.length > 0
				? `✅ Applied: ${created.join(', ')}`
				: '✅ Raw constraints already present'
		);
	} catch (error) {
		throw new Error(`Failed to apply raw constraints: ${String(error)}`, { cause: error });
	}

	console.log('🎉 Test database setup complete!');
}
