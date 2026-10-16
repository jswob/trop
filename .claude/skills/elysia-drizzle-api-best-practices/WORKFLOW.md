# Development Workflow - Reference

Reference for feature development flow, TDD, running checks, database snapshots, seeding, and new model checklists.

**When to Use:** Starting a new feature, adding a database model, running checks, understanding the TDD cycle, managing database state during development.

**Commands:** every check and database step below is named by its role ("the safe push", "the API typecheck"). `PROJECT.md` maps each role to the project's command and gives the folder layout.

## Table of Contents

- [Feature Development Flow](#feature-development-flow)
- [TDD Flow](#tdd-flow)
- [Running Targeted Checks](#running-targeted-checks)
- [Database Snapshot Workflow](#database-snapshot-workflow)
- [Seeding](#seeding)
- [New Model Checklist](#new-model-checklist)
- [JSDoc Documentation](#jsdoc-documentation)

---

## Feature Development Flow

Step-by-step sequence for building a complete API feature end-to-end:

1. **Schema** — Create/update the table's Drizzle schema file, one file per table in the schema folder
2. **Barrel export** — Re-export table + types from the schema barrel (the schema folder's `index.ts`)
3. **Push** — Run the safe push (auto-snapshots before pushing)
4. **Fixture** — Create a fixture file for the table in the fixtures folder, using `createFixture` from the project's fixtures package with `@faker-js/faker` for field resolvers
5. **Register** — Add the fixture to the composed factory via `composeFactory`
6. **Red** — Write failing tests (see [TDD Flow](#tdd-flow))
7. **Green** — Implement route handler to make tests pass. Shape the database work so the number of queries does not grow with the data: no query in a loop, independent reads together, one write per table and kind of change (see QUERIES.md), and pin it with a query budget test (TESTING.md § Query Budget Tests). If the handler exceeds ~30 lines or has multi-phase logic (fetch-then-validate-then-write), extract business logic into `service.ts` as a plain async function returning `ServiceResult<T>` (see ELYSIA.md § Service Layer Pattern)
8. **Refactor** — Clean up while tests stay green. Add JSDoc to non-trivial service, helper, validation, and utility functions (see [JSDoc Documentation](#jsdoc-documentation))
9. **Check** — Run targeted checks in strict order (see [Running Targeted Checks](#running-targeted-checks))
10. **Seed** — Add the table to the project's seed if the dev database needs its data (see [Seeding](#seeding))

**Recommended folder structure for feature modules:**

```
src/modules/[feature]/
  ├── index.ts           (Elysia route definitions — HTTP layer)
  ├── service.ts         (business logic — ONLY when handler exceeds ~30 lines or has multi-phase logic)
  ├── validation.ts      (pure validation functions — no DB access)
  └── [feature].test.ts  (collocated integration tests)
```

> `service.ts` is created only when a handler exceeds ~30 lines of non-trivial logic, OR when logic is shared across modules. Simple CRUD stays inline in `index.ts`. See ELYSIA.md § Service Layer Pattern.

Compose into the main app via `.use()`:

```typescript
import { featureRoutes } from './modules/feature';
const app = new Elysia().use(featureRoutes);
```

---

## TDD Flow

**Two-level testing model:**

- **Unit tests** — Services, utils, pure logic. No database. Standard vitest imports (`import { test, expect } from 'vitest'`).
- **Integration tests** — Routes + DB via transaction isolation. Import from the test context module. Use `factory` for data, `tx` for queries, `app.handle()` for HTTP simulation.

**Red-Green-Refactor cycle:**

### 1. Red — Write a failing test

```typescript
import { test, expect, describe } from 'test-context'; // the project's test context (PROJECT.md)

describe('POST /api/items', () => {
	test('creates an item and returns 201', async ({ factory, tx }) => {
		const res = await createApp(tx).handle(
			new Request('http://localhost/api/items', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ name: 'Test Item' }),
			})
		);
		expect(res.status).toBe(201);
		const data = await res.json();
		expect(data).toMatchObject({ name: 'Test Item' });
	});
});
```

### 2. Green — Implement the minimum to pass

```typescript
// src/modules/items/index.ts
const itemRoutes = new Elysia({ prefix: '/api/items' }).post(
	'/',
	async ({ body, db }) => {
		const [item] = await db.insert(items).values(body).returning();
		return item;
	},
	{ body: t.Object({ name: t.String() }) }
);
```

### 3. Refactor — Improve while tests stay green

Extract service logic, add validation details, improve error handling — re-run tests after each change.

> **Rule:** Always import `test`, `expect`, `describe` from the test context module in integration tests, never from `vitest` directly. The test context provides `tx`, `factory`, and `db` fixtures.

See TESTING.md for full fixture API, test context setup, and transaction isolation details.

---

## Running Targeted Checks

**Prescribed strict order** (`PROJECT.md` names the command for each step):

1. **API typecheck** — Catch type errors early (fast, targeted feedback)
2. **Format** — ESLint fix + Prettier (modifies files in place)
3. **Tests, run once** — Run all tests once (NOT a watch-mode script)
4. **Lint** — Final gate (includes typecheck again, plus ESLint strict mode and the project's other linters)

### Fix Flow vs Verify Flow

| Step | Fix Flow (during dev) | Verify Flow (CI / read-only) |
| ---- | --------------------- | ---------------------------- |
| 1    | API typecheck         | API typecheck                |
| 2    | Format (writes fixes) | Format check (read-only)     |
| 3    | Tests, run once       | Tests, run once              |
| 4    | Lint                  | Lint                         |

> **Note:** Lint re-runs typecheck internally. This is intentional as a final comprehensive gate that also includes ESLint strict mode and the project's other linters.

> **Caveat:** The API typecheck does not see the client. A change to what a route returns can pass it and still break the client's Eden type (TS2742 — see ELYSIA.md § Type Export Pattern), so run the client's typecheck as well after such a change.

### Command Reference

| Check            | What It Does                                  | When to Use                         |
| ---------------- | --------------------------------------------- | ----------------------------------- |
| API typecheck    | TypeScript type check for API app             | First check during dev              |
| Full typecheck   | TypeScript check for all packages             | Full project verification           |
| Client typecheck | TypeScript check for the app using API types  | After changing what a route returns |
| Format           | ESLint fix + Prettier (modifies files)        | During dev                          |
| Format check     | Same checks, read-only                        | CI / verify without changes         |
| Tests, run once  | Run all vitest tests once                     | After code changes                  |
| Tests, watch     | Tests in watch mode                           | Interactive development             |
| Lint             | ESLint strict + typecheck + the other linters | Final verification gate             |
| Lint fix         | ESLint auto-fix + the other linters' fixes    | Fix lint issues                     |

### Restricted Test Suites

Some suites are too slow for an agent to run (a service's full, slow or benchmark suite) and are developer-triggered only. `PROJECT.md` lists them:

| Suite                                       | Status    |
| ------------------------------------------- | --------- |
| Fast suites `PROJECT.md` allows             | Allowed   |
| Suites `PROJECT.md` marks as developer-only | NEVER run |

---

## Database Snapshot Workflow

Docker-volume-based instant snapshots, taken by the project's snapshot tool. No SQL dumps — snapshots are filesystem-level copies of the PostgreSQL data directory.

### Command Reference

| Step              | Description                         | Safety                      |
| ----------------- | ----------------------------------- | --------------------------- |
| Baseline snapshot | Create protected baseline snapshot  | Safe — non-destructive      |
| Safe push         | Auto-snapshot + push schema changes | **ALWAYS use this**         |
| Plain push        | Push without snapshot               | **NEVER use directly**      |
| Undo              | Revert the last safe push           | Restores previous snapshot  |
| Reset             | Restore baseline snapshot           | Resets to the baseline      |
| Snapshot `[name]` | Create named snapshot               | Manual checkpoint           |
| Restore `[name]`  | Restore named snapshot              | Manual restore              |
| List snapshots    | List all snapshots                  | Inspect available snapshots |
| Clean             | Remove this project's snapshots     | Full cleanup                |

### Typical Session Workflow

```
# Starting a new feature
baseline snapshot              # Protect current state
safe push                      # Push schema changes (auto-snapshots first)

# Something went wrong
undo                           # Revert last push

# Want to experiment
snapshot experiment-1          # Save current state
# ... try things ...
restore experiment-1           # Restore if needed

# Reset everything
reset                          # Back to baseline
```

### Hard Rules

1. **ALWAYS** use the safe push — never a plain push
2. **NEVER** generate migration files — only the user creates them
3. **ALWAYS** take a baseline snapshot before major experiments
4. Migration files are **production artifacts** — created intentionally by the user only

---

## Seeding

**Purpose:** Realistic development data for manual testing and UI development.

**Location:** the project's seed script. `PROJECT.md` says where it lives, how to run it, and where its data comes from (generated rows, or a snapshot of real data).

**When to seed:**

- After the safe push (schema changed, tables may be empty)
- After a reset (baseline restored, dev data lost)
- When starting fresh on a new machine

**Convention:** When you add a table, add it to the seed the way `PROJECT.md` describes, so the dev database stays populated with representative data.

**Seeding vs Test Data:**

- **Seed data** — development / UI / manual testing. Loaded by the seed script, never by tests.
- **Test data** — automated tests only. Uses the project's fixtures package exclusively.

> **Generated seeds:** For a seed that generates its rows, the `drizzle-seed` package gives deterministic, reproducible data instead of hand-written `db.insert()` calls.

---

## New Model Checklist

Condensed checklist for adding a new database model end-to-end:

1. **Schema file** — one file per table in the schema folder (use the shared columns, DRIZZLE.md § Common Columns Pattern)
2. **Barrel export** — Add to the schema barrel
3. **Type exports** — Export `$inferSelect` / `$inferInsert` types
4. **Relations** — Define `relations()` if foreign keys exist
5. **Schema folders** — If the table lives in a folder the project's tooling does not watch yet, register the folder (`PROJECT.md` says where), so the pre-commit schema check sees it
6. **Push** — The safe push
7. **Fixture** — A fixture file for the table, using `createFixture` from the project's fixtures package with `@faker-js/faker` fields
8. **Register fixture** — Add to the composed factory via `composeFactory`
9. **Tests** — Write collocated tests (import from the test context module)
10. **Route handler** — In the table's module folder
11. **Compose** — Mount into the main app via `.use()`, where the project composes its modules (`PROJECT.md`)
12. **Seed** — Add the table to the seed if the dev database needs its data
13. **Check** — Run strict order: API typecheck -> format -> tests, run once -> lint

---

## JSDoc Documentation

Rules for when and how to write JSDoc comments in the API codebase.

### Functions That MUST Have JSDoc

- **Service files** (`service.ts`) — business logic functions
- **Helper files** (`post-helpers.ts`, etc.) — shared logic used across handlers
- **Validation files** (`validation.ts`) — pure validation functions (not TypeBox schema definitions)
- **Utility files** (`utils/*.ts`) — shared utilities
- **Reusable schema objects** used across multiple files (e.g., shared column objects such as `timestamps` — DRIZZLE.md § Common Columns Pattern)

### Functions That Must NOT Have JSDoc

- Route handlers in `index.ts` — the route path + method + validation schema already document intent
- Schema definitions (`pgTable(...)` calls) — the table/column names are self-documenting
- Database connection setup
- Test files and fixtures
- Trivial one-liner functions where the name says everything (e.g., `ok()`, `fail()`)

### Format

```typescript
/**
 * Purpose sentence explaining what and why.
 * @param paramName - Description of the parameter
 * @returns Description (only when return value has nuances beyond the type signature)
 * @throws ErrorClass - Description (only for actual thrown exceptions)
 */
```

### Tag Rules

| Tag        | When to Include                                                                                                                                                                                       |
| ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `@param`   | **Always** — for every parameter                                                                                                                                                                      |
| `@returns` | **Only** when the return type alone doesn't convey what's returned (e.g., "returns the post with its tag names and author name populated"). Skip when the TypeScript return type is self-explanatory. |
| `@throws`  | **Only** for actual thrown exceptions (`throw new Error(...)`) — NOT for `fail()` error codes from `ServiceResult`. Most service functions return errors via `fail()`, those are not `@throws`.       |
| `@example` | **Never** — bloats docs without adding value in this kind of codebase                                                                                                                                 |

### Good Examples

**Service function** — all params documented, `@returns` adds context beyond the type:

```typescript
/**
 * Update a post's top-level fields and its tag links in a single transaction.
 * @param db - Database connection or transaction
 * @param postId - UUID of the post to update
 * @param input - Partial fields to patch (only provided fields are updated)
 * @returns Updated post with its tag names and author name populated
 */
export async function updatePost(
  db: DatabaseConnection,
  postId: string,
  input: UpdatePostInput
): Promise<ServiceResult<...>> {
```

**Helper function** — `@returns` skipped because `ServiceResult<Map<...>>` is self-explanatory:

```typescript
/**
 * Collect every tag name the post inputs refer to, fetch those tags from the
 * database in one query, and verify none are unknown.
 * @param db - Database connection or transaction
 * @param posts - Post inputs referring to tags by name
 * @param authorId - Author whose private tags may also be used
 */
export async function resolveTagNames(
  db: DatabaseConnection,
  posts?: PostInput[],
  authorId?: string
): Promise<ServiceResult<Map<string, TagInfo>>> {
```

**Validation function** — pure logic, no DB, `@returns` skipped:

```typescript
/**
 * Validate that a post's status is compatible with its publish date, and that
 * the fields its status requires are present.
 * @param post - The post input to validate
 * @param tagsByName - Map of tag name to tag (for the archived-tag check)
 * @param index - Position in the posts array (used in error messages)
 */
export function validatePostInput(
  post: PostInput,
  tagsByName: Map<string, TagInfo>,
  index: number
): ValidationResult {
```

### Bad Examples

**Over-documented** — `@returns`, `@throws` on a simple util that returns an obvious type:

```typescript
// BAD: @returns and @throws add no value here
/**
 * Map a ServiceError to an HTTP status code and response body.
 * @param error - The service error to map
 * @returns An object with status number and body containing error message
 * @throws TypeError - If error code is not in the status map
 */
export function mapError(error: ServiceError) { ... }

// GOOD: concise, @param only
/**
 * Map a ServiceError to an HTTP status code and response body.
 * @param error - The service error to map
 */
export function mapError(error: ServiceError) { ... }
```

**JSDoc on route handler** — route path + validation schema already document intent:

```typescript
// BAD: don't JSDoc route handlers
/**
 * Create a new post with its tag links.
 * @param body - The request body containing post data
 */
.post('/', async ({ body, db }) => { ... }, { body: CreatePostSchema })

// GOOD: no JSDoc needed — the route, method, and schema speak for themselves
.post('/', async ({ body, db }) => { ... }, { body: CreatePostSchema })
```

---

## Related Files

- SKILL.md — routing hub with decision tree, pattern index, common pitfalls
- ELYSIA.md — Elysia HTTP framework routes, lifecycle, plugins, DI, guards
- VALIDATION.md — validation reference: TypeBox API, targets, layers, rules
- DRIZZLE.md — Drizzle ORM schema, queries, transactions, connections
- QUERIES.md — query count per request, one-pass reads, parallel reads, bulk writes
- TESTING.md — test context, fixtures, route testing patterns
