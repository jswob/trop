import { defineConfig } from 'vitest/config';

export default defineConfig({
	test: {
		name: { label: '@app/ui', color: 'yellow' },
		environment: 'jsdom',
		globals: true,
		setupFiles: ['./src/test/setup.ts'],
	},
});
