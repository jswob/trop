# TanStack Query v5 + Eden Treaty — Data Fetching Reference

Patterns for data fetching, mutations, cache management, and server-sent events using TanStack Query v5 with Eden Treaty as the type-safe HTTP client.

**When to Use:** Fetching server data, creating/updating/deleting resources, managing cache invalidation, integrating SSE streams, prefetching in route loaders.

**Prerequisite:** TanStack Query v5 + Eden Treaty. For route loader integration, see [ROUTING.md § Data Loading](ROUTING.md#data-loading). For state management decisions (query cache vs local state vs Context), see [STATE.md](STATE.md).

---

## Table of Contents

- [Eden Treaty Client Setup](#eden-treaty-client-setup)
- [Response Shape & Type Narrowing](#response-shape--type-narrowing)
- [Request Parameters](#request-parameters)
- [Error Handling](#error-handling)
- [queryOptions Factory](#queryoptions-factory)
- [Query Key Factory](#query-key-factory)
- [Query Hooks](#query-hooks)
- [Mutations & Cache Invalidation](#mutations--cache-invalidation)
- [Optimistic Updates](#optimistic-updates)
- [Route Loader Integration](#route-loader-integration)
- [SSE / Streaming](#sse--streaming)
- [Hook Organization](#hook-organization)
- [Deep Dive](#deep-dive)

---

## Eden Treaty Client Setup

- **`treaty<App>(url)`** — creates a type-safe client from your Elysia `App` type (no code generation)
- **Create synchronously at module level** — async creation causes runtime issues
- **Config options** customize fetch behavior, headers, and interceptors

```ts
// src/lib/api.ts
import { treaty } from '@elysiajs/eden';
import type { App } from 'your-api-package';

export const client = treaty<App>(import.meta.env.VITE_API_BASE_URL);
```

| Option       | Purpose                                                     |
| ------------ | ----------------------------------------------------------- |
| `fetch`      | Default fetch options (e.g., `{ credentials: 'include' }`)  |
| `headers`    | Static headers or function `(path, options) => HeadersInit` |
| `onRequest`  | Intercept before sending (add auth tokens, logging)         |
| `onResponse` | Intercept after receiving (global error handling)           |

**Header priority** (highest → lowest): inline method headers → config `headers` → config `fetch.headers`.

---

## Response Shape & Type Narrowing

Every Eden Treaty call returns a discriminated union:

```ts
const { data, error, response, status, headers } = await client.api.items.get();
```

| Property   | Type                                           | Description                                      |
| ---------- | ---------------------------------------------- | ------------------------------------------------ |
| `data`     | `T \| null`                                    | Response body for 2xx. `null` when error.        |
| `error`    | `{ status: number; value: ErrorType } \| null` | Error body for status >= 300. `null` on success. |
| `response` | `Response`                                     | Raw Web Standard Response                        |
| `status`   | `number`                                       | HTTP status code                                 |
| `headers`  | `Headers`                                      | Response headers                                 |

**Critical:** `data` is nullable until you check `error`. After `if (error) { ... }`, TypeScript narrows `data` from `T | null` to `T`.

---

## Request Parameters

### GET — query params

```ts
const { data } = await client.api.items.get({ query: { status: 'active', page: 1 } });
```

### POST/PUT/PATCH — body + options

```ts
// Body as first arg, options as second
const { data } = await client.api.items.post({ name: 'New Item' });
const { data } = await client.api.items.post({ name: 'Item' }, { query: { notify: true } });
```

### Path parameters — chaining

```ts
const { data } = await client.api.items({ id: itemId }).get();
const { data } = await client.api['item-categories']({ id }).get(); // bracket notation for hyphens
```

### AbortSignal

```ts
const controller = new AbortController();
const { data } = await client.api.items.get({ fetch: { signal: controller.signal } });
```

---

## Error Handling

- **`extractErrorMessage`** — normalize any error shape to a string
- **`isEdenError`** — type guard for Eden-shaped errors
- **Standard pattern:** check `error` → throw `Error` with extracted message → TanStack Query catches it

### Utility Functions

```ts
// src/lib/api-utils.ts
export function extractErrorMessage(error: unknown): string {
	if (error instanceof Error) return error.message;
	if (typeof error === 'string') return error;
	if (error && typeof error === 'object' && 'message' in error) return String(error.message);
	return 'An unknown error occurred';
}

export function isEdenError(error: unknown): error is { message: string; status?: number } {
	return Boolean(error && typeof error === 'object' && 'message' in error);
}
```

### Standard Error Pattern (used in all queryFn / mutationFn)

```ts
const { data, error } = await client.api.items.get();
if (error) throw new Error(extractErrorMessage(error.value));
return data; // type-narrowed to non-null
```

### Preserving Status for Retry Control (Optional)

```ts
class ApiError extends Error {
	constructor(
		message: string,
		public status: number
	) {
		super(message);
		this.name = 'ApiError';
	}
}

// In queryFn — enables status-aware retry logic in QueryClient config
if (error) throw new ApiError(extractErrorMessage(error.value), error.status);
```

---

## queryOptions Factory

The central pattern bridging hooks and loaders. `queryOptions()` co-locates `queryKey` and `queryFn` in a type-safe, reusable factory.

```ts
import { queryOptions } from '@tanstack/react-query';
import { client } from '@/lib/api';
import { extractErrorMessage } from '@/lib/api-utils';
import { itemKeys } from './keys';

export function itemDetailOptions(id: string) {
	return queryOptions({
		queryKey: itemKeys.detail(id),
		queryFn: async () => {
			const { data, error } = await client.api.items({ id }).get();
			if (error) throw new Error(extractErrorMessage(error.value));
			return data;
		},
	});
}

export function itemListOptions(filters?: { search?: string }) {
	return queryOptions({
		queryKey: itemKeys.list(filters),
		queryFn: async () => {
			const { data, error } = await client.api.items.get({ query: filters });
			if (error) throw new Error(extractErrorMessage(error.value));
			return data;
		},
	});
}
```

### Usage Across the App

```ts
useQuery(itemDetailOptions(id)); // Component
useSuspenseQuery(itemDetailOptions(id)); // Suspense component
queryClient.ensureQueryData(itemDetailOptions(id)); // Route loader
queryClient.setQueryData(itemDetailOptions(id).queryKey, item); // Direct cache write
queryClient.getQueryData(itemDetailOptions(id).queryKey); // Cache read
```

| Benefit                | Description                                          |
| ---------------------- | ---------------------------------------------------- |
| Single source of truth | queryKey + queryFn defined once, used everywhere     |
| Type-safe cache ops    | `getQueryData` / `setQueryData` return correct types |
| Loader reuse           | Same factory in route loaders and component hooks    |
| No key drift           | Keys always match between invalidation and fetching  |

---

## Query Key Factory

Hierarchical keys enabling granular cache invalidation. One factory per resource domain.

```ts
// src/hooks/api/keys.ts — one factory per resource domain
export const itemKeys = {
	all: ['items'] as const,
	lists: () => [...itemKeys.all, 'list'] as const,
	list: (filters?: { search?: string }) => [...itemKeys.lists(), filters] as const,
	details: () => [...itemKeys.all, 'detail'] as const,
	detail: (id: string) => [...itemKeys.details(), id] as const,
};
```

### Invalidation Granularity

```ts
queryClient.invalidateQueries({ queryKey: itemKeys.all }); // ALL item queries
queryClient.invalidateQueries({ queryKey: itemKeys.lists() }); // Only lists (not details)
queryClient.invalidateQueries({ queryKey: itemKeys.detail(id) }); // One specific detail
```

---

## Query Hooks

Thin wrappers around `queryOptions` factories. The hook adds component-specific options like `enabled`.

```ts
// src/hooks/api/useItems.ts
import { useQuery } from '@tanstack/react-query';
import { itemListOptions, itemDetailOptions } from './itemOptions';

export function useItems(filters?: { search?: string }) {
	return useQuery(itemListOptions(filters));
}

export function useItem(id: string) {
	return useQuery({ ...itemDetailOptions(id), enabled: !!id });
}
```

### Pattern Summary

1. Eden client call → destructure `{ data, error }`
2. Check `error` → `throw new Error(extractErrorMessage(error.value))`
3. Return `data` (type-narrowed to non-null)
4. Wrap in `queryOptions` factory for reusability
5. Export thin hook wrapper with optional `enabled` logic

### Dependent Queries

With `useQuery` — use `enabled` to gate on parent data:

```ts
const parent = useQuery(parentOptions(id));
const children = useQuery({ ...childOptions(id), enabled: !!parent.data });
```

With `useSuspenseQuery` — `enabled` is not supported. Use component composition instead: parent component fetches and passes data as props to child, which fetches its own dependent query. See [REACT.md § Component Architecture](REACT.md#component-architecture) for this pattern.

---

## Mutations & Cache Invalidation

### Standard Mutation

```ts
import { useMutation, useQueryClient } from '@tanstack/react-query';

export function useCreateItem() {
	const queryClient = useQueryClient();
	return useMutation({
		mutationFn: async (body: { name: string }) => {
			const { data, error } = await client.api.items.post(body);
			if (error) throw new Error(extractErrorMessage(error.value));
			return data;
		},
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: itemKeys.lists() });
		},
	});
}
```

### Direct Cache Update (Skip Refetch)

```ts
export function useUpdateItem() {
	const queryClient = useQueryClient();
	return useMutation({
		mutationFn: async ({ id, ...body }: { id: string; name: string }) => {
			const { data, error } = await client.api.items({ id }).patch(body);
			if (error) throw new Error(extractErrorMessage(error.value));
			return data;
		},
		onSuccess: updated => {
			queryClient.setQueryData(itemKeys.detail(updated.id), updated); // Cache the response
			queryClient.invalidateQueries({ queryKey: itemKeys.lists() }); // Refetch lists
		},
	});
}
```

### Invalidation Scope

| Action | Invalidate                                                      |
| ------ | --------------------------------------------------------------- |
| Create | `keys.lists()` — lists need the new item                        |
| Update | `keys.detail(id)` + `keys.lists()` — or use direct cache update |
| Delete | `keys.all` — both lists and details are stale                   |

---

## Optimistic Updates

Use for instant UI feedback (toggles, inline edits, reordering). For most CRUD, simple invalidation ([§ Mutations](#mutations--cache-invalidation)) is sufficient.

```ts
export function useToggleItemStatus() {
	const queryClient = useQueryClient();
	return useMutation({
		mutationFn: async ({ id, active }: { id: string; active: boolean }) => {
			const { data, error } = await client.api.items({ id }).patch({ active });
			if (error) throw new Error(extractErrorMessage(error.value));
			return data;
		},
		onMutate: async ({ id, active }) => {
			await queryClient.cancelQueries({ queryKey: itemKeys.detail(id) }); // 1. Cancel refetches
			const previous = queryClient.getQueryData(itemKeys.detail(id)); // 2. Snapshot
			queryClient.setQueryData(
				itemKeys.detail(id),
				(
					old // 3. Optimistic update
				) => (old ? { ...old, active } : old)
			);
			return { previous }; // 4. Rollback context
		},
		onError: (_err, { id }, context) => {
			if (context?.previous) queryClient.setQueryData(itemKeys.detail(id), context.previous);
		},
		onSettled: (_data, _error, { id }) => {
			queryClient.invalidateQueries({ queryKey: itemKeys.detail(id) }); // Always refetch
		},
	});
}
```

**When to use:** Toggle states, inline edits, drag-and-drop reordering (immediate feedback critical).
**When to avoid:** Complex entities with server-computed fields, creation (no ID yet).

---

## Route Loader Integration

`queryOptions` factories bridge TanStack Query and TanStack Router loaders. The router context provides `queryClient` — see [ROUTING.md § Router Creation](ROUTING.md#router-creation--type-safety).

### ensureQueryData — Block Until Ready

```ts
loader: async ({ context: { queryClient }, params: { itemId } }) => {
	await queryClient.ensureQueryData(itemDetailOptions(itemId));
};
```

### prefetchQuery — Non-Blocking (Hover Intent)

```ts
loader: async ({ context: { queryClient } }) => {
	queryClient.prefetchQuery(itemListOptions()); // fire-and-forget, never throws
};
```

### Parallel Prefetch in Loaders

```ts
loader: async ({ context: { queryClient }, params: { itemId } }) => {
	await Promise.allSettled([
		queryClient.ensureQueryData(itemDetailOptions(itemId)),
		queryClient.ensureQueryData(categoryListOptions()),
	]);
};
```

| Method            | Behavior                                         | Use Case                          |
| ----------------- | ------------------------------------------------ | --------------------------------- |
| `ensureQueryData` | Returns cached or fetches. Awaitable.            | Route loaders (blocks navigation) |
| `prefetchQuery`   | Fetches if stale. Never throws. Fire-and-forget. | Hover/intent preloading           |
| `fetchQuery`      | Always fetches, throws on error. Awaitable.      | Force-refresh scenarios           |

For `loaderDeps` bridging search params to query dependencies and full loader API, see [ROUTING.md § Data Loading](ROUTING.md#data-loading).

---

## SSE / Streaming

Eden Treaty's streaming support has known runtime issues. Use native `EventSource` with TanStack Query cache for reliable SSE.

### Pattern: EventSource + Query Cache

```ts
import { useEffect, useRef } from 'react';
import { useQueryClient } from '@tanstack/react-query';

export function useEventStream(resourceId: string | null) {
	const queryClient = useQueryClient();
	const esRef = useRef<EventSource | null>(null);

	useEffect(() => {
		if (!resourceId) return;
		const baseUrl = import.meta.env.VITE_API_BASE_URL;
		const es = new EventSource(`${baseUrl}/api/resources/${resourceId}/stream`);
		esRef.current = es;

		es.onmessage = evt => {
			const event = JSON.parse(evt.data as string) as StreamEvent;
			if (event.type === 'progress') {
				queryClient.setQueryData(['resources', resourceId, 'stream'], event.data);
			}
			if (event.type === 'complete' || event.type === 'error') {
				queryClient.invalidateQueries({ queryKey: ['resources', resourceId] });
				es.close();
			}
		};
		es.onerror = () => es.close();
		return () => es.close();
	}, [resourceId, queryClient]);

	return esRef;
}
```

### Simpler Alternative: SSE as Invalidation Trigger

When you only need to know that data changed (not track progress), SSE events just trigger refetches:

```ts
es.onmessage = () => {
	queryClient.invalidateQueries({ queryKey: resourceKeys.detail(id) });
	es.close();
};
```

For stable effect callbacks that read latest props without re-subscribing, see [REACT.md § useEffectEvent](REACT.md#useeffectevent).

---

## Hook Organization

```
src/hooks/api/
  keys.ts               All query key factories
  use[Resource].ts      queryOptions + query hooks + mutation hooks per resource
  index.ts              Barrel re-export
```

**Convention per file:** Export `queryOptions` factories first (for loader reuse), then query hooks, then mutation hooks. Import keys from `./keys.ts`.

```ts
// src/hooks/api/index.ts
export {
	useItems,
	useItem,
	useCreateItem,
	useUpdateItem,
	useDeleteItem,
	itemListOptions,
	itemDetailOptions,
} from './useItems';
export { useCategories, categoryListOptions } from './useCategories';
```

---

## Deep Dive

**Official Documentation:**

- [TanStack Query v5 — Query Options](https://tanstack.com/query/v5/docs/react/guides/query-options)
- [TanStack Query v5 — Optimistic Updates](https://tanstack.com/query/v5/docs/framework/react/guides/optimistic-updates)
- [TanStack Query v5 — Prefetching](https://tanstack.com/query/v5/docs/framework/react/guides/prefetching)
- [TanStack Query v5 — Dependent Queries](https://tanstack.com/query/v5/docs/framework/react/guides/dependent-queries)
- [Eden Treaty — Overview](https://elysiajs.com/eden/overview)
- [Eden Treaty — Parameters](https://elysiajs.com/eden/treaty/parameters)
- [Eden Treaty — Response](https://elysiajs.com/eden/treaty/response)

**TanStack Query v5 Naming Changes (from v4):** `cacheTime` → `gcTime`, `isLoading` (initial) → `isPending`, `isInitialLoading` → `isLoading`, `onSuccess`/`onError` on `useQuery` removed (use mutation callbacks or `useEffect`), `keepPreviousData` → `placeholderData: keepPreviousData` (import function), object-only syntax for `useQuery({ queryKey, queryFn, ...opts })`.

**Related Skill Files:**

| File                           | Focus                                               |
| ------------------------------ | --------------------------------------------------- |
| [REACT.md](REACT.md)           | Component patterns, hooks, performance, refactoring |
| [STATE.md](STATE.md)           | State management decisions, Context, useReducer     |
| [ROUTING.md](ROUTING.md)       | TanStack Router, route loaders, code splitting      |
| [VALIDATION.md](VALIDATION.md) | Zod forms, search params, API response validation   |
| [STYLING.md](STYLING.md)       | Tailwind v4, CVA variants, cn() utility             |
| [TESTING.md](TESTING.md)       | Vitest + React Testing Library, query hook testing  |
| [WORKFLOW.md](WORKFLOW.md)     | Project structure, conventions, checklists          |
