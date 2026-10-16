---
name: react-frontend-best-practices
description: React SPA best practices reference. Covers React 19 ref-as-prop use() Context Compiler component architecture compound components headless hooks container presentational performance code splitting lazy loading, TanStack Router file-based routing loaders beforeLoad search params navigation guards, TanStack Query v5 Eden Treaty queryOptions mutations cache invalidation optimistic updates SSE streaming, Zod validation form validation safeParse schema composition environment variables, Tailwind CSS v4 @theme @utility CVA variants cn utility dark mode responsive design container queries, testing Vitest React Testing Library RTL Playwright E2E component testing hook testing renderHook, development workflow project structure checklists imports exports.
---

# React Frontend Best Practices

Routing hub for React SPA development: components, hooks, routing, data fetching, validation, styling, testing, and workflow. All implementation detail lives in the reference files below — this file maps your need to the right one.

**When to Use:** Starting any frontend task involving React components, hooks, TanStack Router/Query, Zod validation, Tailwind styling, or testing. This file routes to the correct reference.

**Open the session with `PROJECT.md`**, beside this file, where the repo has one. It says where this repo keeps its UI (a component folder in the app, or a UI package of its own), what it uses in place of the defaults below (its styling helpers, its design system and how that is changed), and its commands. Where `PROJECT.md` and this skill disagree, `PROJECT.md` wins: it is the repo's, and this skill is shared by every repo that installs it.

---

## Decision Tree

Find your task below and go to the indicated reference file.

**React — components, hooks, React 19, performance:**

| I need to...                                                                | Go to                                         |
| --------------------------------------------------------------------------- | --------------------------------------------- |
| Use React 19 features (ref-as-prop, `use()`, Context as Provider)           | REACT.md § React 19 Features                  |
| Build component architecture (compound, headless, container/presentational) | REACT.md § Component Architecture             |
| Apply hooks best practices (derived state, functional setState, lazy init)  | REACT.md § Hooks Best Practices               |
| Optimize performance (lazy loading, memoization, content-visibility)        | REACT.md § Performance Patterns               |
| Check if a component is too complex or needs refactoring                    | REACT.md § Component Complexity & Refactoring |
| Avoid common React anti-patterns                                            | REACT.md § Common Anti-Patterns               |

**State Management — useState, useReducer, Context:**

| I need to...                                                          | Go to                                       |
| --------------------------------------------------------------------- | ------------------------------------------- |
| Decide between useState, useReducer, and Context                      | STATE.md § Decision Tree                    |
| Use useReducer with TypeScript (discriminated unions, typed dispatch) | STATE.md § useReducer with TypeScript       |
| Implement typed Context with React 19 `use()` API                     | STATE.md § Context Patterns                 |
| Split Context for performance (state vs dispatch)                     | STATE.md § Context Splitting                |
| Combine Context + useReducer for complex state                        | STATE.md § Context + useReducer Combination |
| Know when local state is sufficient                                   | STATE.md § When Local State Suffices        |

**Routing — TanStack Router, routes, loaders, guards:**

| I need to...                                   | Go to                                      |
| ---------------------------------------------- | ------------------------------------------ |
| Set up TanStack Router with Vite plugin        | ROUTING.md § Setup                         |
| Understand file-based route naming conventions | ROUTING.md § File-Based Routes             |
| Create type-safe router with `declare module`  | ROUTING.md § Router Creation & Type Safety |
| Load data in routes with loaders               | ROUTING.md § Data Loading                  |
| Add auth guards with `beforeLoad`              | ROUTING.md § Auth Guards (beforeLoad)      |
| Implement lazy routes and code splitting       | ROUTING.md § Lazy Routes                   |
| Navigate programmatically or with Link         | ROUTING.md § Navigation                    |
| Handle errors, pending states, or 404s         | ROUTING.md § Error / Pending / NotFound    |

**Data Fetching — TanStack Query, Eden Treaty, SSE:**

| I need to...                                      | Go to                                           |
| ------------------------------------------------- | ----------------------------------------------- |
| Set up Eden Treaty client                         | QUERY_EDEN.md § Eden Treaty Client Setup        |
| Handle Eden response shape (data/error narrowing) | QUERY_EDEN.md § Response Shape & Type Narrowing |
| Extract and display API errors                    | QUERY_EDEN.md § Error Handling                  |
| Create queryOptions factory for hooks and loaders | QUERY_EDEN.md § queryOptions Factory            |
| Define hierarchical query keys                    | QUERY_EDEN.md § Query Key Factory               |
| Write query hooks with dependent queries          | QUERY_EDEN.md § Query Hooks                     |
| Handle mutations and cache invalidation           | QUERY_EDEN.md § Mutations & Cache Invalidation  |
| Implement optimistic updates                      | QUERY_EDEN.md § Optimistic Updates              |
| Prefetch data in route loaders                    | QUERY_EDEN.md § Route Loader Integration        |
| Integrate SSE/streaming with query cache          | QUERY_EDEN.md § SSE / Streaming                 |

