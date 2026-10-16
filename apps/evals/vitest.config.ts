import { defineConfig } from 'vitest/config';

export default defineConfig({
	test: {
		environment: 'node',
		// A task's tests never reach the network: model calls are faked per test (`fakeModels`).
		unstubGlobals: true,
		unstubEnvs: true,
		// No task is measured yet.
		passWithNoTests: true,
	},
});
