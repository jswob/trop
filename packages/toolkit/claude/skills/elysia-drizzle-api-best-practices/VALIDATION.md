# Validation - Complete Reference

Validation layer responsibilities, TypeBox schema API, and placement rules for Elysia API routes.

**When to Use:** Deciding where to place validation logic, writing TypeBox schemas, choosing validation targets, adding custom error messages.

**Prerequisite:** `import { t } from 'elysia'` for TypeBox schemas.

## Table of Contents

- [TypeBox Schema API](#typebox-schema-api)
- [Validation Targets](#validation-targets)
- [Custom Error Messages](#custom-error-messages)
- [Caveats](#caveats)
- [Validation Layers](#validation-layers)
- [Rules](#rules)
- [Examples](#examples)
- [Related Files](#related-files)

---

## TypeBox Schema API

**Key Concepts:**

- `t.Object({...})` — TypeBox schema for runtime + compile-time validation
- Six validation targets: `body`, `query`, `params`, `headers`, `cookie`, `response`
- `typeof schema.static` — extract TypeScript type from any TypeBox schema
- Schema serves four purposes: runtime validation, type coercion, TypeScript types, OpenAPI spec

```typescript
import { Elysia, t } from 'elysia';

const app = new Elysia()
	.post('/users', ({ body }) => createUser(body), {
		body: t.Object({
			name: t.String({ minLength: 1 }),
			email: t.String({ format: 'email' }),
			age: t.Optional(t.Number({ minimum: 0 })),
		}),
		response: {
			200: t.Object({ id: t.Number(), name: t.String() }),
			400: t.Object({ error: t.String() }),
		},
	})
	.get('/users', ({ query }) => findUsers(query), {
		query: t.Object({
			page: t.Number({ default: 1 }),
			limit: t.Number({ default: 20 }),
		}),
	});
```

---

## Validation Targets

| Target     | Use                      | Notes                                |
| ---------- | ------------------------ | ------------------------------------ |
| `body`     | JSON, form-data payloads | Disabled for GET/HEAD per RFC 2616   |
| `query`    | URL query parameters     | Auto-coerced to declared types       |
| `params`   | Path parameters (`:id`)  | Inferred as strings unless typed     |
| `headers`  | Request headers          | Keys lowercase-normalized            |
| `cookie`   | Request cookies          | `additionalProperties: true` default |
| `response` | Handler return values    | Can be status-code-specific          |

---

## Custom Error Messages

Every field with constraints (`minLength`, `format`, etc.) should include a custom error message.

```typescript
const uuidParams = t.Object({
	id: t.String({ format: 'uuid', error: 'Invalid UUID format' }),
});

const tagBody = t.Object({
	name: t.String({ minLength: 1, error: 'Name is required' }),
});

// Dynamic error function
t.Number({ error: ({ value }) => `Expected number, got ${typeof value}` });
```

---

## Caveats

- **Query auto-coercion:** Query values are always strings in the URL. Elysia auto-coerces to declared types, but be explicit with schemas.
- **Production validation detail stripping:** Validation details are stripped in production by default to prevent schema leakage. Override with `Elysia.allowUnsafeValidationDetails = true` (not recommended).

---

## Validation Layers

| Layer         | Where                                  | What                                                                          |
| ------------- | -------------------------------------- | ----------------------------------------------------------------------------- |
| **Schema**    | Route config `{ body, params, query }` | Shape, types, formats, discriminated unions, custom error messages            |
| **Transform** | `onTransform` hook                     | Trim whitespace, normalize casing                                             |
| **Handler**   | Handler body                           | Entity existence (404), DB constraints (409), DB-dependent domain rules (422) |

---

## Rules

1. **If a rule can be a TypeBox schema, it must be.** Shape, types, formats, and conditional required fields belong in the schema — not the handler.

2. **Use discriminated unions for conditional fields on POST.** When a field is required only for certain values of another field (e.g., `publishAt` required when `status` is `"scheduled"`), use `t.Union([...])` with distinct `t.Object` variants. For PATCH with optional fields, discriminated unions don't apply — use handler validation instead.

3. **Trim in `transform`, not handlers.** Add an `onTransform` hook at the plugin level to trim string fields before schema validation runs. The schema's `minLength: 1` then catches whitespace-only values automatically.

4. **Handler validation only for DB-dependent rules.** Entity existence checks (404), uniqueness constraint violations (409), and domain rules that require database lookups (422) belong in the handler.

5. **Always add `{ error }` to TypeBox fields.** Every field with constraints (`minLength`, `format`, etc.) should include a custom error message: `t.String({ minLength: 1, error: 'Name is required' })`.

6. **Use `return status()`, not throw.** Status responses bypass the `onError` hook chain and provide type narrowing.

---

## Examples

### Discriminated Union (POST body)

```typescript
const postBody = t.Union([
	t.Object({
		title: t.String({ minLength: 1, error: 'Title is required' }),
		status: t.Literal('scheduled'),
		publishAt: t.String({
			format: 'date-time',
			error: 'publishAt must be an ISO date-time for a scheduled post',
		}),
	}),
	t.Object({
		title: t.String({ minLength: 1, error: 'Title is required' }),
		status: t.Union([t.Literal('draft'), t.Literal('published')]),
	}),
]);
```

### Transform Hook (whitespace trimming)

```typescript
new Elysia({ prefix: '/api/tags' }).onTransform(({ body }) => {
	if (body && typeof body === 'object' && 'name' in body && typeof body.name === 'string') {
		body.name = body.name.trim();
	}
});
// ... routes (no manual trim() in handlers)
```

---

## Related Files

- ELYSIA.md -- Elysia HTTP framework routes, lifecycle, plugins, DI, guards
- WORKFLOW.md -- TDD flow and development workflow
