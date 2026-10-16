---
name: elysia-drizzle-api-best-practices
description: Elysia Drizzle ORM API best practices reference. Covers Elysia route composition plugin lifecycle validation handler error handling guard scope dependency injection type export, Drizzle schema pgTable column types relations relational queries transactions insert update delete select database connection type exports, query efficiency N+1 round trips batching bulk writes Promise.all query count query budget slow endpoint performance, development workflow TDD database snapshot seeding drizzle-fixtures typecheck format lint, testing TDD Vitest test context transaction isolation route testing createFixture composeFactory traits fixture composition drizzle-fixtures, JSDoc documentation comments.
---

# Elysia + Drizzle API

Routing hub for Elysia, Drizzle ORM, query efficiency, development workflow, and testing references. All implementation detail lives in the reference files below — this file maps your need to the right one. Nothing in these files is specific to a project: `PROJECT.md` carries that, and it is the one file here that the repo writes itself.

**When to Use:** Starting any API work involving Elysia routes, Drizzle queries, development workflow, or API testing. This file routes to the correct reference. Any change that reads or writes more than one row or table also goes through QUERIES.md before the code is written.

---

## Open the Session

Read **`PROJECT.md`** first. It says where the API lives and how it is laid out, which fixtures package and test context module the tests use, the command behind each check and each database step, which suites are too slow to run, and the project's own examples of the patterns here. Everything below refers to those by role ("the safe push", "the test context module", "the client's typecheck"), never by path or script name.

If `PROJECT.md` is missing, or describes a project that does not match the repo you are in, say so rather than guessing.

---

## Decision Tree

Find what you need below and go to the indicated reference file.

**Elysia — HTTP framework, routing, middleware:**

| I need to...                                                | Go to                             |
| ----------------------------------------------------------- | --------------------------------- |
| Define routes with method chaining or grouping              | ELYSIA.md § Route Composition     |
| Validate body, query, params, headers, or response          | VALIDATION.md                     |
| Return status codes or access context properties            | ELYSIA.md § Handler Patterns      |
| Add request/response middleware (auth, logging, transforms) | ELYSIA.md § Lifecycle Hooks       |
| Handle errors globally or per-route                         | ELYSIA.md § Error Handling        |
| Compose modular routers with `.use()`                       | ELYSIA.md § Plugins               |
| Inject services, derive context, or resolve data            | ELYSIA.md § Dependency Injection  |
| Export app type for type-safe clients                       | ELYSIA.md § Type Export Pattern   |
| Scope hooks, schemas, or guards to route groups             | ELYSIA.md § Guard and Scope       |
| Extract complex handler logic into a service layer          | ELYSIA.md § Service Layer Pattern |

**Drizzle ORM — schema, queries, database:**

| I need to...                                               | Go to                               |
| ---------------------------------------------------------- | ----------------------------------- |
| Define tables, columns, indexes, or foreign keys           | DRIZZLE.md § Schema Definition      |
| Reuse common columns (id, timestamps, soft-delete)         | DRIZZLE.md § Common Columns Pattern |
| Export inferred types from table definitions               | DRIZZLE.md § Type Exports           |
| Write SELECT queries with filters, joins, or pagination    | DRIZZLE.md § Select Queries         |
| Insert rows with returning or upsert                       | DRIZZLE.md § Insert Queries         |
| Update rows with SQL expressions                           | DRIZZLE.md § Update Queries         |
| Delete rows with filter and returning                      | DRIZZLE.md § Delete Queries         |
| Define relations and use `db.query` with eager loading     | DRIZZLE.md § Relational Queries     |
| Use transactions with rollback or nested savepoints        | DRIZZLE.md § Transactions           |
| Configure database connection, schema injection, or casing | DRIZZLE.md § Database Connection    |

**Query efficiency — round trips, batching, bulk writes:**

| I need to...                                                 | Go to                                          |
| ------------------------------------------------------------ | ---------------------------------------------- |
| Know how many queries a route or service may make            | QUERIES.md § The Rule                          |
| Read parents with their children (lists, nested records)     | QUERIES.md § Read a Tree in One Pass           |
| Run several reads without waiting on each one                | QUERIES.md § Send Independent Queries Together |
| Insert, update or delete many rows, or parents with children | QUERIES.md § Bulk Writes                       |
| Update many rows to a different value each                   | QUERIES.md § Bulk Writes (`caseBy`)            |
| Validate and write a multi-row save                          | QUERIES.md § Check First, Then Write           |
| Avoid select-backs, existence checks and repeated reads      | QUERIES.md § Do Not Ask Twice                  |
| Find out why an endpoint is slow                             | QUERIES.md § The Rule, § Checklist             |
| Pin a query count in a test                                  | TESTING.md § Query Budget Tests                |

