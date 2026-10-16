import pino, { type DestinationStream, type LoggerOptions } from 'pino';

const isDevelopment = process.env.NODE_ENV !== 'production';

/** Query parameters that carry a credential: the security guard reads a JWT from `?token=`. */
const SECRET_QUERY_PARAMS = ['token'];

/**
 * Keys blanked in anything logged by hand, at the top level and one level down. The request line
 * never carries them; this catches an object a caller passes itself.
 */
const REDACTED_KEYS = ['password', 'token', 'authorization'];

const CENSOR = 'redacted';

/** The parts of Elysia's request context that `@bogeychan/elysia-logger` hands to the logger. */
interface RequestContext {
	request: Request;
	set: { status?: number | string };
	store: { responseTime?: number };
	isError: boolean;
	code?: unknown;
	error?: unknown;
}

function isRequestContext(
	object: Record<string, unknown>
): object is Record<string, unknown> & RequestContext {
	return object.request instanceof Request && typeof object.isError === 'boolean';
}

/** The request's path and query, with every credential in the query blanked. */
function loggedPath(request: Request): string {
	const url = new URL(request.url);
	for (const param of SECRET_QUERY_PARAMS) {
		if (url.searchParams.has(param)) url.searchParams.set(param, CENSOR);
	}
	return url.pathname + url.search;
}

/**
 * What a failed request's line says about the error. A validation error's message quotes the
 * value it rejected, which for a sign-in is the password, so only the part it rejected is kept.
 */
function describeError(code: unknown, error: unknown): Record<string, unknown> {
	if (typeof error !== 'object' || error === null) return { code };
	const { status, message, type } = error as {
		status?: unknown;
		message?: unknown;
		type?: unknown;
	};
	if (code === 'VALIDATION') return { code, status, on: type };
	return { code, status, message };
}

/**
 * A request's log line: the method, the path, the status and the time taken, or the error. Never
 * its headers, its body or its response — they hold the password, the JWT and the user's data.
 */
export function describeRequest(ctx: RequestContext): Record<string, unknown> {
	const line = {
		method: ctx.request.method,
		path: loggedPath(ctx.request),
		status: ctx.set.status,
	};
	if (ctx.isError) return { ...line, ...describeError(ctx.code, ctx.error) };
	return { ...line, responseTime: ctx.store.responseTime };
}

/**
 * The API's logger. `wrap(logger)` from `@bogeychan/elysia-logger` logs Elysia's whole request
 * context for every request; the `log` formatter replaces it with `describeRequest`.
 * @param destination - Where lines go; a test passes its own. Left out, they go to stdout
 *   (pretty-printed outside production).
 */
export function createLogger(destination?: DestinationStream) {
	const options: LoggerOptions = {
		level: isDevelopment ? 'debug' : 'info',
		redact: {
			paths: REDACTED_KEYS.flatMap(key => [key, `*.${key}`]),
			censor: CENSOR,
		},
		formatters: {
			log: object => (isRequestContext(object) ? describeRequest(object) : object),
		},
	};
	if (destination) return pino(options, destination);

	return pino({
		...options,
		transport: isDevelopment
			? {
					target: 'pino-pretty',
					options: {
						colorize: true,
						translateTime: 'SYS:standard',
						ignore: 'pid,hostname',
					},
				}
			: undefined,
	});
}

export const logger = createLogger();
