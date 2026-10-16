import { execSync } from 'child_process';
import path from 'path';
import { fileURLToPath } from 'url';
import { test as base, expect } from '@playwright/test';

// ESM-compatible directory path
const _currentFile = fileURLToPath(import.meta.url);
const _currentDir = path.dirname(_currentFile);

/**
 * Custom fixture type for E2E tests
 */
interface TestFixtures {
	restoreDatabase: void;
}

/**
 * Extended test with automatic database restoration.
 * Before each test, the database is restored from the baseline snapshot
 * to ensure test isolation without HTTP API overhead.
 */
export const test = base.extend<TestFixtures>({
	restoreDatabase: [
		// eslint-disable-next-line no-empty-pattern -- Reason: Playwright fixture signature requires destructuring even when not using base fixtures
		async ({}, use) => {
			// Restore database from baseline snapshot before each test
			const apiDir = path.resolve(_currentDir, '../../../api');

			execSync('bun e2e:restore', {
				cwd: apiDir,
				stdio: 'pipe',
			});

			// Small delay to ensure PostgreSQL is fully ready after restart
			await new Promise(resolve => setTimeout(resolve, 500));

			await use();
		},
		{ auto: true },
	],
});

export { expect };
