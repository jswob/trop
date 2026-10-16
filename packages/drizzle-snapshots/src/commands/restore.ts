import { copyToNewVolume, listVolumes, replaceContents, withStopped } from '../docker';
import {
	composeProject,
	containerFor,
	findSnapshot,
	projectSnapshots,
	projectVolumes,
	type ProjectVolumes,
	type Snapshot,
} from '../volumes';

function chooseSnapshot(query: string | undefined, names: ProjectVolumes): Snapshot {
	const volumes = listVolumes();
	const snapshots = projectSnapshots(volumes, names);

	if (snapshots.length === 0 && !query) {
		console.log('💡 Create your first snapshot with: bun db:snapshot');
		throw new Error('No snapshots found');
	}

	if (!query) {
		const latest = snapshots.find(s => !s.isBase && s.createdAt !== null);
		if (!latest) {
			console.log('💡 Use "bun db:restore base" or name a snapshot');
			throw new Error('No timestamped snapshot to restore');
		}
		return latest;
	}

	const found = findSnapshot(query, volumes, names);
	if (!found) {
		console.log('\nAvailable snapshots:');
		snapshots.forEach(s => console.log(`  - ${s.name}${s.isBase ? ' 🔒 (BASE)' : ''}`));
		throw new Error(`Snapshot not found: ${query}`);
	}
	return found;
}

/**
 * Replace the dev database with a snapshot, keeping a safety copy of what it held.
 * @param query - `base`, a snapshot's name or timestamp, or nothing for the latest
 */
export function restore(query?: string) {
	const names = projectVolumes(composeProject());
	const snapshot = chooseSnapshot(query, names);

	console.log(`🔄 Restoring from: ${snapshot.name}${snapshot.isBase ? ' 🔒 (BASE)' : ''}`);

	withStopped(containerFor('app'), () => {
		const backup = `${names.backupPrefix}${Math.floor(Date.now() / 1000)}`;
		console.log('💾 Creating safety backup...');
		copyToNewVolume(names.main, backup);

		console.log('🔄 Restoring snapshot...');
		replaceContents(snapshot.volume, names.main);

		console.log(`💾 Current state backed up to: ${backup}`);
		console.log('💡 "bun db:clean" lists and removes old backups');
	});

	console.log('✅ Database restored successfully');
	if (snapshot.isBase) {
		console.log('🎯 You are now back to your protected baseline state');
	}
}
