import { wrap } from '@bogeychan/elysia-logger';
import { Elysia } from 'elysia';
import { vi, describe, expect, test } from 'vitest';

import { securityGuard } from '../middleware/security';
import { createAuthRoute } from '../modules/auth';
import { createLogger } from './logger';

const { PASSWORD, JWT_SECRET } = vi.hoisted(() => ({
	PASSWORD: 'sign-in-password-in-body',
	JWT_SECRET: 'jwt-secret-for-logger-tests',
}));

vi.mock('../config', () => ({
	config: { authPassword: PASSWORD, jwtSecret: JWT_SECRET },
}));

/** The API's request logging and auth, writing every log line into `lines`. */
function createApp() {
	const lines: string[] = [];
	const logger = createLogger({ write: (line: string) => void lines.push(line) });
	const app = new Elysia()
		.use(wrap(logger))
		.use(securityGuard({ queryTokenPaths: /^\/api\/things$/ }))
		.use(createAuthRoute())
		.get('/api/things', () => ['a thing']);
	return { app, lines, logger };
}

function signIn(
	app: { handle: (request: Request) => Promise<Response> },
	body: unknown
): Promise<Response> {
	return app.handle(
		new Request('http://localhost/api/auth/login', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(body),
		})
	);
}

/** Response logging runs after the response is sent; give it a turn to write. */
async function settled(lines: string[], count: number): Promise<string[]> {
	for (let attempt = 0; attempt < 50 && lines.length < count; attempt++) {
		await new Promise(resolve => setTimeout(resolve, 5));
	}
	return lines;
}

describe('request logging', () => {
	test('a sign-in logs neither the password nor the token it returns', async () => {
		const { app, lines } = createApp();

		const response = await signIn(app, { password: PASSWORD });
		const { token } = (await response.json()) as { token: string };

		const logged = (await settled(lines, 1)).join('');
		expect(response.status).toBe(200);
		expect(logged).toContain('/api/auth/login');
		expect(logged).not.toContain(PASSWORD);
		expect(logged).not.toContain(token);
	});

	test('a sign-in the schema rejects does not log the value its error quotes', async () => {
		const { app, lines } = createApp();

		const response = await signIn(app, { password: [PASSWORD] });

		const logged = (await settled(lines, 1)).join('');
		expect(response.status).toBe(422);
		expect(logged).toContain('VALIDATION');
		expect(logged).not.toContain(PASSWORD);
	});

	test('a request logs neither its Authorization header nor a token in its query', async () => {
		const { app, lines } = createApp();
		const { token } = (await (await signIn(app, { password: PASSWORD })).json()) as {
			token: string;
		};

		const byHeader = await app.handle(
			new Request('http://localhost/api/things', { headers: { Authorization: `Bearer ${token}` } })
		);
		const byQuery = await app.handle(
			new Request(`http://localhost/api/things?token=${token}&page=2`)
		);

		const logged = (await settled(lines, 3)).join('');
		expect([byHeader.status, byQuery.status]).toEqual([200, 200]);
		expect(logged).not.toContain(token);
		expect(logged).toContain('/api/things?token=redacted&page=2');
	});

	test('a request line holds the method, path, status and time taken', async () => {
		const { app, lines } = createApp();
		const { token } = (await (await signIn(app, { password: PASSWORD })).json()) as {
			token: string;
		};

		await app.handle(
			new Request('http://localhost/api/things', { headers: { Authorization: `Bearer ${token}` } })
		);

		const line = JSON.parse((await settled(lines, 2))[1]!) as Record<string, unknown>;
		expect(line).toMatchObject({ method: 'GET', path: '/api/things', status: 200 });
		expect(typeof line.responseTime).toBe('number');
		expect(Object.keys(line).sort()).toEqual(
			['hostname', 'level', 'method', 'path', 'pid', 'responseTime', 'status', 'time'].sort()
		);
	});

	test('credentials a caller logs by hand are blanked', async () => {
		const { lines, logger } = createApp();

		logger.info({
			password: PASSWORD,
			body: { token: 'a-token' },
			headers: { authorization: 'Bearer x' },
		});

		const logged = (await settled(lines, 1)).join('');
		expect(logged).not.toContain(PASSWORD);
		expect(logged).not.toContain('a-token');
		expect(logged).not.toContain('Bearer x');
	});
});
