import { copyToNewVolume, listVolumes, removeVolume, withStopped } from '../docker';
import {
	composeProject,
	containerFor,
	projectSnapshots,
	projectVolumes,
	snapshotsToPrune,
} from '../volumes';

/** Timestamped snapshots kept besides the base; older ones are removed after each new one. */
const KEEP = 4;

function pruneOldSnapshots(names: ReturnType<typeof projectVolumes>) {
	const toDelete = snapshotsToPrune(projectSnapshots(listVolumes(), names), KEEP);
	if (toDelete.length === 0) return;

	console.log(`🧹 Cleaning up ${toDelete.length} old snapshot(s)...`);
	for (const snapshot of toDelete) {
		removeVolume(snapshot.volume);
		console.log(`  ❌ Removed: ${snapshot.name}`);
	}
}

/**
 * Copy the dev database into a snapshot: `base` replaces the protected baseline, a name makes a
 * named one, nothing makes a timestamped one and prunes the oldest beyond four.
 * @param name - `base`, a name, or nothing
 */
export function snapshot(name?: string) {
	const names = projectVolumes(composeProject());
	const isBase = name === 'base';
	const volume = isBase
		? names.base
		: `${names.snapshotPrefix}${name ?? Math.floor(Date.now() / 1000)}`;

	console.log(`📸 Creating ${isBase ? 'BASE' : 'snapshot'}: ${volume}`);

	withStopped(containerFor('app'), () => {
		if (listVolumes().includes(volume)) {
			if (!isBase) throw new Error(`A snapshot named ${name} already exists`);
			removeVolume(volume);
			console.log('  🔄 Updating existing base snapshot');
		} else if (isBase) {
			console.log('  ✨ Creating new base snapshot');
		}

		copyToNewVolume(names.main, volume);
		if (!isBase) pruneOldSnapshots(names);
	});

	console.log(`✅ ${isBase ? 'Base snapshot' : 'Snapshot'} created successfully`);
	if (isBase) {
		console.log('🔒 Base snapshot is protected and will never be auto-deleted');
	}
}
