# Drizzle Utils

Runtime helpers for Drizzle on postgres.js. The API imports them, so they ship in its production
image. Nothing here knows the app: no table, column or database name of its own.

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

## Notes

- `createDbTargets` reads `DB_HOST`, `DB_USER`, `DB_PASSWORD`, then `DB_NAME`/`DB_PORT`,
  `TEST_DB_NAME`/`TEST_DB_PORT` and `E2E_DB_NAME`/`E2E_DB_PORT` (ports 5432, 5433, 5434 when
  unset). `app` is `DATABASE_URL` when it is set; `test` and `e2e` never are.
- `prepared.test.ts` needs the test database (`TEST_DB_*` from the repo's `.env`, or the
  `postgres` database on port 5433). Its tables are temporary, so it leaves nothing behind.
- Run: `bun --cwd packages/drizzle-utils test --run`, `bun --cwd packages/drizzle-utils typecheck`.
