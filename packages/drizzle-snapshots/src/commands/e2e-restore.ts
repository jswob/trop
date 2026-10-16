/**
 * E2E Database Restore Script
 *
 * Restores the E2E database from the baseline snapshot.
 * This provides fast (~2-5 second) database restoration for test isolation.
 */
import { listVolumes, replaceContents, withStopped } from '../docker';
import { composeProject, containerFor, projectVolumes } from '../volumes';

export function e2eRestore(): void {
	const startTime = Date.now();
	const names = projectVolumes(composeProject());

	if (!listVolumes().includes(names.e2eBase)) {
		console.log('💡 Run "bun e2e:setup" to create the baseline snapshot');
		throw new Error(`Baseline snapshot not found: ${names.e2eBase}`);
	}

	withStopped(containerFor('e2e'), () => {
		console.log('🔄 Restoring from baseline snapshot...');
		replaceContents(names.e2eBase, names.e2e);
		console.log('✅ Database restored from baseline');
	});

	const duration = ((Date.now() - startTime) / 1000).toFixed(2);
	console.log(`⏱️ Restore completed in ${duration}s`);
}
