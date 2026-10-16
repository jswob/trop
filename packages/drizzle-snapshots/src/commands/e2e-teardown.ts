/**
 * E2E Database Teardown Script
 *
 * Removes the E2E baseline snapshot volume.
 * Called at the end of E2E test suite to clean up resources.
 */
import { listVolumes, removeVolume } from '../docker';
import { composeProject, projectVolumes } from '../volumes';

export function e2eTeardown(): void {
	console.log('🧪 E2E teardown...\n');
	const names = projectVolumes(composeProject());

	console.log('🧹 Removing E2E baseline snapshot...');
	if (!listVolumes().includes(names.e2eBase)) {
		console.log('ℹ️ Baseline snapshot does not exist, nothing to remove');
	} else {
		removeVolume(names.e2eBase);
		console.log('✅ Baseline snapshot removed');
	}

	console.log('\n✅ E2E teardown complete');
}
