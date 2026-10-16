# @jswob/drizzle-utils

Runtime helpers for Drizzle ORM on postgres.js: database targets, the SSL choice, prepared
statements, a many-row `CASE`, PostgreSQL error codes and a query counter for tests.

```bash
bun add @jswob/drizzle-utils drizzle-orm postgres
```

It ships TypeScript source: import it from Bun, Vite or Vitest.

## Exports

| Export                                                          | What it is                                                                        |
| --------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| `createDbTargets(defaultNames)`                                 | `dbTarget` and `selectedDbTarget` for the `app`, `test` and `e2e` databases       |
| `databaseSsl(url)`                                              | `require` or `false`: `DATABASE_SSL` when set, else the URL's `sslmode=require`   |
| `withPreparedStatements(client)`                                | Wraps a postgres.js client so every query Drizzle sends is a prepared statement   |
| `caseBy(key, column, pairs)`                                    | A `CASE` giving each row its own value, for a many-row update in one statement    |
| `isPgError(error, code)`                                        | Whether a thrown error carries a SQLSTATE code, on itself or on Drizzle's `cause` |
| `isUniqueViolation`, `isCheckViolation`, `isExclusionViolation` | `isPgError` for `23505`, `23514`, `23P01`                                         |
| `dbNow()`                                                       | `now()`, for a timestamp written on update by the database's clock                |
| `createQueryCounter()`                                          | `debug` for a postgres.js client and `countQueries(run)`, for query-budget tests  |

## Database targets

```ts
import { createDbTargets } from '@jswob/drizzle-utils';

export const { dbTarget, selectedDbTarget } = createDbTargets({
	app: 'app_db',
	test: 'app_test_db',
	e2e: 'app_e2e_db',
});

const { url } = dbTarget('test');
```

It reads `DB_HOST`, `DB_USER`, `DB_PASSWORD`, then `DB_NAME`/`DB_PORT`, `TEST_DB_NAME`/`TEST_DB_PORT`
and `E2E_DB_NAME`/`E2E_DB_PORT` (ports 5432, 5433 and 5434 when unset). `app` is `DATABASE_URL`
when it is set; `test` and `e2e` never are, so a test can never reach production.

## License

MIT
