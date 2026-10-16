import { execSync } from 'child_process';
import path from 'path';
import { fileURLToPath } from 'url';

// ESM-compatible directory path
const _currentFile = fileURLToPath(import.meta.url);
const _currentDir = path.dirname(_currentFile);

/**
 * Global teardown function that runs once after all tests.
 * Cleans up the E2E baseline snapshot.
 */
function globalTeardown(): void {
	// eslint-disable-next-line no-console -- Reason: Intentional logging for E2E teardown visibility
	console.log('\n🧹 Running E2E global teardown...');

	// Path to the API directory
	const apiDir = path.resolve(_currentDir, '../../api');

	try {
		// Run e2e:teardown to clean up the baseline snapshot
		execSync('bun e2e:teardown', {
			cwd: apiDir,
			env: {
				...process.env,
				NODE_ENV: 'test',
			},
			stdio: 'inherit',
		});

		// eslint-disable-next-line no-console -- Reason: Intentional logging for E2E teardown visibility
		console.log('✅ E2E global teardown complete\n');
	} catch (error) {
		console.error('❌ E2E global teardown failed:', error);
		// Don't throw - teardown failures shouldn't fail the test suite
	}
}

export default globalTeardown;
