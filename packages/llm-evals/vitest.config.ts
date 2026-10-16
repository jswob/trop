import tsconfigPaths from 'vite-tsconfig-paths';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	plugins: [tsconfigPaths()],
	test: {
		environment: 'node',
		// Evals never reach the network from a test: model calls are stubbed per test (lib/fake-models.ts).
		unstubGlobals: true,
		unstubEnvs: true,
	},
});
