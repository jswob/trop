# React 19 — Component & Hooks Reference

Patterns for React 19 components, hooks, performance, and refactoring in a Vite SPA.

**When to Use:** Building or modifying React components, writing custom hooks, optimizing performance, refactoring.

**Prerequisite:** React 19 + TypeScript. For state management decisions (useState vs useReducer vs Context), see [STATE.md](STATE.md).

---

## Table of Contents

- [React 19 Features](#react-19-features)
- [Component Architecture](#component-architecture)
- [Hooks Best Practices](#hooks-best-practices)
- [Performance Patterns](#performance-patterns)
- [Component Complexity & Refactoring](#component-complexity--refactoring)
- [Common Anti-Patterns](#common-anti-patterns)
- [Deep Dive](#deep-dive)

---

## React 19 Features

- **ref as prop** — function components accept `ref` as a regular prop; `forwardRef` is deprecated
- **`use()` API** — reads Context conditionally (after early returns, inside if-blocks)
- **Context as Provider** — `<MyContext value={}>` replaces `<MyContext.Provider value={}>`
- **React Compiler** — auto-memoizes at build time; eliminates manual `useMemo`/`useCallback`/`memo`
- **`useEffectEvent`** — stable callback for effects that reads latest values without being a dependency
- **`<Activity>`** — preserves DOM/state when hiding components (experimental, React 19.2)

### ref as prop

```tsx
function TextInput({
	placeholder,
	ref,
}: {
	placeholder: string;
	ref?: React.Ref<HTMLInputElement>;
}) {
	return <input placeholder={placeholder} ref={ref} />;
}

// Compose refs when both internal and external refs are needed
function Input({ ref, ...props }: InputProps & { ref?: React.Ref<HTMLInputElement> }) {
	const internalRef = useRef<HTMLInputElement>(null);
	const mergedRef = useCallback(
		(node: HTMLInputElement | null) => {
			internalRef.current = node;
			if (typeof ref === 'function') ref(node);
			else if (ref) ref.current = node;
		},
		[ref]
	);
	return <input ref={mergedRef} {...props} />;
}
```

### use() for conditional Context

```tsx
import { use } from 'react';

function Heading({ children }: { children?: React.ReactNode }) {
	if (!children) return null;
	const theme = use(ThemeContext); // Works after early return — useContext cannot
	return <h1 style={{ color: theme.color }}>{children}</h1>;
}
```

For data fetching, prefer TanStack Query over `use()` with Promises. See [QUERY_EDEN.md](QUERY_EDEN.md).

### Context as Provider

```tsx
<ThemeContext value="dark">{children}</ThemeContext>
// Replaces deprecated: <ThemeContext.Provider value="dark">
```

See [STATE.md § Context Patterns](STATE.md#context-patterns) for typed Context with `use()` and value memoization.

### React Compiler Strategy

| Phase            | Action                                                                          |
| ---------------- | ------------------------------------------------------------------------------- |
| **Now**          | Write pure render functions; manual memoization only where profiling shows need |
| **When adopted** | Remove `useMemo`/`useCallback`/`memo`; compiler handles finer-grained tracking  |
| **Code style**   | No mutations in render; prefer derived computations over synced state           |

### useEffectEvent

Stable callback that reads latest props/state without being an effect dependency.

```tsx
function ChatRoom({ roomId, theme }: { roomId: string; theme: string }) {
	const onConnected = useEffectEvent(() => showNotification(`Connected`, theme));
	useEffect(() => {
		const conn = createConnection(roomId);
		conn.on('connected', onConnected);
		conn.connect();
		return () => conn.disconnect();
	}, [roomId]); // theme NOT in deps — read via useEffectEvent
}
```

**Rules:** Only call from inside effects. Don't pass to other components as props.

### Activity (Experimental)

`<Activity mode="hidden">` preserves component state and DOM when hiding. Useful for tab UIs where form state should survive tab switches. Effects clean up on hide, re-run on show.

---

## Component Architecture

Patterns ordered by frequency of use:

- **Container-Presentational** — hook extracts data/logic, component renders (most common)
- **Compound Components** — parent owns state via Context, children consume it
- **Headless Hook** — hook provides behavior, consumer provides all UI
- **Static JSX Hoisting** — hoist constant elements to module scope
- **CVA Variants** — multi-variant styling; see [STYLING.md § CVA Variants](STYLING.md#cva-variants)

### Container-Presentational (Hook + Component)

The default architecture. Hook handles data/logic; component handles rendering.

```tsx
// useItemList.ts — container logic
function useItemList(categoryId: string) {
	const query = useQuery({
		queryKey: ['items', categoryId],
		queryFn: () => api.items.list({ categoryId }),
	});
	const deleteMutation = useMutation({
		mutationFn: (id: string) => api.items.delete({ id }),
		onSuccess: () => queryClient.invalidateQueries({ queryKey: ['items', categoryId] }),
	});
	return { items: query.data ?? [], isLoading: query.isLoading, deleteItem: deleteMutation.mutate };
}

// ItemList.tsx — presentational
function ItemList({
	items,
	isLoading,
	onDelete,
}: {
	items: Item[];
	isLoading: boolean;
	onDelete: (id: string) => void;
}) {
	if (isLoading) return <Skeleton count={5} />;
	return (
		<ul>
			{items.map(item => (
				<ItemCard key={item.id} item={item} onDelete={() => onDelete(item.id)} />
			))}
		</ul>
	);
}
```

### Compound Components

Multi-part UI (tabs, modals, accordions) where sub-components share implicit state.

```tsx
import { createContext, use, useState, type ReactNode } from 'react';

type TabsCtx = { active: string; setActive: (id: string) => void };
const TabsContext = createContext<TabsCtx | null>(null);
function useTabs() {
	const ctx = use(TabsContext);
	if (!ctx) throw new Error('Tab components must be used within <Tabs>');
	return ctx;
}
function Tabs({ defaultTab, children }: { defaultTab: string; children: ReactNode }) {
	const [active, setActive] = useState(defaultTab);
	return <TabsContext value={{ active, setActive }}>{children}</TabsContext>;
}
function Tab({ id, children }: { id: string; children: ReactNode }) {
	const { active, setActive } = useTabs();
	return (
		<button role="tab" aria-selected={active === id} onClick={() => setActive(id)}>
			{children}
		</button>
	);
}
function Panel({ id, children }: { id: string; children: ReactNode }) {
	return useTabs().active === id ? <div role="tabpanel">{children}</div> : null;
}
Tabs.Tab = Tab;
Tabs.Panel = Panel;
```

### Headless Hook

Reusable behavior, no UI opinion. Consumer renders however it wants.

```tsx
function useSortable<T>(items: T[], defaultKey: keyof T & string) {
	const [sortKey, setSortKey] = useState(defaultKey);
	const [sortDir, setSortDir] = useState<'asc' | 'desc'>('asc');
	const sorted = useMemo(
		() =>
			[...items].sort((a, b) => {
				const cmp = a[sortKey] < b[sortKey] ? -1 : a[sortKey] > b[sortKey] ? 1 : 0;
				return sortDir === 'asc' ? cmp : -cmp;
			}),
		[items, sortKey, sortDir]
	);
	const toggleSort = (key: keyof T & string) => {
		if (key === sortKey) setSortDir(d => (d === 'asc' ? 'desc' : 'asc'));
		else {
			setSortKey(key);
			setSortDir('asc');
		}
	};
	return { sorted, sortKey, sortDir, toggleSort };
}
```

### Static JSX Hoisting

Hoist constant elements to module scope so React reuses the same reference.

```tsx
const searchIcon = (
	<svg viewBox="0 0 24 24" className="h-4 w-4">
		<path d="..." />
	</svg>
);
const emptyState = <div className="py-12 text-center text-gray-500">No results found</div>;

function SearchView({ results }: { results: Item[] }) {
	const [query, setQuery] = useState('');
	return (
		<div>
			{searchIcon}
			<input value={query} onChange={e => setQuery(e.target.value)} />
			{results.length === 0 ? emptyState : <ResultsList items={results} />}
		</div>
	);
}
```

### Pattern Selection

| Scenario                                      | Pattern                                                                      |
| --------------------------------------------- | ---------------------------------------------------------------------------- |
| Separate data logic from rendering            | Container-Presentational (hook + component)                                  |
| Multi-part UI with shared state               | Compound Components                                                          |
| Reusable behavior, flexible rendering         | Headless Hook                                                                |
| Visual variants with Tailwind                 | CVA — see [STYLING.md](STYLING.md)                                           |
| Static decorative JSX                         | Static JSX Hoisting                                                          |
| Expensive subtree, frequent parent re-renders | `React.memo` — see [Performance § Memoized Extraction](#memoized-extraction) |

---

## Hooks Best Practices

- **Derive during render** — never store computed values in state
- **Functional setState** — use `prev =>` form in async/event handlers
- **Lazy initialization** — pass a function to `useState` for expensive initial values
- **useTransition** — mark non-urgent updates to keep UI responsive
- **useRef for transient values** — frequent changes that don't need re-renders
- **Handlers over effects** — user-triggered logic belongs in event handlers, not effects
- **useEffectEvent** — stable callback in effects; reads latest values without re-subscribing
- **Narrow dependencies** — use primitives in dep arrays, not objects

### Derived State — Compute During Render

```tsx
// BAD — state + effect to sync derived value
const [filtered, setFiltered] = useState(items);
useEffect(() => {
	setFiltered(items.filter(i => i.name.includes(filter)));
}, [items, filter]);

// GOOD — compute during render (useMemo only if profiling shows it's expensive)
const filtered = items.filter(i => i.name.includes(filter));
```

### Functional setState

```tsx
// BAD — stale closure risk in async/setTimeout/callbacks
const increment = () => setCount(count + 1);
// GOOD — always reads latest
const increment = () => setCount(prev => prev + 1);
```

### Lazy State Initialization

Pass a function to `useState` for expensive initial values: `useState(() => parseMarkdown(content))` — the function runs only on mount, not every render.

### useTransition

```tsx
function SearchResults() {
	const [query, setQuery] = useState('');
	const [isPending, startTransition] = useTransition();
	const [results, setResults] = useState<Item[]>([]);
	function handleSearch(input: string) {
		setQuery(input); // urgent
		startTransition(() => setResults(expensiveFilter(allResults, input))); // non-urgent
	}
	return (
		<div>
			<input value={query} onChange={e => handleSearch(e.target.value)} />
			<div style={{ opacity: isPending ? 0.7 : 1 }}>
				<ResultsList results={results} />
			</div>
		</div>
	);
}
```

`useDeferredValue` is similar but for values from props where you can't wrap the update.

### useRef for Transient Values

```tsx
function useScrollPosition() {
	const scrollY = useRef(0);
	useEffect(() => {
		const onScroll = () => {
			scrollY.current = window.scrollY;
		};
		window.addEventListener('scroll', onScroll, { passive: true });
		return () => window.removeEventListener('scroll', onScroll);
	}, []);
	return scrollY;
}
```

### Event Handlers Over Effects

```tsx
// BAD — effect reacting to state change from a user action
useEffect(() => {
	if (submitted) sendAnalytics('submitted');
}, [submitted]);
// GOOD — logic in the handler that caused it
function handleSubmit() {
	sendAnalytics('submitted');
	submitForm();
}
```

If a side effect is caused by a user action, put it in the handler. If caused by the component being displayed or a value changing, use an effect.

### useEffectEvent in Effects

```tsx
function useSSE(url: string, onMessage: (data: unknown) => void) {
	const handleMessage = useEffectEvent((event: MessageEvent) =>
		onMessage(JSON.parse(event.data as string))
	);
	useEffect(() => {
		const source = new EventSource(url);
		source.onmessage = handleMessage;
		return () => source.close();
	}, [url]); // onMessage NOT in deps
}
```

### useMemo / useCallback Guidelines

| Use when                                                | Don't use when                                 |
| ------------------------------------------------------- | ---------------------------------------------- |
| Expensive computation (>1ms, verified by profiling)     | Simple primitive check (`status === "active"`) |
| Reference-stable value passed to `React.memo` children  | Value only used in JSX, not a dep              |
| Function used as effect dep or passed to memoized child | Template literal or string concat              |

Extract non-primitive defaults to module scope: `const DEFAULT_CONFIG = { pageSize: 10 };`

### Narrow Effect Dependencies

Use `[user.name]` not `[user]` — primitives prevent unnecessary effect re-runs.

> **React Compiler:** When adopted, manual `useMemo`/`useCallback`/`memo` become unnecessary. Write pure render functions and avoid mutations during render to benefit immediately.

---

## Performance Patterns

- **CRITICAL:** Waterfall elimination — parallelize independent async work
- **CRITICAL:** Bundle size — lazy-load heavy components, avoid barrel imports
- **MEDIUM:** Re-render optimization — derive state, memoize expensive subtrees, transitions
- **MEDIUM:** Rendering performance — content-visibility, explicit conditionals
- **LOW-MEDIUM:** JS optimizations — Map/Set lookups, immutable array methods

### Waterfall Elimination

```tsx
// BAD — sequential               | GOOD — parallel
const a = await getA();            | const [a, b, c] = await Promise.all([getA(), getB(), getC()]);
const b = await getB();            |
const c = await getC();            |
```

For TanStack Query, multiple `useQuery` hooks fire in parallel automatically. See [QUERY_EDEN.md](QUERY_EDEN.md).

### Lazy Loading + Preload on Intent

```tsx
const HeavyView = lazy(() => import('./features/HeavyView'));
function preloadHeavyView() {
	import('./features/HeavyView');
}

function NavLink() {
	return (
		<Link to="/heavy" onMouseEnter={preloadHeavyView} onFocus={preloadHeavyView}>
			Open
		</Link>
	);
}
// Wrap lazy components in <Suspense fallback={<Skeleton />}>
```

For route-level code splitting, see [ROUTING.md § Lazy Routes](ROUTING.md#lazy-routes).

### Avoid Barrel File Imports

```tsx
// BAD — may pull entire barrel     | GOOD — direct imports, better tree-shaking
import { Button } from "@/components"; | import { Button } from "@/components/ui/Button";
```

### CSS content-visibility

```css
.list-item {
	content-visibility: auto;
	contain-intrinsic-size: 0 120px;
}
```

Browser skips layout/paint for off-screen items. High impact for long lists.

### Explicit Conditional Rendering

```tsx
// BAD — renders "0" when count is 0  | GOOD — explicit boolean
{count && <Badge count={count} />}     | {count > 0 ? <Badge count={count} /> : null}
```

### Memoized Extraction

When a parent re-renders frequently but a subtree doesn't depend on the changing state:

```tsx
const MemoizedChart = memo(function ChartSection({ data }: { data: ChartData }) {
	return <ExpensiveChart data={data} />;
});
```

### JS Micro-Optimizations

- **Index Maps:** `const map = useMemo(() => new Map(items.map(i => [i.id, i])), [items]);` — O(1) lookup
- **Immutable methods:** `toSorted()`, `toReversed()`, `toSpliced()` — don't mutate (breaks React change detection)
- **Set for membership:** `new Set(ids).has(id)` is O(1) vs `ids.includes(id)` is O(n)

| Priority       | Key Patterns                                                             |
| -------------- | ------------------------------------------------------------------------ |
| **Critical**   | `Promise.all`, `React.lazy`, preload on hover                            |
| **Medium**     | Derive state, `useTransition`, `content-visibility`, memoized extraction |
| **Low-Medium** | Map/Set lookups, immutable array methods, hoist static JSX               |

---

## Component Complexity & Refactoring

Thresholds are guidelines backed by community consensus (30+ sources). Context matters more than numbers.

### Quick Reference

| Metric                 | Green         | Yellow     | Red               |
| ---------------------- | ------------- | ---------- | ----------------- |
| Total component lines  | <100          | 100-300    | 300+              |
| JSX return lines       | <20           | 20-50      | 50+               |
| Logic before return    | <5 lines      | 5-15 lines | 15+ lines         |
| Props (product code)   | 1-3           | 4-7        | 7+                |
| Props (design system)  | 1-7           | 8-15       | 15+               |
| Related useState calls | 1-2           | 3-4        | 5+ (extract hook) |
| Boolean state flags    | 1-2           | 3          | 3+ (use enum)     |
| useEffect calls        | 1 per concern | 2+ related | Multi-purpose     |
| Cyclomatic complexity  | 1-10          | 11-20      | 20+               |
| Conditional branches   | 1-2           | 3          | 4+                |

Page components naturally trend larger (200-400 lines). Design system components legitimately have many props.

### Decision Flowchart

1. **Causing actual problems?** (perf, reuse, testing, merge conflicts) — If no: stop
2. **Can't see it on one screen?** — Too big, proceed
3. **5+ lines of logic before `return`?** — Extract hook (Recipe B)
4. **JSX sections depending on distinct data?** — Extract sub-components (Recipe A)
5. **Props drilled through non-consuming layers?** — Composition or Context; see [STATE.md](STATE.md)
6. **Many config props for different sections?** — Compound component (Recipe D)
7. **Different JSX per branch?** — Separate component per variant
8. **File >300 lines after splitting?** — Co-located modules (Recipe C)

### Refactoring Recipes

**A. Extract Sub-Component** — JSX section that relies on a single data object, is a visual "box", or repeats (list items). Move to own component, pass data as a single prop.

**B. Extract Custom Hook** — 5+ lines before `return`, 3+ related state variables, or independently testable logic. Name `use[Purpose]`. Return object (not tuple for >2 values).

**C. Split File** — File >300 lines with component + hook + types. Split: `index.ts` (re-export), `Component.tsx`, `useComponent.ts`, `Component.types.ts`.

**D. Extract Compound Component** — Many config props controlling sections. Create Context + sub-components + dot notation. See [§ Compound Components](#compound-components).

### AHA Principle

> **Avoid Hasty Abstractions.** Duplication is cheaper than the wrong abstraction. Write similar code twice; on the third, consider abstracting — only if you understand how it should vary. — Sandi Metz / Kent C. Dodds

Avoid single-use wrappers that add indirection without improving testability or reuse. Exception: single-use hooks that cleanly separate logic from UI.

See [WORKFLOW.md](WORKFLOW.md) for co-location conventions when splitting components.

---

## Common Anti-Patterns

1. **Derived state in useState** — compute during render, not sync with effects
2. **forwardRef in new code** — use ref-as-prop (React 19)
3. **Barrel imports everywhere** — import directly from source modules
4. **`count && <Badge />`** — renders `0`; use `count > 0 ? ... : null`
5. **Manual fetch in useEffect** — use TanStack Query. See [QUERY_EDEN.md](QUERY_EDEN.md)
6. **Prop drilling 3+ levels** — use composition or Context. See [STATE.md](STATE.md)
7. **Over-memoization** — only where profiling shows need; Compiler handles the rest

---

## Deep Dive

**Official Documentation:**

- [React 19 Blog Post](https://react.dev/blog/2024/12/05/react-19)
- [React Hooks Reference](https://react.dev/reference/react/hooks)
- [React Compiler](https://react.dev/learn/react-compiler)

**Related Skill Files:**

| File                           | Focus                                               |
| ------------------------------ | --------------------------------------------------- |
| [STATE.md](STATE.md)           | State management decision tree, Context, useReducer |
| [ROUTING.md](ROUTING.md)       | TanStack Router, route loaders, code splitting      |
| [QUERY_EDEN.md](QUERY_EDEN.md) | TanStack Query + Eden Treaty data fetching          |
| [STYLING.md](STYLING.md)       | Tailwind v4, CVA variants, cn() utility             |
| [TESTING.md](TESTING.md)       | Vitest + React Testing Library + Playwright         |
| [VALIDATION.md](VALIDATION.md) | Zod forms, search params, API responses             |
| [WORKFLOW.md](WORKFLOW.md)     | Project structure, conventions, checklists          |