**Development Workflow — TDD, checks, database, seeding:**

| I need to...                                         | Go to                                    |
| ---------------------------------------------------- | ---------------------------------------- |
| Build a feature end-to-end (schema to route)         | WORKFLOW.md § Feature Development Flow   |
| Follow the red-green-refactor TDD cycle              | WORKFLOW.md § TDD Flow                   |
| Run typecheck, format, test, lint in the right order | WORKFLOW.md § Running Targeted Checks    |
| Snapshot, restore, undo, or reset the database       | WORKFLOW.md § Database Snapshot Workflow |
| Seed the dev database with realistic data            | WORKFLOW.md § Seeding                    |
| Add a new database model end-to-end                  | WORKFLOW.md § New Model Checklist        |
| Document non-trivial functions with JSDoc            | WORKFLOW.md § JSDoc Documentation        |

**Testing — TDD, fixtures, route testing:**

| I need to...                                          | Go to                                     |
| ----------------------------------------------------- | ----------------------------------------- |
| Follow the TDD workflow for API endpoints             | TESTING.md § TDD Workflow                 |
| Configure Vitest for API testing                      | TESTING.md § Vitest Configuration         |
| Set up test context with db, tx, and factory fixtures | TESTING.md § Test Context Pattern         |
| Understand transaction-based test isolation           | TESTING.md § Transaction Isolation        |
| Test Elysia routes without an HTTP server             | TESTING.md § Elysia Route Testing         |
| Write full integration tests (routes + database)      | TESTING.md § Route + Database Integration |
| Test service functions directly without HTTP          | TESTING.md § Service Function Testing     |
| Define reusable fixtures with `createFixture`         | TESTING.md § createFixture API            |
| Compose fixtures into a factory with `composeFactory` | TESTING.md § composeFactory + Builder     |
| Add named variations with traits and augmentations    | TESTING.md § Traits                       |
| Create related records via fixture composition        | TESTING.md § Fixture Composition          |

---

## Hard Rules

Non-negotiable conventions. Violations break tests, DX, or CI.

1. **ALWAYS** use the project's fixtures package for test data (`createFixture` / `composeFactory`; `PROJECT.md` names it). Use `@faker-js/faker` for field resolvers. Create a fixture file per new table.
2. **ALWAYS** push schema changes with the safe push, which snapshots the database first, never a plain push. `PROJECT.md` names both.
3. **NEVER** generate migration files — only the user creates them. `PROJECT.md` names the commands to leave alone.
4. **ALWAYS** import `test`, `expect`, `describe` from the project's test context module in integration tests (`PROJECT.md` gives its path), never from `vitest` directly.
5. **ALWAYS** use package exports (`@<scope>/<package>/<subpath>`), never internal cross-package paths.
6. **NEVER** run a query inside a loop, a `.map()`, or a helper called once per row. The number of statements a request sends must not grow with the number of rows it handles: every statement is a network round trip to the database, and that is what a slow endpoint is made of. Read each table once, write each table once per kind of change, and pin the count with a query budget test. See QUERIES.md.

---

## Pattern Index

Alphabetical index of key patterns and APIs across all reference files.

