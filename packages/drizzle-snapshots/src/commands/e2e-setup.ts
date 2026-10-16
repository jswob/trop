/**
 * E2E Database Setup
 *
 * 1. Drop and recreate the E2E database (clean slate)
 * 2. Run migrations (apply committed schema changes)
 * 3. Run push (apply uncommitted schema changes)
 * 4. Run the app's seed (populate with test data)
 * 5. Create the baseline snapshot `e2e-restore` puts back before every test
 *
 * The migration and the seed run with `DB_TARGET=e2e`. A seed empties every table first, so it
 * must never reach a database named by a stray `DATABASE_URL`.
 */
import type { ResolvedConfig } from '../config';
import { isPublished, pushSchema, recreateDatabase, runAgainst } from '../database';
import { copyToNewVolume, listVolumes, removeVolume, withStopped } from '../docker';
import { composeProject, containerFor, projectVolumes } from '../volumes';

function createBaselineSnapshot(): void {
	const names = projectVolumes(composeProject());
	console.log('📸 Creating E2E baseline snapshot...');

	withStopped(containerFor('e2e'), () => {
		if (listVolumes().includes(names.e2eBase)) {
			removeVolume(names.e2eBase);
			console.log('  🔄 Updating existing baseline snapshot');
		} else {
			console.log('  ✨ Creating new baseline snapshot');
		}

		copyToNewVolume(names.e2e, names.e2eBase);
	});

	console.log('✅ Baseline snapshot created');
	console.log('🔒 Baseline snapshot will be used for fast test restoration');
}

/**
 * Build the e2e database from scratch, seed it, and snapshot it as the baseline.
 * @param config - The app's config
 */
export function e2eSetup(config: ResolvedConfig): void {
	const target = config.dbTarget('e2e');
	console.log('🧪 Setting up E2E database...\n');

	console.log(`🔍 Checking E2E database on port ${target.port}...`);
	if (!isPublished(target)) {
		throw new Error(
			'E2E database is not running. Please run "docker compose up -d postgres-e2e" first.'
		);
	}
	console.log('✅ E2E database container is running');

	console.log(`🗄️ Creating fresh E2E database (${target.name})...`);
	recreateDatabase(target, containerFor('e2e'));
	console.log('✅ E2E database created');

	console.log('📦 Running migrations on E2E database...');
	runAgainst('e2e', config.migrate, 'run migrations');
	console.log('✅ Migrations completed');

	console.log('🔍 Checking and applying schema changes...');
	pushSchema('e2e');
	console.log('✅ Schema push completed');

	console.log('🌱 Seeding E2E database...');
	runAgainst('e2e', config.seed, 'seed the E2E database');
	console.log('✅ Seeding completed');

	createBaselineSnapshot();

	console.log('\n🎉 E2E database setup complete!');
	console.log('💡 Run "bun e2e" to execute E2E tests');
}
