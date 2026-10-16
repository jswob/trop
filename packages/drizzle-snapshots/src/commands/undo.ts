import { existsSync, readFileSync, unlinkSync } from 'fs';
import { join } from 'path';
import {
	copyToNewVolume,
	listVolumes,
	removeVolume,
	replaceContents,
	withStopped,
} from '../docker';
import { composeProject, containerFor, isLegacySnapshot, projectVolumes } from '../volumes';

const LAST_PUSH_FILE = join(process.cwd(), '.last_push_snapshot');

/** Put the dev database back as it was before the last `push`, and drop that push's snapshot. */
export function undo() {
	if (!existsSync(LAST_PUSH_FILE)) {
		console.log('💡 Use "bun db:push:safe" to create trackable pushes');
		throw new Error('No push to undo');
	}

	const names = projectVolumes(composeProject());
	const lastPushSnapshot = readFileSync(LAST_PUSH_FILE, 'utf8').trim();

	if (isLegacySnapshot(lastPushSnapshot)) {
		console.log(`💡 Restore it by name instead: bun db:restore ${lastPushSnapshot}`);
		throw new Error(
			`The last push was recorded before per-project snapshot names; ${lastPushSnapshot} is left untouched`
		);
	}
	if (!lastPushSnapshot.startsWith(names.snapshotPrefix)) {
		throw new Error(`The last push snapshot ${lastPushSnapshot} belongs to another project`);
	}
	if (!listVolumes().includes(lastPushSnapshot)) {
		unlinkSync(LAST_PUSH_FILE);
		console.log('💡 It may have been cleaned up or manually removed');
		throw new Error(`Last push snapshot no longer exists: ${lastPushSnapshot}`);
	}

	console.log(`⏪ Undoing last push (restoring: ${lastPushSnapshot})`);

	withStopped(containerFor('app'), () => {
		const backup = `${names.backupPrefix}${Math.floor(Date.now() / 1000)}`;
		console.log('💾 Creating safety backup...');
		copyToNewVolume(names.main, backup);

		console.log('🔄 Restoring to pre-push state...');
		replaceContents(lastPushSnapshot, names.main);
		console.log(`💾 Current state backed up to: ${backup}`);

		console.log('🗑️  Removing used snapshot...');
		removeVolume(lastPushSnapshot);
		unlinkSync(LAST_PUSH_FILE);
	});

	console.log('✅ Last push undone successfully');
	console.log('🎯 Database restored to state before last push');
}
