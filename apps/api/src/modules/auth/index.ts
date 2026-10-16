import { timingSafeEqual } from 'node:crypto';
import { jwt } from '@elysiajs/jwt';
import { Elysia, t } from 'elysia';

import { config } from '../../config';

const JWT_EXPIRY_SECONDS = 60 * 60 * 24 * 7; // a week

/** Timing-safe password comparison: SHA-256 hashes both inputs to fixed 32 bytes, then compares. */
async function verifyPassword(input: string, expected: string): Promise<boolean> {
	const encoder = new TextEncoder();
	const [inputHash, expectedHash] = await Promise.all([
		crypto.subtle.digest('SHA-256', encoder.encode(input)),
		crypto.subtle.digest('SHA-256', encoder.encode(expected)),
	]);
	return timingSafeEqual(Buffer.from(inputHash), Buffer.from(expectedHash));
}

export const createAuthRoute = () =>
	new Elysia({ prefix: '/api/auth' })
		.use(jwt({ name: 'jwt', secret: config.jwtSecret ?? 'dev-no-auth' }))
		.post(
			'/login',
			async ({ body, jwt, status }) => {
				if (!config.authPassword || !config.jwtSecret) {
					return status(500, { error: 'Authentication not configured' });
				}

				const isValid = await verifyPassword(body.password, config.authPassword);
				if (!isValid) {
					return status(401, { error: 'Invalid password' });
				}

				const token = await jwt.sign({
					iat: true,
					exp: Math.floor(Date.now() / 1000) + JWT_EXPIRY_SECONDS,
				});

				return { token };
			},
			{ body: t.Object({ password: t.String({ minLength: 1 }) }) }
		);
