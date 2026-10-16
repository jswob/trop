import path from 'path';
import tailwindcss from '@tailwindcss/vite';
import { tanstackRouter } from '@tanstack/router-plugin/vite';
import react from '@vitejs/plugin-react';
import { defineConfig, loadEnv } from 'vite';

// https://vite.dev/config/
export default defineConfig(({ mode }) => {
	// Load env file from project root (../../)
	const env = loadEnv(mode, '../..', '');

	const apiUrl =
		process.env.VITE_API_URL ||
		env.VITE_API_URL ||
		(mode === 'production' ? undefined : `http://localhost:${env.API_PORT || '3001'}`);

	if (mode === 'production' && apiUrl && !apiUrl.startsWith('http')) {
		throw new Error(
			'VITE_API_URL must be a valid URL (e.g., https://your-api.fly.dev) or left empty for same-origin deployments. ' +
				'Set it in the Vercel project Environment Variables or pass it via process env.'
		);
	}

	return {
		plugins: [tanstackRouter({ target: 'react', autoCodeSplitting: true }), tailwindcss(), react()],
		resolve: {
			alias: {
				'@': path.resolve(import.meta.dirname, './src'),
			},
		},
		server: {
			port: parseInt(env.WEB_PORT || '5173'),
		},
		define: {
			'import.meta.env.VITE_API_URL': JSON.stringify(apiUrl ?? ''),
		},
	};
});