**Validation — Zod, forms, search params, environment variables:**

| I need to...                                     | Go to                                   |
| ------------------------------------------------ | --------------------------------------- |
| Define schemas and infer TypeScript types        | VALIDATION.md § Zod Fundamentals        |
| Validate forms with Zod (FormData or controlled) | VALIDATION.md § Form Validation         |
| Handle and display validation errors             | VALIDATION.md § Error Handling          |
| Compose schemas (extend, merge, pick, omit)      | VALIDATION.md § Schema Composition      |
| Organize schemas in the project                  | VALIDATION.md § Schema Organization     |
| Validate URL search params with zodValidator     | VALIDATION.md § Search Params           |
| Validate API responses at trust boundaries       | VALIDATION.md § API Response Validation |
| Validate environment variables at startup        | VALIDATION.md § Environment Variables   |

**Styling — Tailwind v4, CVA, cn(), theming:**

| I need to...                             | Go to                                    |
| ---------------------------------------- | ---------------------------------------- |
| Set up Tailwind v4 with CSS-first config | STYLING.md § CSS-First Configuration     |
| Define design tokens with @theme         | STYLING.md § Design Tokens (@theme)      |
| Create custom utilities with @utility    | STYLING.md § Custom Utilities (@utility) |
| Create component variants with CVA       | STYLING.md § CVA Variants                |
| Merge conditional classes with cn()      | STYLING.md § cn() Utility                |
| Implement dark mode                      | STYLING.md § Dark Mode                   |
| Choose between flex and grid layouts     | STYLING.md § Layout — Flex vs Grid       |
| Build responsive designs                 | STYLING.md § Responsive Design           |
| Add animations and transitions           | STYLING.md § Animations                  |

**Testing — Vitest, React Testing Library, Playwright:**

| I need to...                                 | Go to                                       |
| -------------------------------------------- | ------------------------------------------- |
| Configure Vitest for React testing           | TESTING.md § Setup                          |
| Write component tests with RTL               | TESTING.md § Component Testing Fundamentals |
| Use correct query priority (getByRole first) | TESTING.md § Query Priority                 |
| Avoid common testing mistakes                | TESTING.md § Common Mistakes                |
| Test custom hooks with renderHook            | TESTING.md § Hook Testing                   |
| Mock API calls with vi.mock                  | TESTING.md § API Mocking                    |
| Test router components                       | TESTING.md § Router Testing                 |
| Write E2E tests with Playwright              | TESTING.md § E2E with Playwright            |
| Know what to test vs skip                    | TESTING.md § What to Test                   |

**Workflow — structure, conventions, checklists:**

| I need to...                             | Go to                                  |
| ---------------------------------------- | -------------------------------------- |
| Understand project structure conventions | WORKFLOW.md § Project Structure        |
| Follow feature development flow          | WORKFLOW.md § Feature Development Flow |
| Add a new page                           | WORKFLOW.md § New Page Checklist       |
| Add a new component                      | WORKFLOW.md § New Component Checklist  |
| Add a new hook                           | WORKFLOW.md § New Hook Checklist       |
| Follow import/export rules               | WORKFLOW.md § Import/Export Rules      |

---

## Hard Rules

The defaults this skill holds every repo to. `PROJECT.md` may name the repo's own way for rule 3.

1. **ALWAYS** use `queryOptions` factory to share query config between hooks and route loaders.
2. **ALWAYS** check Eden response `error` before using `data` — the discriminated union requires narrowing.
3. **ALWAYS** use CVA for components with 2+ variant dimensions; use cn() for simpler conditional classes — unless `PROJECT.md` names the repo's own way.
4. **ALWAYS** prefer accessible queries (`getByRole`, `getByLabelText`) over `getByTestId` in component tests.
5. **NEVER** put business logic in route components — extract to custom hooks.
6. **NEVER** use Context for server state — use TanStack Query cache instead.

---

## Pattern Index

Alphabetical index of key patterns and APIs across all reference files.

