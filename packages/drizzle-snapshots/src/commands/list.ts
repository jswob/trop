import { existsSync, readFileSync } from 'fs';
import { join } from 'path';
import { listVolumes, run } from '../docker';
import {
	composeProject,
	isLegacySnapshot,
	projectBackups,
	projectSnapshots,
	projectVolumes,
} from '../volumes';

const LAST_PUSH_FILE = join(process.cwd(), '.last_push_snapshot');

function formatDate(timestamp: number): string {
	return new Date(timestamp * 1000).toLocaleString('en-US', {
		year: 'numeric',
		month: '2-digit',
		day: '2-digit',
		hour: '2-digit',
		minute: '2-digit',
		second: '2-digit',
	});
}

function getVolumeSize(volumeName: string): string {
	try {
		const sizeOutput = run(
			`docker run --rm -v ${volumeName}:/data alpine du -sh /data`,
			`size ${volumeName}`
		);
		return sizeOutput.split('\t')[0] || 'Unknown';
	} catch {
		return 'Unknown';
	}
}

/** Print this project's snapshots, safety copies, and the volumes from before project names. */
export function list() {
	const names = projectVolumes(composeProject());
	const volumes = listVolumes();
	const snapshots = projectSnapshots(volumes, names);
	const base = snapshots.find(s => s.isBase);
	const regular = snapshots.filter(s => !s.isBase);

	let lastPushSnapshot: string | null = null;
	if (existsSync(LAST_PUSH_FILE)) {
		try {
			lastPushSnapshot = readFileSync(LAST_PUSH_FILE, 'utf8').trim();
		} catch {
			// Ignore errors reading last push file
		}
	}

	console.log(`📊 Database Snapshots (${composeProject()}):\n`);

	if (base) {
		console.log(`🔒 base                    (${getVolumeSize(base.volume)})    PROTECTED BASELINE`);
	} else {
		console.log('🔒 base                    (missing)   💡 Create with: bun db:base');
	}

	if (regular.length === 0) {
		console.log('\n📭 No regular snapshots found');
		console.log('💡 Create snapshots with: bun db:snapshot [name]');
		console.log('💡 Auto-create with: bun db:push:safe');
	} else {
		console.log('\n📸 Regular Snapshots:');

		const nameWidth = Math.max(...regular.map(s => s.name.length), 15);

		regular.forEach((snapshot, index) => {
			let indicators = '';
			if (index === 0) indicators += ' ⭐';
			if (snapshot.volume === lastPushSnapshot) indicators += ' 🔄';

			const dateStr = snapshot.createdAt !== null ? formatDate(snapshot.createdAt) : 'Custom';
			const sizeStr = `(${getVolumeSize(snapshot.volume)})`.padEnd(10);

			console.log(`   ${snapshot.name.padEnd(nameWidth)} ${sizeStr} ${dateStr}${indicators}`);
		});

		console.log('\n🔄 = Can be undone with "bun db:undo"');
		console.log('⭐ = Latest snapshot');
	}

	const backups = projectBackups(volumes, names);
	if (backups.length > 0) {
		console.log(
			`\n💾 ${backups.length} safety backup(s) from restores; "bun db:clean" lists and removes them`
		);
	}

	const legacy = volumes.filter(isLegacySnapshot);
	if (legacy.length > 0) {
		console.log(
			'\n📦 Snapshots from before per-project names (any project on this machine may own them):'
		);
		legacy.forEach(volume => console.log(`   ${volume}`));
		console.log(
			'   Never pruned or removed here. Restore one by its full name: bun db:restore <volume>'
		);
	}

	console.log('\n💡 Commands:');
	console.log('  bun db:restore              - Restore latest snapshot');
	console.log('  bun db:restore base         - Restore baseline');
	console.log('  bun db:restore <name>       - Restore specific snapshot');
	console.log('  bun db:undo                 - Undo last push');
	console.log('  bun db:base                 - Create/update baseline');
	console.log('  bun db:snapshot [name]      - Create new snapshot');
}
