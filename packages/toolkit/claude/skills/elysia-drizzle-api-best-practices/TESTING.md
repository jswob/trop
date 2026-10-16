# Testing & Fixtures - API Reference

API reference for TDD workflow, Vitest test context, transaction isolation, Elysia route testing, and the drizzle-fixtures library.

**When to Use:** Writing API tests, setting up test fixtures, testing Elysia routes against a database, creating factory data with traits, composing fixture relationships.

**Prerequisite:** `import { test, expect, describe }` from the project's test context module — never directly from `'vitest'` in tests that need database access. `PROJECT.md` gives the module's path; the examples here write it as `'test-context'`.

> **HARD RULE:** ALWAYS use `createFixture`/`composeFactory` from the project's fixtures package (`PROJECT.md` names it) for test data. Create a fixture for every new table.

## Table of Contents

- [TDD Workflow](#tdd-workflow)
- [Vitest Configuration](#vitest-configuration)
- [Test Context Pattern](#test-context-pattern)
- [Transaction Isolation](#transaction-isolation)
- [Elysia Route Testing](#elysia-route-testing)
- [Route + Database Integration](#route--database-integration)
- [Service Function Testing](#service-function-testing)
- [Query Budget Tests](#query-budget-tests)
- [createFixture API](#createfixture-api)
- [composeFactory + Builder](#composefactory--builder)
- [Traits](#traits)
- [Fixture Composition](#fixture-composition)
- [Deep Dive](#deep-dive)

---

## TDD Workflow

**Key Concepts:**

- Test-first for API endpoints: write the test, then implement the route handler
- Collocated test files: `routes/authors.ts` → `routes/authors.test.ts`
- Arrange-Act-Assert structure using test context fixtures
- One fixture file per database table, composed via `composeFactory()`

```typescript
import { test, expect, describe } from 'test-context';

describe('Authors API', () => {
	test('GET /api/authors returns all authors', async ({ factory, tx }) => {
		// Arrange — create test data in transaction
		const author = await factory.author.create();

		// Act — handle request (see Route + Database Integration)
		const app = createApp(tx);
		const res = await app.handle(new Request('http://localhost/api/authors'));
		const data = await res.json();

		// Assert
		expect(res.status).toBe(200);
		expect(data).toContainEqual(expect.objectContaining({ id: author.id }));

		// Cleanup — automatic via transaction rollback
	});
});
```

> **Caveat:** Always import `test`, `expect`, and `describe` from the test context module, not from `vitest`. The test context import provides `tx`, `factory`, and `db` fixtures.

See WORKFLOW.md § TDD Flow for the full red-green-refactor cycle and feature development sequence.

---

## Vitest Configuration

**Key Concepts:**

- `globals: true` enables `describe`/`it`/`expect` without explicit imports
- `environment: 'node'` for API testing (not jsdom)
- `tsconfigPaths()` plugin resolves TypeScript path aliases in tests
- Setup file loads `.env` from the monorepo root and points the app's database connection at the test database

```typescript
// vitest.config.ts — key settings: globals: true, environment: 'node',
// setupFiles: ['./vitest.setup.ts'], plugin: tsconfigPaths()

// vitest.setup.ts — loads env from the monorepo root, then points every connection
// the app opens at the test database (PROJECT.md says how this project does it):
process.env.DATABASE_URL = process.env.TEST_DATABASE_URL;
// afterAll: closes test DB connection
```

---

## Test Context Pattern

**Key Concepts:**

- `test.extend<T>()` creates a custom test with injected fixtures
- Three fixtures: `db` (database connection), `tx` (rollback transaction), `factory` (bound fixture builders)
- Fixture lifecycle: code before `use()` → setup, `use(value)` → inject, code after `use()` → teardown
- Fixtures only initialize when destructured in the test signature (lazy evaluation)
- Fixtures can depend on other fixtures via destructuring in the fixture function

```typescript
import { test as baseTest } from 'vitest';
import type { PostgresJsDatabase } from 'drizzle-orm/postgres-js';

interface TestFixtures {
	db: PostgresJsDatabase<typeof schema>;
	tx: TestTransaction;
	factory: BoundFactory;
}

export const test = baseTest.extend<TestFixtures>({
	// No dependencies — uses singleton DB connection
	db: async ({}, use) => {
		await use(getTestDb());
	},

	// No dependencies — creates fresh transaction per test
	tx: async ({}, use) => {
		const db = getTestDb();
		try {
			await db.transaction(async tx => {
				await use(tx as TestTransaction);
				tx.rollback();
			});
		} catch (error) {
			if (!(error instanceof TransactionRollbackError)) throw error;
		}
	},

	// Depends on tx — binds factory to transaction
	factory: async ({ tx }, use) => {
		const session = baseFactory(tx);
		await use(session);
	},
});

export { expect, describe } from 'vitest';
```

> **Caveat:** Fixtures only initialize if destructured in the test signature. A test with `async ({ factory })` will initialize `factory` (and its dependency `tx`), but not `db`. Use `{ auto: true }` as the second argument to force initialization.

---

## Transaction Isolation

**Key Concepts:**

- Each test gets a fresh PostgreSQL transaction — no shared state between tests
- After `use()` returns (test completes), `tx.rollback()` undoes all writes
- `TransactionRollbackError` is the expected outcome — caught and suppressed
- Child fixtures (like `factory`) inherit the same transaction automatically

```typescript
tx: async ({}, use) => {
  const db = getTestDb();
  try {
    await db.transaction(async (tx) => {
      // Setup: transaction begins
      await use(tx as TestTransaction);
      // Teardown: rollback all writes from this test
      tx.rollback();
    });
  } catch (error) {
    // TransactionRollbackError is expected — suppress it
    if (!(error instanceof TransactionRollbackError)) throw error;
  }
},
```

**Benefits:**

- **Guaranteed cleanup** — the database handles deletion order automatically
- **Test isolation** — tests never interfere with each other
- **Performance** — no manual DELETE queries or table truncation
- **Simplicity** — no tracking mechanism or cleanup hooks needed

> **Caveat:** Never nest manual `db.transaction()` calls inside a test that uses the `tx` fixture. The test is already running inside a transaction. Nested transactions require savepoints, which `tx.rollback()` would also undo.

---

## Elysia Route Testing

**Key Concepts:**

- `app.handle(new Request(...))` processes requests without starting an HTTP server
- Hostname in the URL is required by the `Request` constructor but ignored by Elysia
- Returns a standard `Response` object — use `.json()`, `.text()`, `.status`
- Test any Elysia route definition in isolation

```typescript
import Elysia from 'elysia';

it('should respond to health check', async () => {
	const app = new Elysia().get('/health', () => ({ status: 'ok' }));

	const response = await app.handle(new Request('http://localhost/health'));
	const data = await response.json();

	expect(response.status).toBe(200);
	expect(data).toMatchObject({ status: 'ok' });
});
```

**Request construction for different methods:**

```typescript
// GET with query params
new Request('http://localhost/api/authors?page=1&limit=10');

// POST with JSON body
new Request('http://localhost/api/authors', {
	method: 'POST',
	headers: { 'Content-Type': 'application/json' },
	body: JSON.stringify({ name: 'Dan', email: 'dan@test.com' }),
});

// With auth header
new Request('http://localhost/api/profile', {
	headers: { authorization: 'Bearer token123' },
});
```

---

## Route + Database Integration

**Key Concepts:**

- Combine test context fixtures with `app.handle()` for full integration tests
- Pass `tx` to the app via dependency injection (derive/decorate) so route queries run inside the test transaction
- Factory creates data in the same transaction — visible to route handlers, rolled back after test

```typescript
import { test, expect, describe } from 'test-context';
import { createApp } from 'src/app';

describe('Authors API', () => {
	test('GET /api/authors returns created authors', async ({ factory, tx }) => {
		// Arrange: factory is bound to tx — data is visible in transaction
		const author = await factory.author.create({ name: 'Dan' });

		// Act: app receives tx via DI — queries run in same transaction
		const app = createApp(tx);
		const res = await app.handle(new Request('http://localhost/api/authors'));
		const data = await res.json();

		// Assert
		expect(res.status).toBe(200);
		expect(data).toContainEqual(
			expect.objectContaining({
				id: author.id,
				name: 'Dan',
			})
		);
	});
});
```

**DI pattern for injecting the transaction:**

```typescript
const createApp = (db: DatabaseConnection) =>
	new Elysia().decorate('db', db).get('/api/authors', async ({ db }) => {
		return db.select().from(authors);
	});
```

> **Caveat:** The app must accept a database connection parameter for test DI to work. Use Elysia's `decorate()` or `derive()` to inject `tx` in place of the production `db`. Routes that import `db` directly from a module cannot be transaction-isolated.

---

## Service Function Testing

Service functions can be tested directly — call with `tx` and assert on the `ServiceResult`. Direct tests are **optional** — use for complex logic edge cases awkward to reach via HTTP. Integration tests (`app.handle()`) remain primary for happy paths.

```typescript
import { test, expect, describe } from 'test-context';
import { createPost } from './service';

describe('createPost service', () => {
	test('returns VALIDATION error for unknown tag name', async ({ tx, factory }) => {
		const author = await factory.author.create();
		const result = await createPost(tx, {
			authorId: author.id,
			title: 'Hello',
			tags: ['nonexistent'],
		});
		expect(result.ok).toBe(false);
		if (!result.ok) expect(result.error.code).toBe('VALIDATION');
	});
});
```

> **Caveat:** Same `tx` fixture — transaction isolation applies identically. See ELYSIA.md § Service Layer Pattern for `ServiceResult<T>` type.

---

## Query Budget Tests

**Key Concepts:**

- `countQueries(run)` — the project's statement counter (`PROJECT.md` says where it lives; `createQueryCounter()` from `@jswob/drizzle-utils` builds one) — the number of statements `run` sends to the test database
- A budget test proves the count is the **same for little data and for a lot**; that is the property a loop of queries breaks
- Write one for every list endpoint, nested read and multi-row save (QUERIES.md § The Rule)

```typescript
import { countQueries } from 'test-db'; // the project's statement counter (PROJECT.md)
import { test, expect, describe } from 'test-context';

describe('GET /api/posts — query budget', () => {
	test('lists no posts and six with the same queries', async ({ tx, factory }) => {
		const app = createPostsRoute(tx);

		const empty = await countQueries(() => app.handle(new Request(BASE_URL)));
		await factory.post.trait('withComments').createList(6); // six posts, each with comments
		const full = await countQueries(() => app.handle(new Request(BASE_URL)));

		expect(full).toBe(empty); // does not grow with the data
		expect(full).toBe(3); // adding a query is a decision, not an accident
	});
});
```

For a save, compare a small request with a large one of the same shape (one post and four, one tag and three): the two counts must be equal.

> **Caveat:** `countQueries` counts everything sent on the test connection while `run` is pending, savepoints included. Create fixtures before or between the calls, never inside `run`.

> **Caveat:** Compare like with like. A save that takes another branch (a status change, an empty part) legitimately sends other statements; the budget is about the same branch with more rows.

---

## createFixture API

**Key Concepts:**

- `createFixture<TTable, TTraits?, TAugmentations?>` defines a reusable fixture for a Drizzle table
- Config shape: `{ table, fields, traits?, hooks? }`
- Field resolvers receive `{ sequence, use }` — sequence is auto-incrementing, use composes with other fixtures
- Override priority (highest first): direct overrides → trait fields → base fields
- Returns a `FixtureFunction` — call with `(db)` to get a `FixtureBuilder`

```typescript
import { createFixture } from 'drizzle-fixtures'; // the project's fixtures package (PROJECT.md)
import { posts } from '../db/schema';

type PostTraits = {
	published: never;
	archived: never;
};

export const postFixture = createFixture<typeof posts, PostTraits>({
	table: posts,
	fields: {
		title: () => 'Test Post',
		slug: ({ sequence }) => `post-${sequence}`,
		authorId: async ({ use }) => {
			const author = await use(authorFixture).create();
			return author.id;
		},
	},
	traits: {
		published: {
			fields: { status: () => 'published', publishedAt: () => new Date() },
		},
		archived: {
			fields: { archived: () => true },
		},
	},
});
```

> **Caveat:** Only define fields that need explicit values. Columns with database defaults (`id` via `generatedAlwaysAsIdentity()`, `createdAt` via `defaultNow()`) should be omitted — the database generates them on insert.

---

## composeFactory + Builder

**Key Concepts:**

- `composeFactory(fixtures)` combines fixture functions into a single factory object
- Call factory with `(db)` or `(tx)` to bind all fixtures to a database connection
- Builder methods provide fluent, immutable chaining for creating test data

```typescript
import { composeFactory } from 'drizzle-fixtures';
import { authorFixture } from './author.fixture';
import { postFixture } from './post.fixture';

// Compose — one fixture per table
const fixtures = {
	author: authorFixture,
	post: postFixture,
};
export const factory = composeFactory(fixtures);
export type FixturesMap = typeof fixtures;

// Bind to transaction (in test context)
const session = factory(tx);
const author = await session.author.create();
const post = await session.post.create({ authorId: author.id });
```

**Builder methods:**

| Method                           | Description               | DB Operation |
| -------------------------------- | ------------------------- | :----------: |
| `.trait(name, params?)`          | Apply a named variation   |      No      |
| `.build(overrides?)`             | Create in-memory object   |      No      |
| `.buildList(count, overrides?)`  | Create multiple in-memory |      No      |
| `.create(overrides?)`            | Insert into database      |    INSERT    |
| `.createList(count, overrides?)` | Insert multiple rows      |  INSERT(s)   |

**Fluent chaining:**

```typescript
const post = await session.post
	.trait('published')
	.trait('archived')
	.create({ title: 'Launch Notes' });
// Override priority: { title: 'Launch Notes' } > trait fields > base fields
```

> **Caveat:** The builder is immutable — `.trait()` returns a new builder instance. Calling `builder.trait('published')` does not modify `builder`. Always chain: `builder.trait('published').create()`.

---

## Traits

**Key Concepts:**

- Traits define named variations of a fixture with different field values or side effects
- Type parameter: `never` for no-arg traits, object type for parameterized traits; the params reach the trait's `afterMake`, not its field resolvers
- Augmentations extend the return type by merging what a trait's `afterMake` returns
- When multiple traits set the same field, the last applied trait wins

```typescript
type AuthorTraits = {
	inactive: never; // No parameters
	suspended: never;
	withPosts: { count: number }; // Required parameter
};
type AuthorAugmentations = {
	withPosts: { posts: Post[] }; // Extends return type
};

const authorFixture = createFixture<typeof authors, AuthorTraits, AuthorAugmentations>({
	table: authors,
	fields: {
		name: () => 'Test Author',
		email: ({ sequence }) => `author${sequence}@example.com`,
		status: () => 'active',
	},
	traits: {
		inactive: {
			fields: { status: () => 'inactive' },
		},
		suspended: {
			fields: { status: () => 'suspended' },
		},
		withPosts: {
			afterMake: async ({ data, params, use }) => {
				const posts = await use(postFixture).createList(params.count, {
					authorId: data.id,
				});
				return { posts }; // Merged into return type
			},
		},
	},
});
```

**Usage:**

```typescript
// No-arg trait
const inactive = await session.author.trait('inactive').create();

// Parameterized trait with an augmented return — posts is typed on the result
const prolific = await session.author.trait('withPosts', { count: 5 }).create();
console.log(prolific.posts); // Post[] — fully typed

// Chained — last trait wins for overlapping fields
const result = await session.author.trait('inactive').trait('suspended').create();
// result.status === 'suspended' (suspended applied after inactive)
```

---

## Fixture Composition

**Key Concepts:**

- `use()` helper available in field resolvers and `afterMake` hooks — returns a full `FixtureBuilder`
- Propagates the database context — child fixtures use the same connection/transaction
- Use `.create()` (not `.build()`) for foreign key relationships to ensure the referenced record exists in the database

**In field resolvers** — create FK dependencies. **In a trait's `afterMake`** — attach related records:

```typescript
// Field resolver: auto-create parent records for FKs
const commentFixture = createFixture({
	table: comments,
	fields: {
		body: ({ sequence }) => `Comment ${sequence}`,
		postId: async ({ use }) => (await use(postFixture).create()).id,
		authorId: async ({ use }) => (await use(authorFixture).create()).id,
	},
});

// Trait afterMake: attach child records to the parent once its row exists
const postFixture = createFixture<
	typeof posts,
	{ withComments: never },
	{ withComments: { comments: Comment[] } }
>({
	table: posts,
	fields: {
		title: () => 'Test Post',
		authorId: async ({ use }) => (await use(authorFixture).create()).id,
	},
	traits: {
		withComments: {
			afterMake: async ({ data, use }) => {
				const comments = await use(commentFixture).createList(2, { postId: data.id });
				return { comments }; // Merged into return type as augmentation
			},
		},
	},
});
```

> **Caveat:** Always use `.create()` when composing fixtures for foreign key relationships. Using `.build()` only creates an in-memory object — no row is inserted, so the FK reference would point to a nonexistent record and cause a constraint violation.

> **Caveat:** The fixture-level `hooks.afterMake` runs before the insert and its return value is discarded, so it can neither attach children by the parent's id nor add to the return type. Attach children in a trait's `afterMake`: on `.create()` it runs after the insert, with the inserted row.

---

## Deep Dive

**Official Docs:** [Vitest Test Context](https://vitest.dev/guide/test-context) · [Vitest Config](https://vitest.dev/config/) · [Drizzle Transactions](https://orm.drizzle.team/docs/transactions) · [drizzle-fixtures](https://github.com/jswob/drizzle-fixtures)

**Related Files:** SKILL.md (routing hub) · ELYSIA.md (routes, lifecycle, plugins, DI) · VALIDATION.md (TypeBox API, layers) · DRIZZLE.md (schema, queries, transactions) · QUERIES.md (query count, batching, bulk writes) · WORKFLOW.md (TDD flow, DB snapshots, seeding)
