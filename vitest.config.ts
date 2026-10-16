import tsconfigPaths from 'vite-tsconfig-paths';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	plugins: [tsconfigPaths()],
	test: {
		globals: true,
		projects: [
			'apps/*',
			'packages/*',
			// template:start
			'scripts',
			'template',
			// template:end
		],
		exclude: ['**/node_modules/**', '**/e2e/**'],
	},
});
