import { execSync } from 'child_process';
import { writeFileSync } from 'fs';
import { join } from 'path';
import type { ResolvedConfig } from '../config';
import { copyToNewVolume, listVolumes, removeVolume, withStopped } from '../docker';
import { DRIZZLE_KIT } from '../drizzle-kit';
import { applyRawConstraints } from '../raw-constraints';
import {
	composeProject,
	containerFor,
	projectSnapshots,
	projectVolumes,
	snapshotsToPrune,
} from '../volumes';

const LAST_PUSH_FILE = join(process.cwd(), '.last_push_snapshot');

/** Timestamped snapshots kept besides the base; older ones are removed after each push. */
const KEEP = 4;

/**
 * Runs a command attached to this terminal, so a prompt it asks — drizzle-kit's data-loss
 * confirmation — is shown and answered. A piped command hides the prompt and leaves the push
 * waiting for an answer that can never come.
 */
function executeInteractive(command: string): void {
	execSync(command, { stdio: 'inherit' });
}

/**
 * Snapshot the dev database, run `drizzle-kit push`, then re-apply the raw constraints a push
 * drops. `undo` puts the snapshot back.
 * @param config - The app's config
 */
export async function push(config: ResolvedConfig) {
	const names = projectVolumes(composeProject());
	const snapshotVolume = `${names.snapshotPrefix}before_push_${Math.floor(Date.now() / 1000)}`;

	console.log('📸 Creating pre-push snapshot...');

	withStopped(containerFor('app'), () => {
		copyToNewVolume(names.main, snapshotVolume);
		console.log(`✅ Pre-push snapshot created: ${snapshotVolume}`);

		// Track this snapshot for undo functionality
		writeFileSync(LAST_PUSH_FILE, snapshotVolume, 'utf8');

		const toDelete = snapshotsToPrune(projectSnapshots(listVolumes(), names), KEEP);
		if (toDelete.length > 0) {
			console.log(`🧹 Cleaning up ${toDelete.length} old snapshot(s)...`);
			for (const snapshot of toDelete) {
				removeVolume(snapshot.volume);
				console.log(`  ❌ Removed: ${snapshot.name}`);
			}
		}
	});

	console.log('🚀 Running drizzle-kit push...');

	try {
		executeInteractive(`${DRIZZLE_KIT} push`);
		console.log('✅ Push completed successfully');
	} catch {
		console.log('💡 Use "bun db:undo" to restore pre-push state');
		throw new Error('Push failed, but snapshot is preserved');
	}

	// Drizzle does not manage EXCLUDE constraints, and a push that recreates a
	// table drops them — re-apply them here so a push never silently weakens
	// the schema.
	console.log('🔒 Re-applying raw constraints...');
	try {
		const created = await applyRawConstraints(config.dbTarget('app').url, config.rawConstraints);
		console.log(
			created.length > 0
				? `✅ Applied: ${created.join(', ')}`
				: '✅ Raw constraints already present'
		);
	} catch (error) {
		console.log('💡 Fix the offending data, then run "bun db:constraints"');
		throw new Error(`Failed to apply raw constraints: ${String(error)}`, { cause: error });
	}

	console.log('💡 Use "bun db:undo" to revert if needed');
}
