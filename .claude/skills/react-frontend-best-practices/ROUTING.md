# TanStack Router — Routing Reference

Type-safe file-based routing for React SPAs with built-in data loading, code splitting, and search params.

**When to Use:** Setting up routing, adding pages/routes, configuring data loaders, auth guards, search params, code splitting, navigation.

**Prerequisite:** TanStack Router + Vite plugin. For search param validation, see [VALIDATION.md](VALIDATION.md). For data fetching in loaders, see [QUERY_EDEN.md](QUERY_EDEN.md).

---

## Table of Contents

- [Setup](#setup)
- [File-Based Routes](#file-based-routes)
- [Router Creation & Type Safety](#router-creation--type-safety)
- [Route Options](#route-options)
- [Data Loading](#data-loading)
- [Auth Guards (beforeLoad)](#auth-guards-beforeload)
- [Lazy Routes](#lazy-routes)
- [Navigation](#navigation)
- [Search Params](#search-params)
- [Error / Pending / NotFound](#error--pending--notfound)
- [Navigation Blocking](#navigation-blocking)
- [Scroll Restoration](#scroll-restoration)
- [Deep Dive](#deep-dive)

---

## Setup

- **Vite plugin** — `@tanstack/router-plugin` MUST be placed **before** `@vitejs/plugin-react`
- **Auto code splitting** — enable `autoCodeSplitting: true` to separate critical route config from lazy-loaded components
- **Route tree** — generated at build time into `routeTree.gen.ts`; add this file to linter, formatter, and search excludes
- **Routes directory** — default `src/routes/`; convention-over-configuration handles the rest

```ts
// vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { tanstackRouter } from '@tanstack/router-plugin/vite';

export default defineConfig({
	plugins: [
		tanstackRouter({ target: 'react', autoCodeSplitting: true }), // MUST be before react()
		react(),
	],
});
```

| Option                  | Default                  | Notes                               |
| ----------------------- | ------------------------ | ----------------------------------- |
| `routesDirectory`       | `./src/routes`           | Where route files live              |
| `generatedRouteTree`    | `./src/routeTree.gen.ts` | Auto-generated; do not edit         |
| `routeFileIgnorePrefix` | `"-"`                    | Files prefixed with `-` are ignored |
| `autoCodeSplitting`     | `false`                  | Set to `true` (recommended)         |

---

## File-Based Routes

- **Filesystem = route tree** — file structure converts to typed routes at build time
- **Flat (dot-separated) and nested (directory) styles** can be mixed; flat is simpler for small apps
- **`$param`** captures dynamic path segments; **`$`** is a catch-all
- **`_prefix`** creates pathless layout routes (wrap children without adding a URL segment)
- **`route.tsx`** lets a route file live inside a directory alongside related files

### File Naming Patterns

| Pattern       | Purpose                          | Example URL          |
| ------------- | -------------------------------- | -------------------- |
| `__root.tsx`  | Root layout wrapping all routes  | — (always renders)   |
| `index.tsx`   | Default child at path segment    | `/`                  |
| `about.tsx`   | Static route                     | `/about`             |
| `$param.tsx`  | Dynamic segment                  | `/posts/$postId`     |
| `_layout.tsx` | Pathless layout (no URL segment) | — (wrapper only)     |
| `route.tsx`   | Directory route config           | `/posts`             |
| `*.lazy.tsx`  | Code-split component             | — (loaded on demand) |
| `$.tsx`       | Catch-all (splat)                | `/files/*`           |

### Flat Route Tree

```
src/routes/
  __root.tsx
  index.tsx                   /
  about.tsx                   /about
  posts.index.tsx             /posts
  posts.$postId.tsx           /posts/:postId
  posts.$postId.edit.tsx      /posts/:postId/edit
```

### Pathless Layout Routes

Files/directories prefixed with `_` wrap children without adding a URL segment:

```
src/routes/
  _authenticated.tsx          Layout guard (no URL segment)
  _authenticated/
    dashboard.tsx             /dashboard
    profile.tsx               /profile
    settings.tsx              /settings
  login.tsx                   /login (public, outside layout)
```

---

## Router Creation & Type Safety

- **`createRouter()`** with the generated `routeTree` — single source of truth
- **Type registration** via `declare module` — enables compile-time route path checking in `<Link>`, `useNavigate`, and all route hooks
- **Router context** — pass dependencies (e.g., `queryClient`) available to all route loaders and `beforeLoad`
- **`RouterProvider`** replaces `<BrowserRouter>` — no wrapper component needed

```tsx
// src/router.ts
import { createRouter } from '@tanstack/react-router';
import type { QueryClient } from '@tanstack/react-query';
import { routeTree } from './routeTree.gen';

export interface RouterContext {
	queryClient: QueryClient;
}

export const router = createRouter({
	routeTree,
	context: { queryClient: undefined! }, // Provided at render time via RouterProvider
	defaultPreload: 'intent',
});

// CRITICAL: Without this, <Link to> and useNavigate lose type safety
declare module '@tanstack/react-router' {
	interface Register {
		router: typeof router;
	}
}
```

In your app entry point, render `<RouterProvider router={router} context={{ queryClient }} />`. No `<BrowserRouter>` wrapper needed.

---

## Route Options

- **`createFileRoute(path)`** — path string is auto-completed by the Vite plugin (type error if mismatched)
- **Typed route hooks** — `Route.useLoaderData()`, `Route.useSearch()`, `Route.useParams()` return fully typed data per route

```tsx
// src/routes/posts.$postId.tsx
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/posts/$postId')({
	loader: async ({ params }) => fetchPost(params.postId),
	component: () => {
		const post = Route.useLoaderData();
		return <h1>{post.title}</h1>;
	},
});
```

| Option               | Purpose                                             |
| -------------------- | --------------------------------------------------- |
| `component`          | Main route component                                |
| `loader`             | Data loading (runs before render)                   |
| `beforeLoad`         | Pre-load hook (auth guards, redirects)              |
| `validateSearch`     | Search params schema (Zod)                          |
| `loaderDeps`         | Extract deps from search/context for loader caching |
| `errorComponent`     | Error boundary                                      |
| `pendingComponent`   | Loading state                                       |
| `notFoundComponent`  | 404 component                                       |
| `staleTime`          | How long loader data stays fresh                    |
| `search.middlewares` | Search param middleware (retain/strip)              |

---

## Data Loading

- **Loaders run before render** — no loading spinners for pre-fetchable data
- **`loaderDeps`** extracts reactive dependencies from search params; loader re-runs when deps change
- **`staleTime` / `gcTime`** control caching — fresh data is not refetched
- **`abortController.signal`** cancels in-flight requests on navigation away
- **Execution order** — `beforeLoad` runs first (parent → child, sequential), then all matched route `loader`s run in parallel

```tsx
export const Route = createFileRoute('/posts')({
	validateSearch: z.object({
		page: z.number().int().positive().catch(1),
		filter: z.string().optional(),
	}),
	loaderDeps: ({ search }) => ({ page: search.page, filter: search.filter }),
	loader: async ({ deps, abortController }) =>
		fetchPosts(deps.page, deps.filter, { signal: abortController.signal }),
	staleTime: 5_000,
});
```

| Concept           | Description                                   |
| ----------------- | --------------------------------------------- |
| `loaderDeps`      | Reactive search/context → loader dependencies |
| `staleTime`       | Milliseconds data is fresh (skip refetch)     |
| `gcTime`          | Milliseconds stale data stays in memory       |
| `abortController` | Signal for cancelling requests on navigation  |

For TanStack Query integration in loaders (`ensureQueryData`, avoiding waterfalls with `Promise.allSettled`), see [QUERY_EDEN.md § Route Loader Integration](QUERY_EDEN.md#route-loader-integration).

---

## Auth Guards (beforeLoad)

- **`beforeLoad`** runs before the loader and all child routes — acts as middleware
- **Throw `redirect()`** to send unauthorized users to a login page
- **Pathless layout routes** guard multiple sibling routes at once (recommended over per-route guards)
- **Pass auth via router context** — React hooks can't be called outside components, so provide auth state through `RouterProvider context`

### Layout Guard (Recommended)

```tsx
// src/routes/_authenticated.tsx
import { createFileRoute, redirect, Outlet } from '@tanstack/react-router';

export const Route = createFileRoute('/_authenticated')({
	beforeLoad: ({ context, location }) => {
		if (!context.auth.isAuthenticated) {
			throw redirect({ to: '/login', search: { redirect: location.href } });
		}
	},
	component: () => <Outlet />,
});
// All files inside _authenticated/ are automatically protected
```

### Providing Auth Context

```tsx
// src/App.tsx — wrap RouterProvider to access auth hooks
function InnerApp() {
	const auth = useAuth();
	return <RouterProvider router={router} context={{ auth }} />;
}
```

### Post-Login Redirect Flow

1. User visits `/dashboard/settings`
2. `beforeLoad` detects no session → `throw redirect({ to: '/login', search: { redirect: '/dashboard/settings' } })`
3. Login page reads `search.redirect`
4. After successful login, navigate to `search.redirect`
5. User arrives at the originally requested page

---

## Lazy Routes

- **Automatic code splitting** (recommended) — enable `autoCodeSplitting: true` in the Vite plugin; no manual work needed
- **Critical code** (route matching, loaders, `beforeLoad`, `validateSearch`) loads immediately
- **Non-critical code** (component, errorComponent, pendingComponent, notFoundComponent) loads on demand
- **Manual split** via `.lazy.tsx` when you need fine-grained control over what's in each bundle
- **`__root.tsx` does NOT support code splitting** — it always renders

| Category         | Includes                                                       | When Loaded                             |
| ---------------- | -------------------------------------------------------------- | --------------------------------------- |
| **Critical**     | Route matching, loaders, beforeLoad, validateSearch            | Immediately (needed for routing + data) |
| **Non-Critical** | component, errorComponent, pendingComponent, notFoundComponent | On demand (when route matches)          |

### Manual Code Splitting

```tsx
// src/routes/posts.tsx — critical (always loaded)
export const Route = createFileRoute('/posts')({
	loader: fetchPosts,
});

// src/routes/posts.lazy.tsx — non-critical (loaded on demand)
import { createLazyFileRoute } from '@tanstack/react-router';

export const Route = createLazyFileRoute('/posts')({
	component: () => {
		const data = Route.useLoaderData();
		return <PostList posts={data} />;
	},
});
```

For component-level lazy loading with `React.lazy()` and preload-on-intent, see [REACT.md § Lazy Loading + Preload on Intent](REACT.md#lazy-loading--preload-on-intent).

---

## Navigation

- **`<Link>`** is fully type-safe — `to`, `params`, `search` are auto-completed per route
- **Active state** — `data-status="active"` attribute for CSS styling, or `activeProps`/`inactiveProps`
- **Preloading** — `preload="intent"` on `<Link>` (or globally via `defaultPreload`) preloads routes on hover/touch
- **`useNavigate()`** for programmatic navigation; **`Route.useNavigate()`** for route-scoped (preferred — better type inference)

```tsx
// Declarative navigation with type-safe params and search
<Link
	to="/posts/$postId"
	params={{ postId: '123' }}
	search={{ tab: 'comments' }}
	activeProps={{ className: 'font-bold' }}
	preload="intent"
>
	Post 123
</Link>
```

```tsx
// Programmatic navigation — prefer Route.useNavigate() for type safety
function PostControls() {
	const navigate = Route.useNavigate();
	const goToNext = () =>
		navigate({
			to: '.',
			search: prev => ({ ...prev, page: prev.page + 1 }),
		});
	return <button onClick={goToNext}>Next Page</button>;
}
```

---

## Search Params

TanStack Router treats search params as first-class type-safe state — validated via Zod, readable and writable with full type inference.

- **`validateSearch`** — attach a Zod schema to the route; invalid params get `.catch()` defaults
- **`Route.useSearch()`** — read validated, typed search params in components
- **Write** via `navigate({ search: (prev) => ({ ...prev, key: value }) })`
- **Middleware** — `retainSearchParams` / `stripSearchParams` control which params persist across navigations

```tsx
export const Route = createFileRoute('/products')({
	validateSearch: z.object({
		page: z.number().catch(1),
		sort: z.enum(['newest', 'price']).catch('newest'),
	}),
});

// Read
function ProductList() {
	const { page, sort } = Route.useSearch();
}

// Write
function SortButton() {
	const navigate = Route.useNavigate();
	return (
		<button onClick={() => navigate({ search: prev => ({ ...prev, sort: 'price' }) })}>
			By Price
		</button>
	);
}
```

For Zod schema patterns (`.catch()` vs `fallback`, complex schemas, error handling), see [VALIDATION.md § Search Params](VALIDATION.md#search-params). For `loaderDeps` bridging search params to TanStack Query loaders, see [QUERY_EDEN.md § Route Loader Integration](QUERY_EDEN.md#route-loader-integration).

---

## Error / Pending / NotFound

- **`errorComponent`** receives `{ error, reset }` — rendered when loaders or components throw
- **`pendingComponent`** with `pendingMs` / `pendingMinMs` — avoids flash-of-loading for fast navigations
- **`notFoundComponent`** — rendered for unmatched paths (automatic) or `throw notFound()` in loaders (resource not found)
- **Router-level defaults** apply to all routes; per-route options override them

```tsx
export const Route = createFileRoute('/posts/$postId')({
	loader: async ({ params }) => {
		const post = await fetchPost(params.postId);
		if (!post) throw notFound();
		return post;
	},
	errorComponent: ({ error, reset }) => (
		<div>
			<p>Error: {error.message}</p>
			<button onClick={reset}>Retry</button>
		</div>
	),
	pendingComponent: () => <Spinner />,
	notFoundComponent: () => <p>Post not found</p>,
});
```

```tsx
// Router-level defaults (apply to all routes)
const router = createRouter({
	routeTree,
	defaultErrorComponent: ({ error }) => <ErrorPage error={error} />,
	defaultPendingComponent: () => <Spinner />,
	defaultNotFoundComponent: () => <NotFoundPage />,
	defaultPendingMs: 200,
	defaultPendingMinMs: 500,
});
```

**Caveats:**

- For loader errors, prefer `router.invalidate()` over `reset` to coordinate both router reload and error boundary reset
- `notFoundComponent` requires the route to have children (and therefore an `<Outlet>`)

---

## Navigation Blocking

- **`useBlocker`** (experimental) — prevents navigation when the user has unsaved changes
- **`withResolver: true`** returns `{ proceed, reset, status }` for custom confirmation UI
- **`enableBeforeUnload`** adds browser-native tab close protection
- **`ignoreBlocker: true`** in navigate options bypasses all blockers

```tsx
function FormEditor() {
	const [isDirty, setIsDirty] = useState(false);
	const { proceed, reset, status } = useBlocker({
		shouldBlockFn: () => isDirty,
		withResolver: true,
		enableBeforeUnload: isDirty,
	});

	return (
		<>
			<form>{/* ... */}</form>
			{status === 'blocked' && (
				<dialog open>
					<p>You have unsaved changes. Leave anyway?</p>
					<button onClick={proceed}>Leave</button>
					<button onClick={reset}>Stay</button>
				</dialog>
			)}
		</>
	);
}
```

`shouldBlockFn` receives `{ current, next, action }` — `current`/`next` contain `routeId`, `pathname`, `params`, `search`; `action` is `'push' | 'replace' | 'pop'`.

---

## Scroll Restoration

Built-in for SPAs. Usually no configuration needed.

| Option                      | Default     | Description                                               |
| --------------------------- | ----------- | --------------------------------------------------------- |
| `scrollRestoration`         | `true`      | Enable/disable                                            |
| `getScrollRestorationKey`   | history key | Customize restoration key (e.g., `(loc) => loc.pathname`) |
| `scrollRestorationBehavior` | `'auto'`    | `'instant'` for no animation                              |
| `scrollToTopSelectors`      | `[]`        | Additional selectors to scroll to top                     |

Prevent scroll reset on specific navigations: `<Link resetScroll={false}>` or `navigate({ resetScroll: false })`.

**Gotcha:** If scroll restoration doesn't work, check for `height: 100%` or `100vh` on `html`/`body` in CSS — removing these usually fixes it.

---

## Deep Dive

**Official Documentation:**

- [TanStack Router Overview](https://tanstack.com/router/latest/docs/framework/react/overview)
- [File-Based Routing](https://tanstack.com/router/latest/docs/framework/react/routing/file-based-routing)
- [Data Loading](https://tanstack.com/router/latest/docs/framework/react/guide/data-loading)
- [Search Params](https://tanstack.com/router/latest/docs/framework/react/guide/search-params)
- [Code Splitting](https://tanstack.com/router/latest/docs/framework/react/guide/code-splitting)
- [Navigation](https://tanstack.com/router/latest/docs/framework/react/guide/navigation)
- [Authenticated Routes](https://tanstack.com/router/latest/docs/framework/react/guide/authenticated-routes)
- [Navigation Blocking](https://tanstack.com/router/latest/docs/framework/react/guide/navigation-blocking)
- [Not Found Errors](https://tanstack.com/router/latest/docs/framework/react/guide/not-found-errors)

**Related Skill Files:**

| File                           | Focus                                               |
| ------------------------------ | --------------------------------------------------- |
| [REACT.md](REACT.md)           | Component patterns, hooks, performance, refactoring |
| [STATE.md](STATE.md)           | State management decision tree, Context, useReducer |
| [QUERY_EDEN.md](QUERY_EDEN.md) | TanStack Query + Eden Treaty data fetching          |
| [VALIDATION.md](VALIDATION.md) | Zod forms, search params, API responses             |
| [STYLING.md](STYLING.md)       | Tailwind v4, CVA variants, cn() utility             |
| [TESTING.md](TESTING.md)       | Vitest + React Testing Library + Playwright         |
| [WORKFLOW.md](WORKFLOW.md)     | Project structure, conventions, checklists          |
