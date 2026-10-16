# Frontend Testing — Reference

Patterns for component testing, hook testing, and E2E testing using Vitest, React Testing Library, and Playwright.

**When to Use:** Writing tests for React components, custom hooks, TanStack Query hooks, route components, or end-to-end user flows.

**Prerequisite:** Vitest + React Testing Library for unit/component tests. Playwright for E2E. For query hook implementation, see [QUERY_EDEN.md](QUERY_EDEN.md). For router patterns, see [ROUTING.md](ROUTING.md).

---

## Table of Contents

- [Setup](#setup)
- [Component Testing Fundamentals](#component-testing-fundamentals)
- [Query Priority](#query-priority)
- [Common Mistakes](#common-mistakes)
- [Hook Testing](#hook-testing)
- [API Mocking](#api-mocking)
- [Router Testing](#router-testing)
- [E2E with Playwright](#e2e-with-playwright)
- [What to Test](#what-to-test)
- [Test Isolation](#test-isolation)
- [Deep Dive](#deep-dive)

---

## Setup

- **Vitest config** — `environment: 'jsdom'`, `globals: true`, `setupFiles` array
- **Setup file** — imports jest-dom matchers, runs cleanup, polyfills browser APIs
- **Exclude E2E** — add `exclude: ['**/e2e/**']` so Playwright tests don't run in Vitest

### Vitest Config Pattern

```ts
// vitest.config.ts
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

export default defineConfig({
	plugins: [react()],
	test: {
		environment: 'jsdom',
		globals: true,
		setupFiles: ['./src/test/setup.ts'],
		exclude: ['**/node_modules/**', '**/e2e/**'],
	},
});
```

### Setup File Responsibilities

| Task                                 | Purpose                                                         |
| ------------------------------------ | --------------------------------------------------------------- |
| `import '@testing-library/jest-dom'` | DOM matchers (`toBeInTheDocument`, `toBeDisabled`, etc.)        |
| `afterEach(() => cleanup())`         | Unmount React trees between tests                               |
| Browser API mocks                    | Polyfill `matchMedia`, `ResizeObserver`, `scrollIntoView`, etc. |

**Note:** Required browser API mocks vary by libraries used. Check console errors during test runs and add mocks as needed.

---

## Component Testing Fundamentals

- **AAA pattern** — Arrange (setup), Act (interact), Assert (verify)
- **`screen`** — access queries without destructuring from render
- **`userEvent.setup()`** — always call before interactions (simulates real browser behavior)
- **`findBy*`** — for async elements (built on `waitFor`)
- **Custom render** — wrap components in providers (QueryClient, Router, Theme)

### AAA Pattern Example

```tsx
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

test('submits form with entered data', async () => {
	// ARRANGE
	const user = userEvent.setup();
	const handleSubmit = vi.fn();
	render(<UserForm onSubmit={handleSubmit} />);

	// ACT
	await user.type(screen.getByRole('textbox', { name: /name/i }), 'Alice');
	await user.click(screen.getByRole('button', { name: /submit/i }));

	// ASSERT
	expect(handleSubmit).toHaveBeenCalledWith({ name: 'Alice' });
});
```

### Custom Render with Providers

When components need providers (QueryClient, Router), create a custom render that wraps in `AllProviders`. See [Hook Testing § QueryClient Test Utilities](#queryclient-test-utilities) for the `createTestQueryClient` pattern, then:

```tsx
// src/test/render.tsx — re-exports RTL with custom wrapper
const customRender = (ui: React.ReactElement, options?: RenderOptions) =>
	render(ui, { wrapper: AllProviders, ...options });
export * from '@testing-library/react';
export { customRender as render };
```

---

## Query Priority

Use queries that reflect how users interact with the UI. Prefer higher-priority queries:

| Priority | Query                                    | When to use                             |
| -------- | ---------------------------------------- | --------------------------------------- |
| 1 (best) | `getByRole('button', { name: /save/i })` | **Default** — accessible elements       |
| 2        | `getByLabelText(/email/i)`               | Form fields with labels                 |
| 3        | `getByPlaceholderText(/search/i)`        | Input placeholders (no label)           |
| 4        | `getByText(/welcome/i)`                  | Non-interactive text                    |
| 5        | `getByDisplayValue('value')`             | Current value of form elements          |
| 6        | `getByAltText(/photo/i)`                 | Images with alt text                    |
| 7        | `getByTitle(/close/i)`                   | Elements with title attribute           |
| 8 (last) | `getByTestId('custom-element')`          | **Only when no accessible query works** |

### Anti-Patterns

- `container.querySelector('.my-class')` — bypasses accessibility, brittle on CSS changes
- `getByTestId` everywhere — misses accessibility issues, doesn't test what users see
- `getByRole('button')` without `name` — ambiguous when multiple buttons exist

---

## Common Mistakes

Based on Kent C. Dodds' guide to common RTL mistakes:

### 1. Not using `screen`

```tsx
// BAD — destructure grows as you add queries
const { getByRole } = render(<Component />);

// GOOD — screen always has all queries
render(<Component />);
screen.getByRole('button');
```

### 2. Using `query*` for existence checks

```tsx
// BAD — queryBy loses helpful error messages
expect(screen.queryByRole('button')).toBeTruthy();

// GOOD — getBy gives detailed error on failure
expect(screen.getByRole('button')).toBeInTheDocument();

// query* is ONLY for asserting non-existence
expect(screen.queryByRole('alert')).not.toBeInTheDocument();
```

### 3. Side effects inside `waitFor`

```tsx
// BAD — click should happen before waitFor
await waitFor(() => {
	fireEvent.click(button);
	expect(element).toBeVisible();
});

// GOOD — separate side effect from assertion
await user.click(button);
await waitFor(() => expect(element).toBeVisible());
```

### 4. Using `waitFor` instead of `findBy*`

```tsx
// WORSE — verbose
await waitFor(() => expect(screen.getByRole('heading')).toHaveTextContent('Welcome'));

// BETTER — findBy* is built on waitFor
expect(await screen.findByRole('heading')).toHaveTextContent('Welcome');
```

### 5. Wrapping render in `act()`

`render()` and `fireEvent` already wrap in `act()`. If you see warnings, investigate the root cause (unhandled state update) rather than adding `act()` wrappers.

### 6. Not using jest-dom matchers

```tsx
expect(button.disabled).toBe(true); // BAD — generic assertion
expect(button).toBeDisabled(); // GOOD — jest-dom, descriptive error
```

**Matchers:** `toBeInTheDocument`, `toBeVisible`, `toBeDisabled`, `toHaveTextContent`, `toHaveValue`, `toHaveClass`, `toBeChecked`, `toHaveFocus`

---

## Hook Testing

- **`renderHook`** — from `@testing-library/react` (not deprecated `@testing-library/react-hooks`)
- **Fresh QueryClient** — create new client per test to prevent cache leakage
- **`waitFor`** — poll until assertion passes (queries are async)

### QueryClient Test Utilities

```tsx
// src/test/query-wrapper.tsx
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import type { ReactNode } from 'react';

export function createTestQueryClient() {
	return new QueryClient({
		defaultOptions: {
			queries: { retry: false, gcTime: 0, staleTime: 0 },
			mutations: { retry: false },
		},
	});
}

export function createWrapper() {
	const queryClient = createTestQueryClient();
	return function TestQueryWrapper({ children }: { children: ReactNode }) {
		return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
	};
}
```

| Setting     | Value   | Reason                                                            |
| ----------- | ------- | ----------------------------------------------------------------- |
| `retry`     | `false` | Tests fail immediately on errors (no timeout waiting for retries) |
| `gcTime`    | `0`     | Garbage collect immediately (prevents "open handles" warnings)    |
| `staleTime` | `0`     | Tests control freshness via mocks                                 |

### Testing Query Hooks

```tsx
import { renderHook, waitFor } from '@testing-library/react';
import { useItems } from '@/hooks/api/useItems';
import { createWrapper } from '@/test/query-wrapper';

vi.mock('@/lib/api', () => ({
	client: { api: { items: { get: vi.fn() } } },
}));
import { client } from '@/lib/api';

test('returns data on success', async () => {
	vi.mocked(client.api.items.get).mockResolvedValue({
		data: [{ id: '1', name: 'Item' }],
		error: null,
		status: 200,
		response: new Response(),
		headers: new Headers(),
	});

	const { result } = renderHook(() => useItems(), { wrapper: createWrapper() });

	await waitFor(() => expect(result.current.isSuccess).toBe(true));
	expect(result.current.data).toEqual([{ id: '1', name: 'Item' }]);
});
```

### Testing Mutation Hooks

Same pattern as query hooks, but wrap `mutate()` in `act()`:

```tsx
const { result } = renderHook(() => useCreateItem(), { wrapper: createWrapper() });
act(() => {
	result.current.mutate({ name: 'New' });
});
await waitFor(() => expect(result.current.isSuccess).toBe(true));
```

---

## API Mocking

- **`vi.mock`** — mock at module level for type-safe HTTP clients
- **Response shape** — match your client's discriminated union structure
- **`vi.mocked()`** — preserves TypeScript types on mocked functions

### vi.mock Pattern

```tsx
vi.mock('@/lib/api', () => ({
	client: {
		api: {
			items: { get: vi.fn(), post: vi.fn() },
			users: { get: vi.fn() },
		},
	},
}));
import { client } from '@/lib/api';

// In tests, control responses:
vi.mocked(client.api.items.get).mockResolvedValue({
	data: [{ id: '1', name: 'Item' }],
	error: null,
	status: 200,
	response: new Response(),
	headers: new Headers(),
});
```

### Response Shape

Type-safe clients typically return a discriminated union:

| Property   | Success    | Error                                  |
| ---------- | ---------- | -------------------------------------- |
| `data`     | `T`        | `null`                                 |
| `error`    | `null`     | `{ status: number; value: ErrorBody }` |
| `status`   | `2xx`      | `4xx`/`5xx`                            |
| `response` | `Response` | `Response`                             |
| `headers`  | `Headers`  | `Headers`                              |

### When to Consider MSW

Mock Service Worker intercepts at the network level. Consider it when:

- Sharing mocks across tests and Storybook
- Testing the full request pipeline (interceptors, headers)
- Your HTTP client's response shape is complex to replicate

For most cases, `vi.mock` is simpler and sufficient.

---

## Router Testing

For components using router hooks (`useNavigate`, `useSearch`, `useParams`), create a test router. See [ROUTING.md](ROUTING.md) for full router patterns.

### createTestRouter Pattern

```tsx
import {
	createMemoryHistory,
	createRootRoute,
	createRoute,
	createRouter,
	RouterProvider,
} from '@tanstack/react-router';
import { render } from '@testing-library/react';

function renderWithRouter(component: React.ComponentType, options?: { initialEntries?: string[] }) {
	const rootRoute = createRootRoute({ component });
	const router = createRouter({
		routeTree: rootRoute,
		history: createMemoryHistory({ initialEntries: options?.initialEntries ?? ['/'] }),
		defaultPendingMinMs: 0, // CRITICAL — prevents artificial delays in tests
	});
	return render(<RouterProvider router={router} />);
}
```

**Key point:** Always set `defaultPendingMinMs: 0` — without this, tests wait for artificial pending delays.

---

## E2E with Playwright

- **Locators** — use accessible queries (`getByRole`, `getByLabel`) over test IDs
- **Auto-wait** — Playwright waits for elements automatically; avoid explicit timeouts
- **Page Object Model** — encapsulate selectors and actions for maintainability
- **Fixtures** — extend base test with custom setup (DB restoration, page objects)

### Preferred Locator Strategies

| Strategy      | Example                                       | Use when                     |
| ------------- | --------------------------------------------- | ---------------------------- |
| `getByRole`   | `page.getByRole('button', { name: /save/i })` | **Default** — accessible     |
| `getByLabel`  | `page.getByLabel('Email')`                    | Form fields with labels      |
| `getByText`   | `page.getByText('Welcome')`                   | Visible text                 |
| `getByTestId` | `page.getByTestId('submit')`                  | **Last resort** — dynamic UI |

### Basic Test Structure

```ts
import { test, expect } from '@playwright/test';

test('can view item list', async ({ page }) => {
	await page.goto('/items');
	await expect(page.getByRole('heading', { name: /items/i })).toBeVisible();
	await expect(page.getByTestId('item-list')).toHaveCount(3);
});
```

### Page Object Model Concept

Encapsulate page selectors and actions in a class for maintainability:

```ts
export class ItemsPage {
	constructor(private page: Page) {}
	readonly heading = this.page.getByRole('heading', { name: /items/i });
	readonly addButton = this.page.getByRole('button', { name: /add item/i });
	async goto() {
		await this.page.goto('/items');
	}
}
```

### Auto-Wait Behavior

Playwright waits automatically before actions. Avoid explicit waits:

```ts
// BAD — fragile timing
await page.waitForTimeout(2000);

// GOOD — wait for specific condition
await expect(page.getByRole('heading')).toBeVisible();
```

---

## What to Test

### Component Tests

| DO Test                                          | DON'T Test                              |
| ------------------------------------------------ | --------------------------------------- |
| User-visible behavior (click → modal shows)      | Implementation details (internal state) |
| Conditional rendering (loading → data → error)   | CSS class names                         |
| Form interactions (type → submit → callback)     | Third-party library internals           |
| Accessibility (buttons have names, labels exist) | Exact markup structure                  |
| Error states (API error → message displayed)     | Snapshot tests                          |

### Hook Tests

| DO Test                                       | DON'T Test                                         |
| --------------------------------------------- | -------------------------------------------------- |
| Query success/error states                    | TanStack Query internals (`isLoading` transitions) |
| Mutation triggers with correct data           | React Query caching behavior                       |
| Cache invalidation (related queries refetch)  | HTTP client request formatting                     |
| Data transformation (if hook transforms data) |                                                    |

### E2E Tests

| DO Test                                        | DON'T Test                             |
| ---------------------------------------------- | -------------------------------------- |
| Critical user journeys (login, create, submit) | Every edge case (use unit tests)       |
| Cross-page navigation                          | Styling/layout (use visual regression) |
| Data persistence (create → reload → visible)   | Third-party widget behavior            |
| Real API integration                           |                                        |

---

## Test Isolation

### Unit/Component Tests

| Mechanism         | How                                                                  |
| ----------------- | -------------------------------------------------------------------- |
| Fresh QueryClient | `createWrapper()` creates new client each call                       |
| Mock reset        | `vi.restoreAllMocks()` in `afterEach` or Vitest `restoreMocks: true` |
| DOM cleanup       | `cleanup()` in `afterEach` (or auto via RTL)                         |

### E2E Tests

| Mechanism             | How                                               |
| --------------------- | ------------------------------------------------- |
| Fresh browser context | Playwright creates new context per test           |
| Database restoration  | Custom fixture restores baseline before each test |
| Isolated storage      | Each test starts with clean localStorage/cookies  |

**Critical:** Never share QueryClient across tests — cached data from test A leaks into test B.

---

## Deep Dive

**Official Documentation:**

- [React Testing Library](https://testing-library.com/docs/react-testing-library/intro/)
- [Vitest Guide](https://vitest.dev/guide/)
- [Playwright Docs](https://playwright.dev/docs/intro)
- [TanStack Query — Testing](https://tanstack.com/query/latest/docs/react/guides/testing)
- [Common Mistakes with RTL](https://kentcdodds.com/blog/common-mistakes-with-react-testing-library)

**Related Skill Files:**

| File                           | Focus                                               |
| ------------------------------ | --------------------------------------------------- |
| [REACT.md](REACT.md)           | Component patterns, hooks, performance, refactoring |
| [STATE.md](STATE.md)           | State management decision tree, Context, useReducer |
| [ROUTING.md](ROUTING.md)       | TanStack Router, route loaders, navigation          |
| [QUERY_EDEN.md](QUERY_EDEN.md) | TanStack Query + Eden Treaty, hook organization     |
| [VALIDATION.md](VALIDATION.md) | Zod forms, search params, API responses             |
| [STYLING.md](STYLING.md)       | Tailwind v4, CVA variants, cn() utility             |
| [WORKFLOW.md](WORKFLOW.md)     | Project structure, conventions, checklists          |