| Pattern / API                       | Reference                                |
| ----------------------------------- | ---------------------------------------- |
| `<Context value={}>` (React 19)     | STATE.md § Context Patterns              |
| `@theme {}` (Tailwind v4)           | STYLING.md § Design Tokens (@theme)      |
| `@utility` (Tailwind v4)            | STYLING.md § Custom Utilities (@utility) |
| `beforeLoad` (auth guard)           | ROUTING.md § Auth Guards (beforeLoad)    |
| `cn()` (class merging)              | STYLING.md § cn() Utility                |
| `createFileRoute`                   | ROUTING.md § Route Options               |
| `createTestRouter`                  | TESTING.md § Router Testing              |
| `cva()` (variants)                  | STYLING.md § CVA Variants                |
| `ensureQueryData` / `prefetchQuery` | QUERY_EDEN.md § Route Loader Integration |
| `extractErrorMessage`               | QUERY_EDEN.md § Error Handling           |
| `queryOptions()`                    | QUERY_EDEN.md § queryOptions Factory     |
| `ref` as prop (React 19)            | REACT.md § React 19 Features             |
| `renderHook`                        | TESTING.md § Hook Testing                |
| `safeParse()`                       | VALIDATION.md § Zod Fundamentals         |
| `treaty<App>()`                     | QUERY_EDEN.md § Eden Treaty Client Setup |
| `use()` API (React 19)              | REACT.md § React 19 Features             |
| `useBlocker`                        | ROUTING.md § Navigation Blocking         |
| `useSuspenseQuery`                  | QUERY_EDEN.md § Query Hooks              |
| `zodValidator`                      | VALIDATION.md § Search Params            |

---

## Common Pitfalls

1. **Missing `error` check on Eden response** — Every Eden call returns `{ data, error }`. Access `data` only after confirming `!error`. See QUERY_EDEN.md § Response Shape.

2. **Component exceeds complexity thresholds** — 300+ lines, 7+ props, 15+ lines before JSX return, 3+ useState, or 2+ useEffect are signals to refactor. See REACT.md § Component Complexity.

3. **Context for everything** — Context is dependency injection, not global state. Use TanStack Query for server state. See STATE.md § Anti-Patterns.

4. **Stale closure in query keys** — Query keys must include all reactive values used in the query function. See QUERY_EDEN.md § Query Key Factory.

5. **Using `getByTestId` first** — Prefer `getByRole` → `getByLabelText` → `getByText` → `getByTestId`. See TESTING.md § Query Priority.

6. **Manual memoization with React Compiler** — React Compiler auto-memoizes. Remove manual `useMemo`/`useCallback`/`memo` when compiler is enabled. See REACT.md § React 19 Features.

7. **Missing `zodValidator` fallback** — Search params need default values via `fallback` or schema `.default()`. See VALIDATION.md § Search Params.

8. **Vite plugin order** — `@tanstack/router-plugin` and `@tailwindcss/vite` must come BEFORE `@vitejs/plugin-react`. See ROUTING.md § Setup, STYLING.md § CSS-First Configuration.

9. **Prop drilling vs premature Context** — 1-2 levels of prop drilling is fine. Add Context only when drilling exceeds 3 levels or causes pain. See STATE.md § Decision Tree.

10. **Schema `.extend()` after `.refine()`** — Zod loses refinements on extend. Call `.extend()` first, then `.refine()`. See VALIDATION.md § Schema Composition.

11. **Testing implementation details** — Test behavior (user sees X, clicks Y, expects Z), not internal state or method calls. See TESTING.md § What to Test.

12. **Missing loader `staleTime`** — Route loaders should set `staleTime` to avoid refetching on every navigation. See QUERY_EDEN.md § Route Loader Integration.

---

## Reference Files

| File          | Lines | Focus                                                      |
| ------------- | ----- | ---------------------------------------------------------- |
| REACT.md      | 499   | React 19, components, hooks, performance, refactoring      |
| STATE.md      | 385   | useState, useReducer, Context patterns, anti-patterns      |
| ROUTING.md    | 470   | TanStack Router setup, routes, loaders, guards, navigation |
| QUERY_EDEN.md | 492   | TanStack Query, Eden Treaty, mutations, cache, SSE         |
| VALIDATION.md | 479   | Zod, forms, search params, schemas, environment variables  |
| STYLING.md    | 487   | Tailwind v4, CVA, cn(), dark mode, responsive, animations  |
| TESTING.md    | 471   | Vitest, RTL, Playwright, component/hook/E2E testing        |
| WORKFLOW.md   | 247   | Project structure, conventions, checklists                 |

---

## External Documentation

**React:**

- [React 19 Blog Post](https://react.dev/blog/2024/12/05/react-19)
- [React Docs](https://react.dev)

**Routing:**

- [TanStack Router Docs](https://tanstack.com/router/latest/docs/framework/react/overview)
- [File-Based Routing](https://tanstack.com/router/latest/docs/framework/react/guide/file-based-routing)

**Data Fetching:**

- [TanStack Query Docs](https://tanstack.com/query/latest/docs/framework/react/overview)
- [Eden Treaty](https://elysiajs.com/eden/treaty/overview)

**Validation:**

- [Zod Documentation](https://zod.dev)

**Styling:**

- [Tailwind CSS v4](https://tailwindcss.com/docs)
- [CVA (class-variance-authority)](https://cva.style/docs)

**Testing:**

- [React Testing Library](https://testing-library.com/docs/react-testing-library/intro/)
- [Vitest](https://vitest.dev)
- [Playwright](https://playwright.dev/docs/intro)
