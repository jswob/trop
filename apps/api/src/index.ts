import { wrap } from '@bogeychan/elysia-logger';
import { Elysia } from 'elysia';
import { config } from './config';
import { db } from './db/connection';
import { corsPolicy } from './middleware/cors';
import { securityGuard } from './middleware/security';
import { createAuthRoute } from './modules/auth';
import { createWakeRoute } from './modules/wake';
import { queryTokenPaths, routes, startBackground } from './routes';
import { logger } from './utils/logger';

const port = config.apiPort;

const app = new Elysia()
	.use(wrap(logger))
	.use(corsPolicy())
	.use(securityGuard({ queryTokenPaths }))
	.use(createAuthRoute())
	.use(createWakeRoute(db))
	.use(routes(db))
	.get('/health', () => ({ status: 'ok', timestamp: Date.now() }));

export type App = typeof app;

app.listen(port);
logger.info(`🦊 Elysia is running at http://localhost:${port}`);
if (process.env.NODE_ENV !== 'test') {
	startBackground(db);
}
