import type postgres from 'postgres';

/** A callback `begin` and `savepoint` hand a scoped client to. */
type Scoped = (scoped: postgres.Sql, ...rest: unknown[]) => unknown;

/**
 * **Make every query sent through this client a prepared statement.**
 *
 * Drizzle sends its queries with `client.unsafe(text, params)`, unprepared unless the query was
 * named with `.prepare()` (from 0.45 it says so with `prepare: false`). An unprepared query with
 * parameters costs two round trips every time — one to learn the parameter types, one to run it —
 * and nothing else can be sent on its connection meanwhile. Prepared, a statement pays that once
 * per connection; after that it is one round trip, and the statements of a `Promise.all` inside
 * one transaction go out together.
 *
 * Drizzle offers no switch for this, so the client is wrapped: `unsafe` asks for `prepare`,
 * whatever Drizzle passed, and the scoped clients `begin` and `savepoint` hand to a transaction
 * are wrapped the same way.
 * Statements are prepared at protocol level, which a PgBouncer pooler in transaction mode supports
 * from 1.22 (Neon's does); postgres.js prepares a statement again when the server has lost it.
 *
 * @param client - A postgres.js client, or the scoped client of a transaction
 */
export function withPreparedStatements<Client extends postgres.Sql>(client: Client): Client {
	return new Proxy(client, {
		get(target, key) {
			const value: unknown = Reflect.get(target, key);
			if (typeof value !== 'function') return value;

			if (key === 'unsafe') {
				return (
					query: string,
					parameters?: postgres.ParameterOrJSON<never>[],
					options?: postgres.UnsafeQueryOptions
				) => target.unsafe(query, parameters, { ...options, prepare: true });
			}

			if (key === 'begin' || key === 'savepoint') {
				const open = value as (...args: unknown[]) => unknown;
				return (...args: unknown[]) =>
					open(
						...args.map(arg =>
							typeof arg === 'function'
								? (scoped: postgres.Sql, ...rest: unknown[]) =>
										(arg as Scoped)(withPreparedStatements(scoped), ...rest)
								: arg
						)
					);
			}

			return value;
		},
	});
}
