# State Management — Reference

Patterns for choosing between useState, useReducer, and Context in React 19 applications.

**When to Use:** Deciding where to put state, choosing between state primitives, implementing Context patterns, optimizing Context performance.

**Prerequisite:** For server state (data from APIs), use TanStack Query cache — see [QUERY_EDEN.md](QUERY_EDEN.md). This file covers client-side UI state only.

---

## Table of Contents

- [Decision Tree](#decision-tree)
- [useReducer with TypeScript](#usereducer-with-typescript)
- [Context Patterns](#context-patterns)
- [Context Splitting](#context-splitting)
- [Context + useReducer Combination](#context--usereducer-combination)
- [When Local State Suffices](#when-local-state-suffices)
- [Anti-Patterns](#anti-patterns)
- [Deep Dive](#deep-dive)

---

## Decision Tree

Choose the simplest solution that meets your needs. Each level adds complexity.

### Level 1: Local `useState`

**Use when:**

- State is a single primitive value (string, number, boolean)
- State is independent — no other state depends on it for updates
- State is used by only one component (or parent + immediate children)
- Updates are simple assignments, not complex transitions

**Examples:** form inputs, toggles, modals, loading spinners, hover/focus states.

### Level 2: Lift State Up

**Use when:**

- Two or more sibling components need the same state
- A parent needs to coordinate children based on shared state
- Prop drilling depth is 1–2 levels (still manageable)

**Do NOT lift prematurely:** If only one component uses the state today, keep it local. Lifting introduces coupling and re-renders in the parent.

### Level 3: `useReducer`

**Switch when:**

- Multiple state values are interdependent (updating one requires reading another)
- State transitions are complex (multi-step, conditional logic)
- You want centralized, testable business logic in a pure function
- State depends on previous state for the update

> "When one element of your state relies on the value of another element of your state in order to update: `useReducer`." — Kent C. Dodds

### Level 4: Context

**Add when:**

- Prop drilling exceeds 2–3 levels
- Many components across different subtrees need the same data
- The data is "environmental" (theme, locale, auth, feature flags)
- Update frequency is low to moderate

**Critical concept:** Context is NOT state management. It is a dependency injection / transport mechanism. The actual state management is done by `useState` or `useReducer`. Context just delivers the value downward.

### Level 5: Context + `useReducer`

**Combine when:**

- You need structured state updates (reducer) available to many components (context)
- The state has complex transitions AND is consumed deep in the tree
- You want a centralized state + dispatch pattern without external libraries
- Practical limit: 2–3 context-reducer pairs per app. Beyond that, consider a dedicated state library.

---

## useReducer with TypeScript

### Discriminated Union Actions

```typescript
interface State {
	count: number;
	status: 'idle' | 'loading' | 'error';
}

type Action =
	| { type: 'reset' }
	| { type: 'increment' }
	| { type: 'setStatus'; status: State['status'] };

function reducer(state: State, action: Action): State {
	switch (action.type) {
		case 'reset':
			return { count: 0, status: 'idle' };
		case 'increment':
			return { ...state, count: state.count + 1 };
		case 'setStatus':
			return { ...state, status: action.status };
		default: {
			const _exhaustive: never = action;
			throw new Error(`Unknown action: ${JSON.stringify(_exhaustive)}`);
		}
	}
}
```

The `default` case with `never` type ensures TypeScript errors if you add an action type but forget its handler.

### React 19 Typing Changes

| Approach                      | Example                                              | Notes                     |
| ----------------------------- | ---------------------------------------------------- | ------------------------- |
| Contextual typing (preferred) | `useReducer(reducer, initialState)`                  | Let TS infer from reducer |
| Explicit tuple                | `useReducer<State, [Action]>(reducer, initialState)` | When inference fails      |
| Annotate reducer params       | `useReducer((s: State, a: Action) => s, init)`       | Inline reducers           |

**Breaking change:** `useReducer<React.Reducer<S, A>>(...)` no longer works in React 19.

### Event-Driven vs Setter-Style Actions

Name actions after **events** (what happened), not **setters** (what to do). This keeps business logic in the reducer:

```typescript
// PREFERRED: Event-driven
type Action =
	| { type: 'userLoggedIn'; user: User }
	| { type: 'formSubmitted'; values: FormValues }
	| { type: 'itemToggled'; id: string };

// AVOID: Setter-style (logic leaks into components)
type Action =
	| { type: 'setUser'; user: User | null }
	| { type: 'setFormValues'; values: FormValues };
```

For simple state updates without complex transitions, prefer `useState` with functional updates. See [REACT.md § Functional setState](REACT.md#functional-setstate).

---

## Context Patterns

### Context as Provider (React 19)

React 19 allows `<Context value={}>` directly, replacing `<Context.Provider value={}>`:

```tsx
import { createContext } from 'react';

const ThemeContext = createContext('light');

function App({ children }: { children: React.ReactNode }) {
	return <ThemeContext value="dark">{children}</ThemeContext>;
}
```

### use() API — Conditional Context Reads

React 19's `use()` replaces `useContext()` with added flexibility — it can be called conditionally:

```tsx
import { use } from 'react';

function Panel({ show }: { show: boolean }) {
	if (!show) return null;
	const theme = use(ThemeContext); // Valid after early return
	return <div className={`panel-${theme}`}>Content</div>;
}
```

| Feature             | `use()`                  | `useContext()`         |
| ------------------- | ------------------------ | ---------------------- |
| Conditional calls   | Yes (inside `if`, `for`) | No (must be top-level) |
| After early returns | Yes                      | No                     |
| Promise support     | Yes (with Suspense)      | No                     |
| Try-catch           | NOT allowed              | N/A                    |

### Typed Context with Null-Check Hook

```typescript
import { createContext, useContext } from 'react';

type Theme = 'light' | 'dark';

const ThemeContext = createContext<Theme | undefined>(undefined);

function useTheme(): Theme {
	const context = useContext(ThemeContext);
	if (context === undefined) {
		throw new Error('useTheme must be used within a ThemeProvider');
	}
	return context;
}
```

**Always export a custom hook, never Context directly.** The hook encapsulates the null check and provides clear error messages. Passing `{} as SomeType` as a default hides bugs — use `undefined` and throw.

### Value Memoization

When the provider re-renders, all consumers re-render if the context value is a new reference. Memoize object values:

```tsx
const value = useMemo(() => ({ user, login, logout }), [user, login, logout]);
return <AuthContext value={value}>{children}</AuthContext>;
```

For a full compound components example using Context, see [REACT.md § Compound Components](REACT.md#compound-components).

---

## Context Splitting

### State vs Dispatch Separation

The `dispatch` function from `useReducer` has a stable identity — it never changes. Placing it in its own context prevents re-renders for components that only dispatch:

```tsx
const TasksContext = createContext<Task[] | null>(null);
const TasksDispatchContext = createContext<React.Dispatch<Action> | null>(null);

function TasksProvider({ children }: { children: React.ReactNode }) {
	const [tasks, dispatch] = useReducer(tasksReducer, initialTasks);
	return (
		<TasksContext value={tasks}>
			<TasksDispatchContext value={dispatch}>{children}</TasksDispatchContext>
		</TasksContext>
	);
}

function useTasks() {
	const ctx = useContext(TasksContext);
	if (!ctx) throw new Error('useTasks must be used within TasksProvider');
	return ctx;
}

function useTasksDispatch() {
	const ctx = useContext(TasksDispatchContext);
	if (!ctx) throw new Error('useTasksDispatch must be used within TasksProvider');
	return ctx;
}
```

**Result:** A component calling only `useTasksDispatch()` (e.g., an "Add" button) will NOT re-render when the tasks list changes.

### Frequency-Based Splitting

When different parts of state update at different frequencies, split them into individual contexts. Example: form with name field (every keystroke) + country selector (rarely changes) + discount slider (medium frequency). Each in its own context means components only re-render when their specific slice changes.

### Domain-Based Splitting

Separate unrelated concerns into independent context-reducer pairs: `AuthContext`, `ThemeContext`, `FeatureFlagsContext`. Don't put everything in one global context.

---

## Context + useReducer Combination

Complete pattern for structured state shared across components:

```tsx
// TasksContext.tsx
import { createContext, useContext, useReducer, type ReactNode } from 'react';

// Types
interface Task {
	id: string;
	text: string;
	done: boolean;
}
type Action =
	| { type: 'added'; text: string }
	| { type: 'toggled'; id: string }
	| { type: 'deleted'; id: string };

// Contexts (null for type safety)
const TasksContext = createContext<Task[] | null>(null);
const TasksDispatchContext = createContext<React.Dispatch<Action> | null>(null);

// Reducer
let nextId = 0;
function tasksReducer(tasks: Task[], action: Action): Task[] {
	switch (action.type) {
		case 'added':
			return [...tasks, { id: String(nextId++), text: action.text, done: false }];
		case 'toggled':
			return tasks.map(t => (t.id === action.id ? { ...t, done: !t.done } : t));
		case 'deleted':
			return tasks.filter(t => t.id !== action.id);
		default: {
			const _exhaustive: never = action;
			throw new Error(`Unknown action: ${JSON.stringify(_exhaustive)}`);
		}
	}
}

// Provider
export function TasksProvider({ children }: { children: ReactNode }) {
	const [tasks, dispatch] = useReducer(tasksReducer, []);
	return (
		<TasksContext value={tasks}>
			<TasksDispatchContext value={dispatch}>{children}</TasksDispatchContext>
		</TasksContext>
	);
}

// Hooks
export function useTasks(): Task[] {
	const ctx = useContext(TasksContext);
	if (!ctx) throw new Error('useTasks must be used within TasksProvider');
	return ctx;
}

export function useTasksDispatch(): React.Dispatch<Action> {
	const ctx = useContext(TasksDispatchContext);
	if (!ctx) throw new Error('useTasksDispatch must be used within TasksProvider');
	return ctx;
}
```

### Usage

```tsx
// App.tsx — wrap at composition root
<TasksProvider>
	<TaskList />
	<AddTaskButton />
</TasksProvider>;

// Components consume via hooks
function AddTaskButton() {
	const dispatch = useTasksDispatch(); // Only dispatch, won't re-render on task changes
	return <button onClick={() => dispatch({ type: 'added', text: 'New task' })}>Add</button>;
}
```

---

## When Local State Suffices

Local `useState` is the correct choice for:

- **Form inputs** — controlled component values (`[value, setValue]`)
- **UI toggles** — modal visibility, dropdown expanded, sidebar collapsed
- **Transient state** — loading spinners, submission progress
- **Animation state** — transition flags, CSS class toggles
- **Ephemeral interaction** — hover effects, focus rings, tooltip visibility

**Key principle:** If the state disappears when the component unmounts and no other component ever needs it, it belongs in local `useState`.

---

## Anti-Patterns

1. **"Context for everything" trap** — Putting all state in Context forces every consumer to re-render on every change. You may be "reinventing React-Redux, poorly."

2. **Global form state in Context** — Form fields update on every keystroke. In Context, this causes all consumers to re-render per keystroke. Keep form state local.

3. **Premature state lifting** — Don't lift state "just in case." Start local. Lift only when a second consumer actually appears.

4. **Derived state stored in Context** — Never store computed values. Compute during render instead: `const filtered = items.filter(...)`.

5. **Over-splitting contexts** — Creating a context per field is overkill. Split by domain first, then by update frequency only when profiling shows actual problems.

6. **Setter-style reducer actions** — `dispatch({ type: 'setUser' })` pushes logic into components. Use event-driven names: `dispatch({ type: 'userLoggedOut' })`.

7. **Exporting Context directly** — Forces consumers to `useContext(MyContext)` without null checks. Always export a custom hook with error handling.

8. **Meaningless defaults** — `createContext({} as SomeType)` hides bugs. Use `undefined`, throw in the hook.

---

## Deep Dive

**Official Documentation:**

- [React — Scaling Up with Reducer and Context](https://react.dev/learn/scaling-up-with-reducer-and-context)
- [React — useReducer](https://react.dev/reference/react/useReducer)
- [React — use() API](https://react.dev/reference/react/use)
- [React 19 Blog](https://react.dev/blog/2024/12/05/react-19)

**Related Skill Files:**

| File                           | Focus                                               |
| ------------------------------ | --------------------------------------------------- |
| [REACT.md](REACT.md)           | Component patterns, hooks, performance, refactoring |
| [ROUTING.md](ROUTING.md)       | TanStack Router, route loaders, code splitting      |
| [QUERY_EDEN.md](QUERY_EDEN.md) | TanStack Query + Eden Treaty (server state)         |
| [VALIDATION.md](VALIDATION.md) | Zod forms, search params, API responses             |
| [STYLING.md](STYLING.md)       | Tailwind v4, CVA variants, cn() utility             |
| [TESTING.md](TESTING.md)       | Vitest + React Testing Library + Playwright         |
| [WORKFLOW.md](WORKFLOW.md)     | Project structure, conventions, checklists          |
