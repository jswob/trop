import { sql } from 'drizzle-orm';
import { Elysia } from 'elysia';
import type { DatabaseConnection } from '../../db/types';
import { logger } from '../../utils/logger';

/**
 * The least time between two wakes of the database. The route is open to anyone, and a database
 * that scales to zero after five idle minutes is billed for the time it is awake: asked in a loop,
 * this keeps it awake a third of the time at most.
 */
const WAKE_EVERY_MS = 15 * 60_000;

/**
 * **Wake the server before it is needed.** The web asks this when it shows sign-in. The request
 * itself starts a stopped or suspended machine, and one `select 1` wakes a database that has scaled
 * to zero, so both are up by the time the password is typed and the first reads go out.
 *
 * It answers `ok` whether or not the database did: nothing is drawn from the answer, and the
 * reads that follow say what is wrong.
 *
 * @param db - Database connection
 * @param options - `everyMs`: the least time between two wakes of the database
 */
export const createWakeRoute = (db: DatabaseConnection, options: { everyMs?: number } = {}) => {
	const everyMs = options.everyMs ?? WAKE_EVERY_MS;
	let wokenAt = -Infinity;

	return new Elysia({ prefix: '/api/wake' }).get('/', async () => {
		const now = Date.now();
		if (now - wokenAt >= everyMs) {
			wokenAt = now;
			await db.execute(sql`select 1`).catch((error: unknown) => {
				logger.warn({ error }, 'Wake could not reach the database');
			});
		}
		return { status: 'ok' as const };
	});
};
