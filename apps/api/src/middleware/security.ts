import { jwt } from '@elysiajs/jwt';
import { Elysia } from 'elysia';

import { config } from '../config';

const PUBLIC_PATHS = new Set(['/health', '/api/wake', '/api/auth/login']);

export interface SecurityGuardOptions {
	/**
	 * Paths whose reader cannot send headers (an `EventSource`), where the token may come as
	 * `?token=` instead. Everywhere else only the `Authorization` header counts, since a token in a
	 * URL ends up in logs and browser history.
	 */
	queryTokenPaths?: RegExp;
}

/** The token after `Bearer ` in an `Authorization` header. */
function bearerToken(authorization: string | undefined): string | undefined {
	return authorization?.startsWith('Bearer ') ? authorization.slice('Bearer '.length) : undefined;
}

/** Elysia plugin that enforces JWT authentication. No-op when JWT_SECRET env var is unset. */
export function securityGuard({ queryTokenPaths }: SecurityGuardOptions = {}) {
	return new Elysia({ name: 'security-guard' })
		.use(jwt({ name: 'jwt', secret: config.jwtSecret ?? 'dev-no-auth' }))
		.onBeforeHandle({ as: 'global' }, async ({ headers, query, jwt, status, path }) => {
			if (PUBLIC_PATHS.has(path)) return;
			if (!config.jwtSecret) return;
			const token =
				bearerToken(headers.authorization) ??
				(queryTokenPaths?.test(path) ? query.token : undefined);
			if (!token) return status(401, { error: 'Unauthorized' });
			const payload = await jwt.verify(token);
			if (!payload) return status(401, { error: 'Unauthorized' });
		});
}
