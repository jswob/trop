/**
 * The app's half of the API. `index.ts` is the shell every app of this shape shares (logging, CORS,
 * the security guard, sign-in, wake, `/health`); this file is what is mounted inside it.
 */

import { Elysia } from 'elysia';
import type { DatabaseConnection } from './db/types';
import { createNotesRoute } from './modules/notes';

/** The app's routes. */
export const routes = (db: DatabaseConnection) => new Elysia().use(createNotesRoute(db));

/**
 * Paths whose reader cannot send headers, so the security guard takes the token from `?token=`
 * there: an `EventSource` reading a stream, say. None yet.
 */
export const queryTokenPaths: RegExp | undefined = undefined;

/**
 * Work the app does beside answering requests, started once the server listens: recovering jobs a
 * stopped process left running, a sweeper that waits to be woken. None yet.
 */
export function startBackground(_db: DatabaseConnection): void {}
