import { Elysia } from 'elysia';
import { afterEach, beforeEach, describe, expect, test, vi } from 'vitest';
import { createAuthRoute } from '../modules/auth';
import { securityGuard } from './security';

const PASSWORD = 'correct horse';

function guardedApp() {
	return new Elysia()
		.use(securityGuard({ queryTokenPaths: /^\/api\/jobs\/[^/]+\/stream$/ }))
		.use(createAuthRoute())
		.get('/health', () => 'ok')
		.get('/api/jobs', () => 'jobs')
		.get('/api/jobs/:id/stream', () => 'stream');
}

async function signIn(app: ReturnType<typeof guardedApp>): Promise<string> {
	const response = await app.handle(
		new Request('http://localhost/api/auth/login', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ password: PASSWORD }),
		})
	);
	const { token } = (await response.json()) as { token: string };
	return token;
}

async function statusOf(app: ReturnType<typeof guardedApp>, path: string, token?: string) {
	const headers = token ? { Authorization: `Bearer ${token}` } : undefined;
	const response = await app.handle(new Request(`http://localhost${path}`, { headers }));
	return response.status;
}

describe('the security guard', () => {
	beforeEach(() => {
		vi.stubEnv('JWT_SECRET', 'test-secret');
		vi.stubEnv('AUTH_PASSWORD', PASSWORD);
	});

	afterEach(() => {
		vi.unstubAllEnvs();
	});

	test('refuses a request with no token and lets a public path through', async () => {
		const app = guardedApp();

		expect(await statusOf(app, '/api/jobs')).toBe(401);
		expect(await statusOf(app, '/health')).toBe(200);
	});

	test('takes the token from the Authorization header on any path', async () => {
		const app = guardedApp();
		const token = await signIn(app);

		expect(await statusOf(app, '/api/jobs', token)).toBe(200);
		expect(await statusOf(app, '/api/jobs/1/stream', token)).toBe(200);
		expect(await statusOf(app, '/api/jobs', 'not-a-token')).toBe(401);
	});

	test('takes the token from the query only on the paths it was given', async () => {
		const app = guardedApp();
		const token = await signIn(app);

		expect(await statusOf(app, `/api/jobs/1/stream?token=${token}`)).toBe(200);
		expect(await statusOf(app, `/api/jobs?token=${token}`)).toBe(401);
		expect(await statusOf(app, `/api/jobs?access_token=${token}`)).toBe(401);
	});

	test('lets everything through when no JWT_SECRET is set', async () => {
		vi.stubEnv('JWT_SECRET', undefined);
		const app = guardedApp();

		expect(await statusOf(app, '/api/jobs')).toBe(200);
	});
});
