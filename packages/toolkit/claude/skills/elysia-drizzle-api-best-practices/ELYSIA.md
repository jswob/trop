# Elysia Framework - API Reference

API reference for Elysia HTTP framework patterns in TypeScript/Bun.

**When to Use:** Building or modifying API routes, adding validation, configuring lifecycle hooks, handling errors, structuring plugins, injecting dependencies.

**Prerequisite:** `import Elysia from 'elysia'` and `import { t } from 'elysia'` for TypeBox schemas.

## Table of Contents

- [Route Composition](#route-composition)
- [Validation](#validation)
- [Handler Patterns](#handler-patterns)
- [Service Layer Pattern](#service-layer-pattern)
- [Lifecycle Hooks](#lifecycle-hooks)
- [Error Handling](#error-handling)
- [Plugins](#plugins)
- [Dependency Injection](#dependency-injection)
- [Type Export Pattern](#type-export-pattern)
- [Guard and Scope](#guard-and-scope)
- [Deep Dive](#deep-dive)

---

## Route Composition

**Key Concepts:**

- `.get(path, handler, hook?)` — register GET route; same signature for `.post`, `.put`, `.patch`, `.delete`
- `.all(path, handler, hook?)` — match any HTTP method
- `.group(prefix, app => app.route(...))` — nest routes under a shared prefix
- `new Elysia({ prefix })` — preferred modular alternative to `.group()`
- Method chaining — all route methods return the instance for fluent composition

```typescript
// Method chaining
const app = new Elysia()
	.get('/users', () => 'list users')
	.post('/users', ({ body }) => body)
	.get('/users/:id', ({ params: { id } }) => id);

// Grouping with prefix
app.group('/posts', app =>
	app
		.get('/', () => 'all posts')
		.post('/', ({ body }) => body)
		.get('/:id', ({ params: { id } }) => id)
);
```

**Path parameter syntax:**

| Pattern  | Example       | Captures                        |
| -------- | ------------- | ------------------------------- |
| Static   | `/users/list` | Exact match only                |
| Dynamic  | `/users/:id`  | Single segment as `params.id`   |
| Optional | `/users/:id?` | Matches with or without segment |
| Wildcard | `/files/*`    | Remaining path as `params['*']` |

Resolution priority: static > dynamic > wildcard.

> **Caveat:** Treat one Elysia instance as one controller. Each module should create its own `new Elysia({ prefix })` instance and export it for `.use()` composition. This enables automatic type inference and clean separation of concerns.

---

## Validation

See **VALIDATION.md** for the complete validation reference: TypeBox schema API, validation targets, custom error messages, layer responsibilities, and placement rules.

For applying schemas to groups of routes, see [Guard and Scope](#guard-and-scope).

---

## Handler Patterns

**Key Concepts:**

- Handler receives a Context object and returns a response value
- `status(code, body)` — preferred status handling with type narrowing
- `set.status` — legacy approach, no type inference on return value
- Literal values (strings, numbers) are compile-time optimized

```typescript
const app = new Elysia()
	// Standard handler with context destructuring
	.get('/users/:id', ({ params: { id }, status }) => {
		const user = findUser(id);
		if (!user) return status(404, { error: 'Not found' });
		return user;
	})
	// Response headers
	.get('/data', ({ set }) => {
		set.headers['x-custom'] = 'value';
		return { data: 'payload' };
	})
	// Redirect
	.get('/old-path', ({ redirect }) => redirect('/new-path', 301));
```

**Context properties:**

| Property                      | Description                                            |
| ----------------------------- | ------------------------------------------------------ |
| `body` / `query` / `params`   | Parsed request body, query string, path parameters     |
| `headers` / `cookie`          | Request headers (lowercase keys) / cookie signal store |
| `store`                       | Global mutable state (from `.state()`)                 |
| `request` / `path` / `server` | Web Standard Request, pathname, Bun server instance    |
| `status(code, body)`          | Function for typed status responses                    |
| `set`                         | Object with `status`, `headers` for response mutation  |
| `redirect(url, code?)`        | Function for HTTP redirects                            |

> **Caveat:** Always prefer `status()` over `set.status`. The `status()` function enables TypeScript type narrowing based on response schemas, while `set.status` does not validate or infer return types.

---

## Service Layer Pattern

**Key Concepts:**

- Extract complex handler logic (>30 lines, multi-phase) into `service.ts` as plain async functions
- Service functions return `ServiceResult<T>` — a discriminated union for typed success/error handling
- Route handlers become thin HTTP adapters that call the service and map the result
- Simple CRUD (single DB call + return) stays inline — no extraction needed

**Extraction heuristic:** If the handler body has more than one "phase" (e.g., fetch-then-validate-then-write = 3 phases), extract to `service.ts`. If it's a single DB call + return, leave inline.

**`ServiceResult<T>` helpers** (in the project's service-result module; `PROJECT.md` gives its path): `ok(data)` wraps success, `fail(code, message)` wraps error, `mapError(error)` returns the route's `status()` response. Error codes: `NOT_FOUND` (404), `VALIDATION` (422), `CONFLICT` (409), `INTERNAL` (500). Each status is a literal, so Eden keeps the failure on the client's `error` and `data` the success type; a route never builds `status(n, …)` from a `number`.

**What goes where:** TypeBox schemas + simple CRUD → `index.ts` | Multi-step business logic → `service.ts` | Pure validation → `validation.ts` | Error types → `utils/service-result.ts` (shared).

**Example — service function + thin route adapter:**

```typescript
// service.ts — all business logic, returns ServiceResult
export async function createPost(
  db: DatabaseConnection, input: CreatePostInput
): Promise<ServiceResult<PostWithDetails>> {
  const [[author], tagMap] = await Promise.all([
    db.select({ id: authors.id }).from(authors).where(eq(authors.id, input.authorId)),
    resolveTagNames(db, input.tags),
  ]);
  if (!author) return fail('NOT_FOUND', 'Author not found');
  if (!tagMap.ok) return tagMap;
  const result = await db.transaction(async (tx) => { /* insert the post, then its tag links */ });
  return ok(result);
}

// index.ts — thin HTTP adapter
.post('/:id/posts', async ({ params, body, status }) => {
  const result = await createPost(db, { authorId: params.id, ...body });
  if (!result.ok) return mapError(result.error);
  return status(201, result.data);
}, { params: uuidParams, body: postBody })
```

> **Caveat:** Service functions accept `db: DatabaseConnection` as their first parameter — never import `db` directly. This enables transaction injection in tests.

> **Best Practice:** Shared logic across modules goes in a shared helper (e.g., `modules/tags/tag-names.ts`), not duplicated across service files. See WORKFLOW.md for module folder structure.

---

## Lifecycle Hooks

**Execution order (11 stages):**

1. **onRequest** — first event for every request (PreContext only — no body/query/params)
2. **onParse** — parse body into `context.body`
3. **onTransform** — mutate context before validation
4. **derive** — append context properties before validation (shared queue with transform)
5. **onBeforeHandle** — custom validation / auth checks before handler
6. **resolve** — append context properties after validation (shared queue with beforeHandle)
7. **Handler** — route handler execution
8. **onAfterHandle** — transform handler return value
9. **mapResponse** — custom response mapping
10. **onError** — catches errors from any lifecycle stage
11. **onAfterResponse** — cleanup after response is sent

```typescript
const app = new Elysia()
	// Interceptor hooks (global/scoped)
	.onRequest(({ request }) => console.log(`${request.method} ${request.url}`))
	.onBeforeHandle(({ headers, status }) => {
		if (!headers['authorization']) return status(401, 'Unauthorized');
	})
	.onAfterHandle(({ response }) => {
		if (typeof response === 'object') return { ...response, timestamp: Date.now() };
	})
	.onError(({ code, error }) => {
		if (code === 'VALIDATION') return { error: error.message };
	})
	// Local hooks (per-route)
	.get('/users', () => 'list', {
		beforeHandle({ headers, status }) {
			if (!headers['x-api-key']) return status(403, 'Forbidden');
		},
	});
```

**Return value behavior:**

| Hook           | Return Effect                                            |
| -------------- | -------------------------------------------------------- |
| onRequest      | Returns value → skips remaining lifecycle                |
| onBeforeHandle | Returns value → skips route handler                      |
| onAfterHandle  | Returns undefined → preserves response; other → replaces |
| mapResponse    | Returns value → skips remaining mapResponse              |
| onError        | Returns value → custom error response                    |

> **Warning:** Hooks only apply to routes registered **after** them. This is a critical ordering rule. Exception: `onRequest` is effectively global.

For scoping hooks to specific route groups, see [Guard and Scope](#guard-and-scope).

---

## Error Handling

**Built-in error codes:**

- `NOT_FOUND` — no matching route
- `PARSE` — body parsing failure
- `VALIDATION` — schema validation failure
- `INTERNAL_SERVER_ERROR` — unhandled errors
- `INVALID_COOKIE_SIGNATURE` — cookie signature mismatch
- `UNKNOWN` — default fallback (status 500)

```typescript
const app = new Elysia().onError(({ code, error, status, path }) => {
	if (code === 'NOT_FOUND') return status(404, { error: 'Route not found', path });
	if (code === 'VALIDATION') return status(422, { error: error.message });
	// Unhandled → falls through to default error response
});
```

**Custom error classes:**

```typescript
class NotAuthorizedError extends Error {
	constructor(message = 'Not authorized') {
		super(message);
	}
}

const app = new Elysia()
	.error({ NotAuthorizedError }) // Register for type narrowing in onError
	.onError(({ code, error, status }) => {
		if (code === 'NotAuthorizedError') return status(403, error.message);
	});
```

**Throw vs return — choose deliberately:**

| Pattern                    | Behavior                             |
| -------------------------- | ------------------------------------ |
| `throw status(418)`        | Triggers `onError` hook chain        |
| `return status(418, body)` | Bypasses `onError`, returns directly |
| `throw new CustomError()`  | Triggers `onError` with custom code  |

> **Warning:** Error type narrowing in `onError` requires prior `.error()` registration. Without it, `code` won't include your custom error names.

> **Caveat:** Validation details are stripped in production to prevent schema leakage. Override with `Elysia.allowUnsafeValidationDetails = true` (not recommended).

---

## Plugins

**Key Concepts:**

- Plugins are decoupled Elysia instances composed via `.use()`
- Instance-based plugins (recommended) — superior type inference
- Plugin deduplication via `name` property prevents duplicate lifecycle execution
- `state` and `decorate` properties are inherited; lifecycle hooks are NOT

```typescript
// Instance-based plugin (recommended)
const userPlugin = new Elysia({ prefix: '/users' })
	.get('/', () => 'list users')
	.get('/:id', ({ params: { id } }) => `user ${id}`)
	.post('/', ({ body }) => body);

// Compose into main app
const app = new Elysia().use(userPlugin).use(postPlugin).listen(3000);
```

**Plugin deduplication:**

```typescript
const authPlugin = new Elysia({ name: 'auth' }).derive({ as: 'global' }, ({ headers }) => ({
	bearer: headers['authorization']?.slice(7) ?? null,
}));

// Applied to multiple sub-routers but lifecycle runs only once
const usersRouter = new Elysia({ prefix: '/users' }).use(authPlugin);
const postsRouter = new Elysia({ prefix: '/posts' }).use(authPlugin);
```

**Creation patterns:**

| Pattern                                | Type Inference | Use When                    |
| -------------------------------------- | -------------- | --------------------------- |
| Instance-based (`new Elysia()`)        | Full           | Default — always prefer     |
| Functional (`(app) => app.state(...)`) | Partial        | Accessing parent properties |

For controlling hook and schema propagation across plugin boundaries, see [Guard and Scope](#guard-and-scope).

---

## Dependency Injection

**Key Concepts:**

- `state(key, value)` — global mutable state, accessed via `store.key`
- `decorate(key, value)` — immutable context properties (services, utilities)
- `derive(fn)` — per-request properties computed before validation (untyped request data)
- `resolve(fn)` — per-request properties computed after validation (typed request data)

```typescript
const app = new Elysia()
	// Global: assigned once at startup
	.state('requestCount', 0)
	.decorate('userService', new UserService())
	// Per-request: derive runs before validation (untyped access)
	.derive(({ headers }) => ({
		bearer: headers['authorization']?.slice(7) ?? null,
	}))
	// Per-request: resolve runs after validation (typed access)
	.guard({ headers: t.Object({ authorization: t.String() }) })
	.resolve(({ headers }) => ({
		token: headers.authorization.slice(7), // fully typed
	}))
	.get('/profile', ({ token, userService }) => {
		return userService.findById(decodeToken(token).userId);
	});
```

**Comparison:**

| Feature             | state  | decorate        | derive                     | resolve                        |
| ------------------- | ------ | --------------- | -------------------------- | ------------------------------ |
| Timing              | Init   | Init            | Transform (pre-validation) | BeforeHandle (post-validation) |
| Access request data | No     | No              | Yes (untyped)              | Yes (typed)                    |
| Type safety         | Yes    | Yes             | No (raw headers)           | Yes                            |
| Mutability          | Yes    | No (convention) | N/A                        | N/A                            |
| Scope               | Global | Global          | Per-request                | Per-request                    |

**DI as plugin pattern** — wrap services as named Elysia instances for type integrity and deduplication:

```typescript
const authPlugin = new Elysia({ name: 'auth' })
	.decorate('db', database)
	.derive(({ headers }) => ({ bearer: headers['authorization']?.slice(7) ?? null }));
```

> **Best Practice:** Use static services via `decorate()` for non-request logic. Use Elysia plugins with `derive()`/`resolve()` for request-dependent services (auth, tenant). See WORKFLOW.md § Feature Development Flow for recommended folder structure.

> **Caveat:** Decouple business logic from Elysia. Services should be plain classes or functions injected via `decorate()`, not Elysia-coupled controllers. This avoids vendor lock-in and keeps logic independently testable.

---

## Type Export Pattern

**Key Concepts:**

- `export type App = typeof app` — expose full app type for type-safe clients
- `typeof schema.static` — extract TypeScript type from any TypeBox schema
- Schema is the single source of truth for types, validation, and OpenAPI

```typescript
// server.ts
import { Elysia, t } from 'elysia';

const UserSchema = t.Object({
	id: t.Number(),
	name: t.String(),
	email: t.String({ format: 'email' }),
});

// Extract type from schema — no separate interface needed
type User = typeof UserSchema.static;

const app = new Elysia()
	.post('/users', ({ body }) => createUser(body), {
		body: t.Omit(UserSchema, ['id']),
		response: { 200: UserSchema },
	})
	.listen(3000);

// Export app type for type-safe clients
export type App = typeof app;
```

**Anti-patterns to avoid:**

- Separate `interface User` duplicating schema fields — use `typeof schema.static`
- Manual type declarations for request/response — let Elysia infer from schemas
- Class-based models with duplicate validation — TypeBox handles both

> **Caveat:** A client app (a web app with an Eden client) builds the type of its client from what each route returns, and it reaches the API only through package exports. A route that returns rows typed by a **named** type exported from a module file (`Promise<PostWithDetails[]>`, an `interface`) makes the client's typecheck fail with TS2742 — "the inferred type of 'client' cannot be named without a reference to …/api/src/modules/…" — while the API's own typecheck stays green. Write the return type of anything a route hands back **out in full** (an anonymous object type), and if the API wants a name for it, derive the name from the function: `type PostWithDetails = Awaited<ReturnType<typeof getPostsWithDetails>>[number]`. After changing what a route returns, run the client's typecheck (`PROJECT.md` names it); only that catches it.

---

## Guard and Scope

This section is the authoritative reference for guard and scope behavior. Other sections cross-reference here.

**Key Concepts:**

- `guard(schema, app)` — apply schemas and hooks to a group of routes
- Schema precedence: local route > guard > global (latest override wins)
- Two merge strategies: `override` (default) and `standalone`
- Three scope levels: `local`, `scoped`, `global`

**Guard for schema grouping:**

```typescript
const app = new Elysia()
	.guard(
		{
			body: t.Object({
				username: t.String(),
				password: t.String(),
			}),
		},
		app =>
			app.post('/sign-in', ({ body }) => signIn(body)).post('/sign-up', ({ body }) => signUp(body))
	)
	.get('/public', () => 'no guard applied');
```

**Guard for hook scoping:**

```typescript
.guard({
  beforeHandle({ headers, status }) {
    if (!headers['authorization']) return status(401, 'Unauthorized')
  }
}, (app) => app
  .get('/profile', ({ bearer }) => getProfile(bearer))
  .get('/settings', () => getSettings())
)
```

**Merge strategies:**

| Strategy             | Behavior                                        | Use When                    |
| -------------------- | ----------------------------------------------- | --------------------------- |
| `override` (default) | Local schema replaces guard schema on collision | Route needs different shape |
| `standalone`         | Both validations run independently              | Guard adds extra checks     |

```typescript
.guard({ schema: 'standalone', response: t.Object({ ok: t.Boolean() }) })
```

**Scope levels — control hook/schema propagation:**

| Level             | Propagation                    | Use When                               |
| ----------------- | ------------------------------ | -------------------------------------- |
| `local` (default) | Current instance + descendants | Route-specific logic                   |
| `scoped`          | Parent + current + descendants | Shared within a module                 |
| `global`          | All instances using the plugin | Cross-cutting concerns (auth, logging) |

**Setting scope:**

```typescript
// Inline on hook/derive/resolve
.derive({ as: 'scoped' }, ({ headers }) => ({ token: headers['x-token'] }))

// On guard
.guard({ as: 'scoped', response: t.String() })

// On entire plugin instance (lifts all hooks up one level)
const plugin = new Elysia({ name: 'auth' })
  .onBeforeHandle(({ headers }) => { /* ... */ })
plugin.as('scoped')
```

---

## Deep Dive

**Official Documentation:**

- [Route Composition](https://elysiajs.com/essential/route)
- [Validation](https://elysiajs.com/essential/validation)
- [Handler](https://elysiajs.com/essential/handler)
- [Lifecycle](https://elysiajs.com/essential/life-cycle)
- [Error Handling](https://elysiajs.com/patterns/error-handling)
- [Plugins](https://elysiajs.com/essential/plugin)
- [Dependency Injection](https://elysiajs.com/patterns/extends-context)
- [Best Practices](https://elysiajs.com/essential/best-practice)

**Related Files:**

- SKILL.md — routing hub with decision tree, pattern index, common pitfalls
- DRIZZLE.md — Drizzle ORM schema, queries, transactions
- TESTING.md — test context, fixtures, route testing patterns
- WORKFLOW.md — TDD flow, targeted checks, DB snapshots, seeding, new model checklist
- VALIDATION.md — complete validation reference: TypeBox API, targets, layers, rules
