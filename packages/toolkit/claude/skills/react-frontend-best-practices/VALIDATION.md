# Zod Validation — Reference

Patterns for runtime validation using Zod in React SPAs: form validation, search params, schema composition, and environment variables.

**When to Use:** Validating user input, URL search params, localStorage data, environment variables, or external API responses.

**Prerequisite:** Zod v3 or v4. For search params integration, see [ROUTING.md § Search Params](ROUTING.md#search-params). For API data fetching patterns, see [QUERY_EDEN.md](QUERY_EDEN.md).

---

## Table of Contents

- [Zod Fundamentals](#zod-fundamentals)
- [Form Validation](#form-validation)
- [Error Handling](#error-handling)
- [Schema Composition](#schema-composition)
- [Schema Organization](#schema-organization)
- [Search Params](#search-params)
- [API Response Validation](#api-response-validation)
- [Environment Variables](#environment-variables)
- [Deep Dive](#deep-dive)

---

## Zod Fundamentals

- **Schema = single source of truth** — define once, get both runtime validation and TypeScript type via `z.infer<>`
- **`.safeParse()` over `.parse()` in UI** — returns discriminated union instead of throwing
- **Built-in validators** for common patterns (email, UUID, min/max, regex)
- **`.refine()` for custom logic** — cross-field validation, async checks

### Schema + Type Inference

```typescript
import { z } from 'zod';

const userSchema = z.object({
	id: z.string().uuid(),
	email: z.string().email('Invalid email'),
	age: z.number().int().positive().optional(),
	role: z.enum(['admin', 'user', 'guest']),
});

type User = z.infer<typeof userSchema>;
// { id: string; email: string; age?: number; role: 'admin' | 'user' | 'guest' }
```

### Parsing

| Method             | On Failure                          | Use When                         |
| ------------------ | ----------------------------------- | -------------------------------- |
| `.parse(data)`     | Throws `ZodError`                   | Inside try/catch, simple scripts |
| `.safeParse(data)` | Returns `{ success: false, error }` | **UI code** — forms, components  |

```typescript
const result = schema.safeParse(formData);
if (!result.success) {
	console.log(result.error.issues); // Array of validation issues
} else {
	submitToApi(result.data); // Fully typed
}
```

### Common Validators

| Type     | Validators                                                                     |
| -------- | ------------------------------------------------------------------------------ |
| String   | `.min(1)`, `.max(200)`, `.email()`, `.url()`, `.uuid()`, `.regex()`, `.trim()` |
| Number   | `.int()`, `.positive()`, `.nonnegative()`, `.min()`, `.max()`, `.finite()`     |
| Array    | `.nonempty()`, `.min()`, `.max()`, `.length()`                                 |
| Optional | `.optional()`, `.nullable()`, `.nullish()`, `.default()`, `.catch()`           |

### Cross-Field Validation

```typescript
const passwordForm = z
	.object({
		password: z.string().min(8),
		confirmPassword: z.string(),
	})
	.refine(data => data.password === data.confirmPassword, {
		message: "Passwords don't match",
		path: ['confirmPassword'],
	});
```

---

## Form Validation

This project uses Zod with native form events — no React Hook Form dependency. Two patterns cover most use cases.

### Pattern A: FormData + safeParse (Simple Forms)

Validate only on submit. Minimal code, no real-time feedback.

```typescript
function SignUpForm() {
  const [errors, setErrors] = useState<Record<string, string>>({})

  const handleSubmit = (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    const formData = Object.fromEntries(new FormData(e.currentTarget))
    const result = schema.safeParse(formData)

    if (!result.success) {
      const fieldErrors: Record<string, string> = {}
      result.error.issues.forEach((issue) => {
        const field = issue.path[0] as string
        if (!fieldErrors[field]) fieldErrors[field] = issue.message
      })
      setErrors(fieldErrors)
    } else {
      setErrors({})
      submitToApi(result.data)
    }
  }

  return (
    <form onSubmit={handleSubmit} noValidate>
      <input name="email" type="email" />
      {errors.email && <p className="text-red-500 text-sm">{errors.email}</p>}
      <button type="submit">Submit</button>
    </form>
  )
}
```

### Pattern B: Controlled + onBlur (Production Forms)

Real-time validation on blur, re-validation on change after first touch.

```typescript
const [values, setValues] = useState({ email: '', name: '' });
const [errors, setErrors] = useState<Record<string, string>>({});
const [touched, setTouched] = useState<Record<string, boolean>>({});

// Validate single field using schema.shape
const validateField = (field: string, value: string) => {
	const result = schema.shape[field as keyof typeof schema.shape].safeParse(value);
	return result.success ? undefined : result.error.issues[0]?.message;
};

const handleBlur = (e: FocusEvent<HTMLInputElement>) => {
	const { name, value } = e.target;
	setTouched(prev => ({ ...prev, [name]: true }));
	setErrors(prev => ({ ...prev, [name]: validateField(name, value) }));
};

const handleChange = (e: ChangeEvent<HTMLInputElement>) => {
	const { name, value } = e.target;
	setValues(prev => ({ ...prev, [name]: value }));
	if (touched[name]) {
		// Re-validate only if already touched
		setErrors(prev => ({ ...prev, [name]: validateField(name, value) }));
	}
};

const showError = (field: string) => touched[field] && errors[field];
```

**Key techniques:**

- `schema.shape[field]` extracts single-field schema for per-field validation
- `touched` state prevents errors from showing before user interaction
- Full schema validation still runs on submit (catches cross-field `.refine()` rules)

### Accessibility Checklist

- Add `noValidate` to `<form>` to disable native browser validation
- Use `aria-invalid={!!error}` on inputs with errors
- Use `aria-describedby="field-error"` linking input to error message `id`
- Use `role="alert"` on error messages for screen reader announcements
- Associate `<label htmlFor>` with `<input id>`

For testing forms with React Testing Library, see [TESTING.md § Component Testing Fundamentals](TESTING.md#component-testing-fundamentals).

---

## Error Handling

### `.flatten()` — Field-Error Map

Best for single-level object schemas (most forms):

```typescript
if (!result.success) {
	const flat = result.error.flatten();
	// { formErrors: [], fieldErrors: { email: ['Invalid email'], age: ['Expected number'] } }
}
```

### Custom Utility: First Error Per Field

For full control over error extraction:

```typescript
function zodErrorToFieldErrors(error: z.ZodError): Record<string, string> {
	const fieldErrors: Record<string, string> = {};
	for (const issue of error.issues) {
		const key = issue.path.join('.'); // Handles nested: 'address.street'
		if (!fieldErrors[key]) fieldErrors[key] = issue.message;
	}
	return fieldErrors;
}
```

### Display Patterns

**Inline (per-field):** Error message directly below input — recommended for most forms.

**Summary (top of form):** All errors grouped at top — useful for long forms or as secondary display.

```tsx
{
	Object.keys(errors).length > 0 && (
		<div role="alert" className="bg-red-50 border border-red-200 p-4 rounded mb-4">
			<ul className="list-disc list-inside text-red-700 text-sm">
				{Object.entries(errors).map(([field, msg]) => (
					<li key={field}>{msg}</li>
				))}
			</ul>
		</div>
	);
}
```

### Zod v3 vs v4 Differences

| Feature          | Zod v3                 | Zod v4                      |
| ---------------- | ---------------------- | --------------------------- |
| Nested errors    | `.format()`            | `z.treeifyError()`          |
| Global error map | `z.setErrorMap()`      | `z.config({ customError })` |
| Custom messages  | `message` + `errorMap` | Unified `error` parameter   |

---

## Schema Composition

### Methods

| Method                   | Use Case                                    |
| ------------------------ | ------------------------------------------- |
| `.extend({ field })`     | Add or override properties on a base schema |
| `.merge(otherSchema)`    | Combine two independent schemas             |
| `.pick({ field: true })` | Select specific fields                      |
| `.omit({ field: true })` | Exclude specific fields                     |
| `.partial()`             | Make all fields optional                    |
| `.required()`            | Make all optional fields required           |

### CRUD Derivation Pattern

```typescript
const itemSchema = z.object({
	id: z.string().uuid(),
	name: z.string().min(1).max(200),
	description: z.string().optional(),
	createdAt: z.coerce.date(),
	updatedAt: z.coerce.date(),
});

type Item = z.infer<typeof itemSchema>;

// Create input (no id, no timestamps)
const createItemSchema = itemSchema.omit({ id: true, createdAt: true, updatedAt: true });

// Update input (all optional except id)
const updateItemSchema = itemSchema
	.omit({ createdAt: true, updatedAt: true })
	.partial()
	.required({ id: true });

// Summary view (list display)
const itemSummarySchema = itemSchema.pick({ id: true, name: true });
```

### Transform & Coerce

| Method              | When to Use                                              |
| ------------------- | -------------------------------------------------------- |
| `z.coerce.number()` | Convert string input to number (forms, search params)    |
| `z.coerce.date()`   | Convert string to Date                                   |
| `.transform(fn)`    | Reshape data after validation (trim, lowercase, compute) |
| `.pipe(schema)`     | Chain validation stages                                  |

**Coercion pitfall:** `z.coerce.number()` converts empty string `""` to `0`. For form inputs, transform empty strings to `undefined` before coercing.

### Critical Rule

**No `.extend()` after `.refine()` or `.transform()`** — once a schema has effects, structural methods are unavailable. Always apply structural modifications first:

```typescript
// CORRECT order
const schema = baseSchema.extend({ extra: z.string() }).omit({ internal: true }).refine(...)

// WRONG order — .extend() doesn't exist on ZodEffects
const broken = baseSchema.refine(...).extend({ extra: z.string() }) // ERROR
```

For discriminated union patterns with reducers, see [STATE.md § useReducer with TypeScript](STATE.md#usereducer-with-typescript).

---

## Schema Organization

### Directory Structure

**Feature-collocated (within single app):**

```
src/features/items/
  schemas/
    create-item.schema.ts
    update-item.schema.ts
  components/
  hooks/
```

**Component-collocated (simplest):**

```
src/features/items/
  ItemForm.tsx
  item-form.schema.ts
  ItemForm.test.tsx
```

### Naming Convention

```typescript
// camelCase for schema, PascalCase for type (recommended)
const userSchema = z.object({ ... })
type User = z.infer<typeof userSchema>
```

### Shared Sub-Schemas

```typescript
// src/schemas/common.ts
export const timestampsSchema = z.object({
	createdAt: z.coerce.date(),
	updatedAt: z.coerce.date(),
});

export const paginationSchema = z.object({
	page: z.number().int().positive().default(1),
	pageSize: z.number().int().positive().max(100).default(20),
});

// Usage
const itemListQuerySchema = paginationSchema.extend({
	search: z.string().optional(),
	status: z.enum(['active', 'archived']).optional(),
});
```

---

## Search Params

TanStack Router treats search params as first-class type-safe state validated via Zod.

### Basic Setup

```typescript
import { zodValidator, fallback } from '@tanstack/zod-adapter';

const searchSchema = z.object({
	page: fallback(z.number(), 1).default(1),
	sort: fallback(z.enum(['newest', 'price']), 'newest').default('newest'),
});

export const Route = createFileRoute('/products')({
	validateSearch: zodValidator(searchSchema),
});
```

### Key Distinctions

| Function                  | Provides Value When                      |
| ------------------------- | ---------------------------------------- |
| `fallback(schema, value)` | Validation **fails** (e.g., `?page=abc`) |
| `.default(value)`         | Param is **absent** from URL             |
| `.catch(value)`           | **Avoid in Zod v3** — causes type loss   |

### Zod v3 vs v4

| Zod Version         | Approach                                                       |
| ------------------- | -------------------------------------------------------------- |
| Zod 3.x             | Use `zodValidator` + `fallback` from `@tanstack/zod-adapter`   |
| Zod 4.x (>= v4.0.6) | Use `.catch().default()` directly — implements Standard Schema |

For full search params API (reading, writing, `loaderDeps`, `stripSearchParams`), see [ROUTING.md § Search Params](ROUTING.md#search-params).

---

## API Response Validation

### When to Validate

| Data Source                 | Validate?       | Why                                       |
| --------------------------- | --------------- | ----------------------------------------- |
| Third-party APIs            | **Yes, always** | Zero control over responses               |
| Own API (typed client)      | Usually no      | Compile-time types flow; server validates |
| `JSON.parse()`              | **Yes**         | Returns `any`                             |
| localStorage/sessionStorage | **Yes**         | Data can be stale or manually edited      |
| Component props             | No              | TypeScript suffices internally            |

### Parse in queryFn

```typescript
const userSchema = z.object({ id: z.number(), name: z.string() });

async function fetchUser(id: number) {
	const res = await fetch(`/api/users/${id}`);
	return userSchema.parse(await res.json()); // Throws ZodError if invalid
}

const useUser = (id: number) =>
	useQuery({
		queryKey: ['user', id],
		queryFn: () => fetchUser(id),
	});
```

If `.parse()` throws, TanStack Query catches it and puts the query in error state — preventing malformed data from reaching components.

For error handling utilities (`extractErrorMessage`, `isEdenError`), see [QUERY_EDEN.md § Error Handling](QUERY_EDEN.md#error-handling).

---

## Environment Variables

Create a validated source of truth at startup:

```typescript
// src/env.ts
import { z } from 'zod';

const envSchema = z.object({
	VITE_API_BASE_URL: z.string().url(),
	VITE_APP_TITLE: z.string().min(1),
	VITE_ENABLE_ANALYTICS: z.enum(['true', 'false']).transform(v => v === 'true'),
	VITE_MAX_RETRIES: z.coerce.number().min(0).default(3),
});

const _env = envSchema.safeParse(import.meta.env);

if (!_env.success) {
	console.error('Invalid environment variables:', _env.error.flatten().fieldErrors);
	throw new Error('Invalid environment variables');
}

export const ENV = _env.data;
```

Then import `ENV` everywhere instead of `import.meta.env`:

```typescript
import { ENV } from '@/env';

const response = await fetch(`${ENV.VITE_API_BASE_URL}/users`);
// ENV.VITE_ENABLE_ANALYTICS is boolean (not string)
// ENV.VITE_MAX_RETRIES is number (not string)
```

**Fail fast:** App throws on startup if env vars are invalid — no silent failures at runtime.

---

## Deep Dive

**Official Documentation:**

- [Zod Documentation — Basics](https://zod.dev/basics)
- [Zod Documentation — Error Handling](https://zod.dev/error-handling)
- [TanStack Router — Search Params](https://tanstack.com/router/latest/docs/framework/react/guide/search-params)

**Related Skill Files:**

| File                           | Focus                                               |
| ------------------------------ | --------------------------------------------------- |
| [REACT.md](REACT.md)           | Component patterns, hooks, performance, refactoring |
| [STATE.md](STATE.md)           | State management, discriminated unions for reducers |
| [ROUTING.md](ROUTING.md)       | TanStack Router, search params, data loading        |
| [QUERY_EDEN.md](QUERY_EDEN.md) | TanStack Query + Eden Treaty, error handling        |
| [STYLING.md](STYLING.md)       | Tailwind v4, CVA variants, cn() utility             |
| [TESTING.md](TESTING.md)       | Vitest + React Testing Library + Playwright         |
| [WORKFLOW.md](WORKFLOW.md)     | Project structure, conventions, checklists          |