| Pattern / API                              | Reference                                      |
| ------------------------------------------ | ---------------------------------------------- |
| `alias(table, name)` (same table twice)    | QUERIES.md § Read a Tree in One Pass           |
| `app.handle(new Request(...))`             | TESTING.md § Elysia Route Testing              |
| Baseline snapshot (`PROJECT.md`)           | WORKFLOW.md § Database Snapshot Workflow       |
| `caseBy(keyColumn, valueColumn, pairs)`    | QUERIES.md § Bulk Writes                       |
| `composeFactory(fixtures)`                 | TESTING.md § composeFactory + Builder          |
| `countQueries(run)`                        | TESTING.md § Query Budget Tests                |
| `createFixture({ table, fields, traits })` | TESTING.md § createFixture API                 |
| `db.query.table.findMany({ with })`        | DRIZZLE.md § Relational Queries                |
| `db.transaction(async (tx) => ...)`        | DRIZZLE.md § Transactions                      |
| `decorate()` / `state()`                   | ELYSIA.md § Dependency Injection               |
| `derive()` / `resolve()`                   | ELYSIA.md § Dependency Injection               |
| `export type App = typeof app`             | ELYSIA.md § Type Export Pattern                |
| `fail()`                                   | ELYSIA.md § Service Layer Pattern              |
| Fixtures package (`PROJECT.md`)            | TESTING.md § createFixture API                 |
| `getPostsWithDetails(db, scope)`           | QUERIES.md § Read a Tree in One Pass           |
| `guard({ schema, hooks }, app)`            | ELYSIA.md § Guard and Scope                    |
| `inArray(column, subquery)`                | QUERIES.md § Read a Tree in One Pass           |
| JSDoc conventions (service, helper, util)  | WORKFLOW.md § JSDoc Documentation              |
| `mapError()`                               | ELYSIA.md § Service Layer Pattern              |
| `new Elysia({ prefix })`                   | ELYSIA.md § Route Composition                  |
| `ok()`                                     | ELYSIA.md § Service Layer Pattern              |
| `onConflictDoUpdate({ target, set })`      | DRIZZLE.md § Insert Queries                    |
| `onError(({ code, error }) => ...)`        | ELYSIA.md § Error Handling                     |
| `pgTable(name, columns)`                   | DRIZZLE.md § Schema Definition                 |
| `Promise.all([...queries])`                | QUERIES.md § Send Independent Queries Together |
| `randomUUID()` (parents with children)     | QUERIES.md § Bulk Writes                       |
| `relations(table, callback)`               | DRIZZLE.md § Relational Queries                |
| `.returning()`                             | DRIZZLE.md § Insert Queries                    |
| Safe push (`PROJECT.md`)                   | WORKFLOW.md § Database Snapshot Workflow       |
| `ServiceResult<T>`                         | ELYSIA.md § Service Layer Pattern              |
| `t.Object({...})` (TypeBox)                | VALIDATION.md § TypeBox Schema API             |
| `table.$inferSelect` / `$inferInsert`      | DRIZZLE.md § Type Exports                      |
| `test.extend<T>()`                         | TESTING.md § Test Context Pattern              |
| Undo the last push (`PROJECT.md`)          | WORKFLOW.md § Database Snapshot Workflow       |
| `use()` (fixture composition)              | TESTING.md § Fixture Composition               |

---

## Common Pitfalls

