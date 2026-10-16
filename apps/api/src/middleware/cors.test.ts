import { Elysia } from 'elysia';
import { describe, expect, test } from 'vitest';
import { corsPolicy } from './cors';

describe('the CORS policy', () => {
	test('lets a browser keep a preflight answer for a day', async () => {
		const app = new Elysia().use(corsPolicy()).get('/api/probe', () => ({ ok: true }));

		const preflight = await app.handle(
			new Request('http://localhost/api/probe', {
				method: 'OPTIONS',
				headers: {
					Origin: 'http://localhost:5173',
					'Access-Control-Request-Method': 'GET',
					'Access-Control-Request-Headers': 'authorization',
				},
			})
		);

		expect(preflight.headers.get('access-control-max-age')).toBe('86400');
	});
});
