import { z } from 'zod';

const envSchema = z.object({
	VITE_API_URL: z.string().default(''),
});

const _env = envSchema.safeParse(import.meta.env);

if (!_env.success) {
	console.error('Invalid environment variables:', z.flattenError(_env.error).fieldErrors);
	throw new Error('Invalid environment variables');
}

if (import.meta.env.PROD && !_env.data.VITE_API_URL.startsWith('http')) {
	// eslint-disable-next-line no-console -- Reason: Informational message about environment configuration
	console.info('VITE_API_URL not set — using same-origin mode (window.location.origin)');
}

export const ENV = _env.data;

/** Returns the base URL for API requests, falling back to same-origin when VITE_API_URL is not set. */
export function getApiBaseUrl(): string {
	return ENV.VITE_API_URL.startsWith('http') ? ENV.VITE_API_URL : window.location.origin;
}
