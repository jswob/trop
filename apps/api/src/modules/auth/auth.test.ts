import { Elysia } from 'elysia';
import { decodeJwt, SignJWT } from 'jose';
import { vi, beforeEach, describe, expect, test } from 'vitest';

import { securityGuard } from '../../middleware/security';
import { createAuthRoute } from './index';

const { TEST_PASSWORD, TEST_JWT_SECRET, mockConfig } = vi.hoisted(() => {
	const TEST_PASSWORD = 'test-password-123';
	const TEST_JWT_SECRET = 'test-jwt-secret-for-auth-tests';
	const mockConfig: { authPassword: string | undefined; jwtSecret: string | undefined } = {
		authPassword: TEST_PASSWORD,
		jwtSecret: TEST_JWT_SECRET,
	};
	return { TEST_PASSWORD, TEST_JWT_SECRET, mockConfig };
});

vi.mock('../../config', () => ({
	config: {
		get authPassword() {
			return mockConfig.authPassword;
		},
		get jwtSecret() {
			return mockConfig.jwtSecret;
		},
	},
}));

function createApp() {
	return createAuthRoute();
}

async function loginAndGetToken(app: {
	handle: (request: Request) => Promise<Response>;
}): Promise<string> {
	const res = await app.handle(
		new Request('http://localhost/api/auth/login', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ password: TEST_PASSWORD }),
		})
	);
	const data = (await res.json()) as { token: string };
	return data.token;
}

async function signToken(payload: Record<string, unknown>): Promise<string> {
	const secret = new TextEncoder().encode(TEST_JWT_SECRET);
	let builder = new SignJWT(payload).setProtectedHeader({ alg: 'HS256' });
	if (typeof payload.exp === 'number') {
		builder = builder.setExpirationTime(payload.exp);
	}
	return builder.sign(secret);
}

beforeEach(() => {
	mockConfig.authPassword = TEST_PASSWORD;
	mockConfig.jwtSecret = TEST_JWT_SECRET;
});

describe('POST /api/auth/login', () => {
	test('returns token on correct password', async () => {
		const app = createApp();
		const res = await app.handle(
			new Request('http://localhost/api/auth/login', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ password: TEST_PASSWORD }),
			})
		);

		expect(res.status).toBe(200);
		const data = (await res.json()) as { token: string };
		expect(data.token).toBeDefined();
		expect(data.token.split('.')).toHaveLength(3);
	});

	test('the token lasts a week', async () => {
		const token = await loginAndGetToken(createApp());

		const { iat, exp } = decodeJwt(token);

		// Both are whole seconds read a moment apart, so the span can come out a second short.
		const week = 7 * 24 * 60 * 60;
		expect(exp! - iat!).toBeGreaterThanOrEqual(week - 1);
		expect(exp! - iat!).toBeLessThanOrEqual(week);
	});

	test('returns 401 on wrong password', async () => {
		const app = createApp();
		const res = await app.handle(
			new Request('http://localhost/api/auth/login', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ password: 'wrong-password' }),
			})
		);

		expect(res.status).toBe(401);
	});

	test('returns 500 when AUTH_PASSWORD is not configured', async () => {
		mockConfig.authPassword = undefined;

		const app = createApp();
		const res = await app.handle(
			new Request('http://localhost/api/auth/login', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ password: 'anything' }),
			})
		);

		expect(res.status).toBe(500);
	});
});

describe('the security guard', () => {
	function guardedApp() {
		return new Elysia()
			.use(securityGuard())
			.use(createAuthRoute())
			.get('/health', () => ({ status: 'ok' }))
			.get('/api/wake', () => ({ status: 'ok' }))
			.get('/api/probe', () => ({ ok: true }));
	}

	function probe(app: ReturnType<typeof guardedApp>, token?: string) {
		return app.handle(
			new Request('http://localhost/api/probe', {
				headers: token === undefined ? {} : { Authorization: `Bearer ${token}` },
			})
		);
	}

	test('lets a valid token through', async () => {
		const app = guardedApp();
		const token = await loginAndGetToken(app);

		const res = await probe(app, token);

		expect(res.status).toBe(200);
	});

	test('refuses a request without a token', async () => {
		const res = await probe(guardedApp());

		expect(res.status).toBe(401);
	});

	test('refuses an expired token', async () => {
		const token = await signToken({
			iat: Math.floor(Date.now() / 1000) - 7200,
			exp: Math.floor(Date.now() / 1000) - 3600,
		});

		const res = await probe(guardedApp(), token);

		expect(res.status).toBe(401);
	});

	test('refuses an invalid token', async () => {
		const res = await probe(guardedApp(), 'invalid-token');

		expect(res.status).toBe(401);
	});

	test('leaves sign-in, the health check and the wake open', async () => {
		const app = guardedApp();

		const health = await app.handle(new Request('http://localhost/health'));
		const wake = await app.handle(new Request('http://localhost/api/wake'));
		const login = await app.handle(
			new Request('http://localhost/api/auth/login', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ password: TEST_PASSWORD }),
			})
		);

		expect(health.status).toBe(200);
		expect(wake.status).toBe(200);
		expect(login.status).toBe(200);
	});
});
