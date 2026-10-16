import { execSync } from 'child_process';
import path from 'path';
import { fileURLToPath } from 'url';

// ESM-compatible directory path
const _currentFile = fileURLToPath(import.meta.url);
const _currentDir = path.dirname(_currentFile);

/**
 * Global setup function that runs once before all tests.
 * Prepares the E2E database by running e2e:setup which creates a baseline snapshot.
 */
function globalSetup(): void {
	// eslint-disable-next-line no-console -- Reason: Intentional logging for E2E setup visibility
	console.log('\n🔧 Running E2E global setup...');

	// Path to the API directory
	const apiDir = path.resolve(_currentDir, '../../api');

	try {
		// Run e2e:setup to prepare the E2E database and create baseline snapshot
		// eslint-disable-next-line no-console -- Reason: Intentional logging for E2E setup visibility
		console.log('📦 Setting up E2E database...');
		execSync('bun e2e:setup', {
			cwd: apiDir,
			env: {
				...process.env,
				NODE_ENV: 'test',
			},
			stdio: 'inherit',
		});

		// eslint-disable-next-line no-console -- Reason: Intentional logging for E2E setup visibility
		console.log('✅ E2E global setup complete\n');
	} catch (error) {
		console.error('❌ E2E global setup failed:', error);
		throw error;
	}
}

export default globalSetup;
