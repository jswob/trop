import path from 'node:path';

import { defineConfig } from 'vitest/config';

const double = (file: string): string =>
	path.resolve(import.meta.dirname, 'src/test/doubles', file);

/**
 * The bench's tests run without an app: each module the server would resolve to one
 * (`src/vite.ts`) resolves to a double here.
 */
export default defineConfig({
	resolve: {
		alias: {
			'virtual:workbench/setup': double('setup.tsx'),
			'virtual:workbench/host': double('host.ts'),
			'virtual:workbench/sources': double('sources.ts'),
			'virtual:workbench/spend': double('sources.ts'),
			'virtual:workbench/styles.css': double('styles.css'),
		},
	},
	test: {
		name: { label: '@jswob/design-workbench', color: 'magenta' },
		environment: 'jsdom',
		globals: true,
		setupFiles: ['./src/test/setup.ts'],
	},
});
