import { fileURLToPath } from 'node:url';

import react from '@vitejs/plugin-react';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	plugins: [react()],
	// One React and one router: an engine linked to its checkout (`toolkit link`) brings its own.
	resolve: { dedupe: ['react', 'react-dom', '@tanstack/react-router'] },
	test: {
		name: { label: '@app/workbench-app', color: 'magenta' },
		environment: 'jsdom',
		globals: true,
		setupFiles: [fileURLToPath(import.meta.resolve('@app/ui/test/setup'))],
		exclude: ['**/node_modules/**', 'probe/out/**'],
	},
});
