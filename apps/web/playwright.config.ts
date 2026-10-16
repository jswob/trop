import { defineConfig, devices } from '@playwright/test';

const isCI = Boolean(process.env.CI);

// Use dedicated port for E2E tests to avoid conflicts with dev server
const E2E_WEB_PORT = 5174;
const API_PORT = Number(process.env.API_PORT) || 3001;

export default defineConfig({
	testDir: './e2e/tests',
	fullyParallel: false,
	forbidOnly: isCI,
	retries: isCI ? 2 : 0,
	workers: 1, // Single worker to prevent DB conflicts
	reporter: isCI ? 'github' : 'html',
	timeout: 30_000,
	expect: {
		timeout: 5_000,
	},
	use: {
		baseURL: `http://localhost:${E2E_WEB_PORT}`,
		trace: 'on-first-retry',
		screenshot: 'only-on-failure',
	},
	globalSetup: './e2e/global-setup.ts',
	globalTeardown: './e2e/global-teardown.ts',
	projects: [
		{
			name: 'chromium',
			use: { ...devices['Desktop Chrome'] },
		},
	],
	webServer: [
		{
			// API server against the e2e database (`DB_TARGET=e2e`), with the root .env for auth
			command: `NODE_ENV=test DB_TARGET=e2e API_PORT=${API_PORT} bun --cwd ../api --env-file=../../.env --hot src/index.ts`,
			url: `http://localhost:${API_PORT}/health`,
			reuseExistingServer: !isCI,
			timeout: 120_000,
			stdout: 'pipe',
			stderr: 'pipe',
		},
		{
			// Web dev server on dedicated E2E port
			command: `NODE_ENV=test WEB_PORT=${E2E_WEB_PORT} bun dev`,
			url: `http://localhost:${E2E_WEB_PORT}`,
			reuseExistingServer: !isCI,
			timeout: 120_000,
			stdout: 'pipe',
			stderr: 'pipe',
		},
	],
});
