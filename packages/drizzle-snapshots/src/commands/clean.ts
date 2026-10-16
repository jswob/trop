import { existsSync, unlinkSync } from 'fs';
import { join } from 'path';
import { listVolumes, removeVolume, replaceContents, withStopped } from '../docker';
import {
	composeProject,
	containerFor,
	isLegacySnapshot,
	projectBackups,
	projectSnapshots,
	projectVolumes,
} from '../volumes';

const LAST_PUSH_FILE = join(process.cwd(), '.last_push_snapshot');

/**
 * List this project's snapshots and safety copies; with `force`, restore the baseline and remove
 * them all.
 * @param force - Remove rather than list
 */
export function clean(force: boolean) {
	const names = projectVolumes(composeProject());
	const volumes = listVolumes();
	const snapshots = projectSnapshots(volumes, names);
	const backups = projectBackups(volumes, names);
	const legacy = volumes.filter(isLegacySnapshot);

	if (snapshots.length === 0 && backups.length === 0) {
		console.log('✨ No snapshots or backups found - already clean!');
		return;
	}

	console.log(`🧹 Database Clean Operation (${composeProject()})`);
	console.log('This will:');
	console.log('  1. Restore baseline (if exists) to preserve clean data');
	console.log("  2. Remove ALL of this project's snapshots, the protected baseline included");
	console.log("  3. Remove this project's safety backups");
	console.log('  4. Clear push tracking files');
	console.log('');
	console.log(`📋 Found ${snapshots.length + backups.length} volume(s) to remove:`);
	snapshots.forEach(s => console.log(`  - ${s.volume}${s.isBase ? ' 🔒 (PROTECTED BASE)' : ''}`));
	backups.forEach(volume => console.log(`  - ${volume} 💾 (backup)`));
	if (legacy.length > 0) {
		console.log(`\n📦 Left alone: ${legacy.length} snapshot(s) from before per-project names`);
	}

	if (!force) {
		console.log('');
		console.log('⚠️  This action cannot be undone!');
		console.log('💡 Run: bun db:clean --force');
		throw new Error('Aborted - use --force to proceed');
	}

	console.log('');
	console.log('🚀 Starting clean operation...');

	withStopped(containerFor('app'), () => {
		const base = snapshots.find(s => s.isBase);
		if (base) {
			console.log('📦 Restoring baseline to preserve clean data...');
			replaceContents(base.volume, names.main);
			console.log('✅ Baseline restored to main database');
		} else {
			console.log('ℹ️  No baseline found - keeping current database state');
		}

		console.log('🗑️  Removing snapshot and backup volumes...');
		for (const volume of [...snapshots.map(s => s.volume), ...backups]) {
			removeVolume(volume);
			console.log(`  ❌ Removed: ${volume}`);
		}

		if (existsSync(LAST_PUSH_FILE)) {
			unlinkSync(LAST_PUSH_FILE);
			console.log('🗑️  Cleared push tracking file');
		}
	});

	console.log('');
	console.log('✅ Clean operation completed successfully!');
	console.log('💡 Use "bun db:base" to create a new protected baseline');
}