1. **Internal path imports** — Always use package exports (the API's type export, which `PROJECT.md` names), never relative paths to source files across packages. See ELYSIA.md § Type Export Pattern.

2. **Missing `.returning()` on inserts/updates** — PostgreSQL returns nothing without `.returning()`. Always chain it to get auto-generated fields like `id` and `createdAt`. See DRIZZLE.md § Insert Queries.

3. **Missing schema barrel re-export** — New tables must be added to the schema barrel (the schema folder's `index.ts`), or `db.query` and the API's type export won't see them; drizzle-kit misses them too when its `schema` setting points at the barrel rather than a glob. See DRIZZLE.md § Type Exports.

4. **`TransactionRollbackError` not caught in test context** — The test `tx` fixture catches this error automatically. If you manually call `tx.rollback()` outside the fixture's try/catch, it will throw unhandled. See TESTING.md § Transaction Isolation.

5. **Factory bound to `db` not `tx`** — Fixture factories must receive the test transaction, not the global `db` connection. Data created with `db` persists after test rollback. See TESTING.md § Test Context Pattern.

6. **Hard-importing `db` in routes prevents DI** — Routes that import `db` directly from a module cannot be transaction-isolated in tests. Accept `db` via `decorate()` or a function parameter. See TESTING.md § Route + Database Integration and ELYSIA.md § Dependency Injection.

7. **`db.query` fails without `{ schema }` injection** — Relational queries (`db.query.authors.findMany`) do not type-check, and throw at runtime (`db.query.authors` is undefined), if `schema` isn't passed to `drizzle()` or the table is missing from it. See DRIZZLE.md § Database Connection and § Relational Queries.

8. **Test files not collocated** — Test files must be next to their source files (`users.ts` -> `users.test.ts`). Never use `__tests__` folders. See TESTING.md § TDD Workflow.

9. **Missing CORS plugin** — Cross-origin requests from the frontend will fail without the CORS plugin. Add `cors()` to the main Elysia app. See ELYSIA.md § Plugins.

10. **Missing fixture for new table** — Every new database table MUST have a corresponding fixture file using `createFixture` from the project's fixtures package. See WORKFLOW.md § New Model Checklist.

11. **Using a plain push** — Always use the safe push, which auto-snapshots before pushing. A plain push risks data loss. See WORKFLOW.md § Database Snapshot Workflow.

12. **Generating migrations** — Never run the migration generators `PROJECT.md` names. Migration files are production artifacts created intentionally by the user only. See WORKFLOW.md § Database Snapshot Workflow.

13. **Business logic in oversized handlers** — Route handlers exceeding ~30 lines or with multi-phase logic (fetch-then-validate-then-write) should extract business logic into `service.ts` as plain async functions returning `ServiceResult<T>`. Keep route handlers as thin HTTP adapters. See ELYSIA.md § Service Layer Pattern.

14. **Missing JSDoc on service/helper functions** — Non-trivial functions in `service.ts`, helper files, `validation.ts`, and `utils/` must have JSDoc with at minimum a purpose sentence and `@param` tags. Route handlers and schema definitions do not get JSDoc. See WORKFLOW.md § JSDoc Documentation.

15. **A query inside a loop (N+1)** — One query per row, awaited in a `for` or fanned out with `Promise.all(rows.map(...))`, makes the request slower with every row added; this is how a list endpoint ends up taking seconds. Fetch the whole set with one query per table and group it in memory. See QUERIES.md § The Rule and § Read a Tree in One Pass.

16. **One write per row** — An `insert`, `update` or `delete` in a loop. Use a multi-row insert, `inArray` in the `where`, or `caseBy` for per-row values. See QUERIES.md § Bulk Writes.

17. **Reads waiting on each other for no reason** — Sequential `await`s on queries that need nothing from each other, or a chain where each read is filtered by ids the previous one returned. Scope by the parent's condition and send them in one `Promise.all`. See QUERIES.md § Send Independent Queries Together.

18. **Asking twice** — A select-back after a write (use `.returning()`), an existence read before an update or delete (check what `.returning()` gives), or every part of a save re-reading the same parent. See QUERIES.md § Do Not Ask Twice.

19. **Single-row data helpers** — A helper shaped `(db, id)` that queries gets called in a loop sooner or later. Write helpers that take a scope or a list. See QUERIES.md § The Rule.

20. **A named type in a route's return** — Rows typed by a named type from a module file pass the API's typecheck and break the client app (an Eden client) with TS2742, because the client cannot name a type no package export reaches. Write the return type out in full and run the client's typecheck (`PROJECT.md` names it) after changing what a route returns. See ELYSIA.md § Type Export Pattern.

---

## Reference Files

| File          | Lines | Focus                                                                        |
| ------------- | ----- | ---------------------------------------------------------------------------- |
| ELYSIA.md     | ~500  | Elysia routes, lifecycle, error handling, plugins, DI, guards, service layer |
| VALIDATION.md | ~140  | TypeBox schemas, validation targets, layers, placement rules                 |
| DRIZZLE.md    | ~495  | Drizzle schema, CRUD queries, relations, transactions, connection            |
| QUERIES.md    | ~320  | Query count per request, one-pass reads, parallel reads, bulk writes         |
| WORKFLOW.md   | ~340  | TDD flow, targeted checks, DB snapshots, seeding, new model checklist, JSDoc |
| TESTING.md    | ~500  | TDD workflow, test context, fixtures, traits, route testing                  |

---

## External Documentation

**Elysia:**

- [Route Composition](https://elysiajs.com/essential/route)
- [Validation](https://elysiajs.com/essential/validation)
- [Lifecycle](https://elysiajs.com/essential/life-cycle)
- [Error Handling](https://elysiajs.com/patterns/error-handling)
- [Plugins](https://elysiajs.com/essential/plugin)
- [Dependency Injection](https://elysiajs.com/patterns/extends-context)
- [Best Practices](https://elysiajs.com/essential/best-practice)

**Drizzle ORM:**

- [Schema Declaration](https://orm.drizzle.team/docs/sql-schema-declaration)
- [Column Types (PG)](https://orm.drizzle.team/docs/column-types/pg)
- [Select Queries](https://orm.drizzle.team/docs/select)
- [Insert Queries](https://orm.drizzle.team/docs/insert)
- [Relations](https://orm.drizzle.team/docs/relations)
- [Transactions](https://orm.drizzle.team/docs/transactions)

**Testing:**

- [Vitest Test Context](https://vitest.dev/guide/test-context)
- [Vitest Config Reference](https://vitest.dev/config/)
- [drizzle-fixtures](https://github.com/jswob/drizzle-fixtures)
