# Tailwind CSS v4 — Styling Reference

Patterns for CSS-first configuration, design tokens, component variants, and responsive layouts with Tailwind CSS v4.

**When to Use:** Styling components, defining design tokens, creating variant-based components, responsive design, dark mode, animations.

**Prerequisite:** Tailwind CSS v4 + Vite plugin. For component architecture patterns, see [REACT.md](REACT.md). For CSS performance (`content-visibility`), see [REACT.md § Performance](REACT.md#performance-patterns).

---

## Table of Contents

- [CSS-First Configuration](#css-first-configuration)
- [Design Tokens (@theme)](#design-tokens-theme)
- [Custom Utilities (@utility)](#custom-utilities-utility)
- [Custom Variants (@custom-variant)](#custom-variants-custom-variant)
- [CVA Variants](#cva-variants)
- [cn() Utility](#cn-utility)
- [Dark Mode](#dark-mode)
- [Layout — Flex vs Grid](#layout--flex-vs-grid)
- [Responsive Design](#responsive-design)
- [Container Queries](#container-queries)
- [Animations](#animations)
- [v3-to-v4 Migration](#v3-to-v4-migration)
- [Deep Dive](#deep-dive)

---

## CSS-First Configuration

Tailwind v4 eliminates `tailwind.config.js`. All customization happens in CSS.

```css
/* v4: Single import replaces three @tailwind directives */
@import 'tailwindcss';
```

```ts
// vite.config.ts — plugin order matters
import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
	plugins: [
		tailwindcss(), // MUST be before react()
		react(),
	],
});
```

| v3                                 | v4                                |
| ---------------------------------- | --------------------------------- |
| `tailwind.config.js`               | `@theme {}` in CSS                |
| `postcss.config.js` + autoprefixer | Not needed                        |
| `content` array for purging        | Automatic (respects `.gitignore`) |
| Three `@tailwind` directives       | Single `@import 'tailwindcss'`    |

---

## Design Tokens (@theme)

The `@theme` directive defines CSS variables that auto-generate utility classes.

```css
@theme {
	--color-brand: oklch(0.72 0.11 221);
	--color-surface: oklch(0.98 0 0);
	--font-display: 'Inter', sans-serif;
	--radius-card: 0.5rem;
}
/* Generates: bg-brand, text-brand, font-display, rounded-card */
```

### Common Namespaces

| Namespace        | Utilities              | Example                    |
| ---------------- | ---------------------- | -------------------------- |
| `--color-*`      | bg, text, border, ring | `bg-brand`, `text-surface` |
| `--font-*`       | font-family            | `font-display`             |
| `--text-*`       | font-size              | `text-xl`                  |
| `--spacing-*`    | p, m, gap, w, h        | `p-4`, `gap-2`             |
| `--radius-*`     | rounded                | `rounded-card`             |
| `--shadow-*`     | shadow                 | `shadow-lg`                |
| `--breakpoint-*` | responsive variants    | `md:flex`                  |
| `--ease-*`       | transition timing      | `ease-out`                 |
| `--animate-*`    | animation              | `animate-spin`             |

### Extending vs Overriding

```css
/* EXTEND — add to defaults */
@theme {
	--color-brand: oklch(0.6 0.2 250);
}

/* OVERRIDE — clear namespace first */
@theme {
	--color-*: initial; /* removes ALL default colors */
	--color-white: #fff;
	--color-brand: oklch(0.6 0.2 250);
}
```

### @theme inline

Use `inline` when theme variables reference other CSS variables:

```css
/* Correct — utility inlines the var() reference */
@theme inline {
	--color-app-bg: var(--surface-color);
}

/* Variables defined elsewhere (e.g., per-theme) */
:root {
	--surface-color: var(--color-white);
}
[data-theme='dark'] {
	--surface-color: var(--color-gray-900);
}
```

---

## Custom Utilities (@utility)

Define custom utilities that support all Tailwind variants.

```css
/* Simple utility */
@utility content-auto {
	content-visibility: auto;
}

/* Nested (pseudo-elements) */
@utility scrollbar-hidden {
	&::-webkit-scrollbar {
		display: none;
	}
}

/* Functional with theme values */
@theme {
	--tab-size-2: 2;
	--tab-size-4: 4;
}
@utility tab-* {
	tab-size: --value(--tab-size-*);
}
/* Usage: tab-2, tab-4 */
```

---

## Custom Variants (@custom-variant)

Define custom variants using CSS selectors.

```css
/* Full syntax */
@custom-variant theme-dark {
	&:where([data-theme='dark'] *) {
		@slot;
	}
}

/* Shorthand (no nesting needed) */
@custom-variant theme-dark (&:where([data-theme="dark"] *));

/* Multi-rule (media query) */
@custom-variant any-hover {
	@media (any-hover: hover) {
		&:hover {
			@slot;
		}
	}
}
```

Usage: `theme-dark:bg-gray-900`, `any-hover:underline`

---

## CVA Variants

[Class Variance Authority (CVA)](https://cva.style/docs) maps component props to Tailwind classes with full TypeScript inference.

### Basic Pattern

```tsx
import { cva, type VariantProps } from 'class-variance-authority';

const buttonVariants = cva(
	'inline-flex items-center justify-center rounded-sm font-medium transition-colors focus-visible:outline-hidden focus-visible:ring-2',
	{
		variants: {
			variant: {
				default: 'bg-slate-900 text-white hover:bg-slate-800',
				outline: 'border border-slate-300 bg-transparent hover:bg-slate-100',
				ghost: 'hover:bg-slate-100',
			},
			size: {
				sm: 'h-9 px-3 text-sm',
				md: 'h-10 px-4 text-sm',
				lg: 'h-11 px-6 text-base',
			},
		},
		defaultVariants: { variant: 'default', size: 'md' },
	}
);
```

### Component with VariantProps

```tsx
type ButtonProps = React.ComponentProps<'button'> & VariantProps<typeof buttonVariants>;

function Button({ className, variant, size, ...props }: ButtonProps) {
	return <button className={cn(buttonVariants({ variant, size }), className)} {...props} />;
}
// Usage: <Button variant="outline" size="lg">Click</Button>
```

### Compound Variants

Apply styles when multiple conditions align:

```tsx
const alertVariants = cva('rounded-sm border p-4', {
	variants: {
		intent: { info: 'border-blue-500', error: 'border-red-500' },
		prominent: { true: 'font-semibold', false: '' },
	},
	compoundVariants: [{ intent: 'error', prominent: true, class: 'bg-red-50 text-red-900' }],
	defaultVariants: { intent: 'info', prominent: false },
});
```

| Use Case                                     | Pattern      |
| -------------------------------------------- | ------------ |
| 2+ variant dimensions (variant, size, state) | CVA          |
| Single conditional class                     | `cn()` alone |
| No variants, just className merge            | `cn()` alone |

---

## cn() Utility

Combines [clsx](https://github.com/lukeed/clsx) (conditional classes) with [tailwind-merge](https://github.com/dcastil/tailwind-merge) (conflict resolution).

```ts
// src/lib/utils.ts
import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}
```

**Requires:** `tailwind-merge@^3.0` for Tailwind v4 compatibility.

### Usage

```tsx
// Conditional classes
cn('px-4 py-2', isActive && 'bg-blue-500', { 'font-bold': isBold });

// Component with className override
function Card({ className, ...props }: React.ComponentProps<'div'>) {
	return <div className={cn('rounded-sm border p-4', className)} {...props} />;
}
// <Card className="p-6" /> → p-4 replaced by p-6
```

| Library          | Purpose                                  |
| ---------------- | ---------------------------------------- |
| `clsx`           | Conditional class joining                |
| `tailwind-merge` | Resolves Tailwind conflicts (later wins) |
| `cn()`           | Both combined                            |

---

## Dark Mode

### Approaches

1. **Automatic** (default) — uses `prefers-color-scheme` media query
2. **Class toggle** — `@custom-variant dark (&:where(.dark, .dark *))`
3. **Data attribute** — `@custom-variant dark (&:where([data-theme=dark], [data-theme=dark] *))`
4. **Three-way toggle** — class-based with system fallback (recommended for user choice)

### Recommended: Data Attribute

```css
@import 'tailwindcss';
@custom-variant dark (&:where([data-theme=dark], [data-theme=dark] *));
```

```html
<html data-theme="dark">
	<div class="bg-white dark:bg-gray-900">...</div>
</html>
```

### Three-Way Toggle (FOUC Prevention)

```html
<script>
	document.documentElement.dataset.theme =
		localStorage.theme === 'dark'
			? 'dark'
			: localStorage.theme === 'light'
				? 'light'
				: matchMedia('(prefers-color-scheme: dark)').matches
					? 'dark'
					: 'light';
</script>
```

---

## Layout — Flex vs Grid

| Use      | When                                                                     |
| -------- | ------------------------------------------------------------------------ |
| **Flex** | One-dimensional (row OR column), content-size-driven, space distribution |
| **Grid** | Two-dimensional (rows AND columns), explicit structure, spanning cells   |

### Flex — Navbar

```html
<nav class="flex items-center justify-between px-6 py-4">
	<div class="font-bold">Logo</div>
	<div class="flex gap-4"><a href="#">Link</a><a href="#">Link</a></div>
</nav>
```

### Grid — Dashboard

```html
<div class="grid grid-cols-[250px_1fr] grid-rows-[auto_1fr] min-h-screen">
	<header class="col-span-2 border-b p-4">Header</header>
	<aside class="border-r p-4">Sidebar</aside>
	<main class="p-6">Content</main>
</div>
```

**Prefer `gap-*`** over `space-*` — v4 changed `space-*` selector from `~ :not([hidden])` to `:not(:last-child)`.

---

## Responsive Design

Mobile-first: unprefixed utilities apply to all sizes; prefixed apply at breakpoint and up.

| Prefix | Min-width      |
| ------ | -------------- |
| `sm:`  | 640px (40rem)  |
| `md:`  | 768px (48rem)  |
| `lg:`  | 1024px (64rem) |
| `xl:`  | 1280px (80rem) |
| `2xl:` | 1536px (96rem) |

### Custom Breakpoints

```css
@theme {
	--breakpoint-xs: 30rem; /* 480px */
	--breakpoint-3xl: 120rem; /* 1920px */
}
```

### Breakpoint Ranges

```html
<div class="md:max-lg:flex-col">Only between md and lg</div>
```

### Responsive Grid

```html
<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
	<!-- cards -->
</div>
```

---

## Container Queries

Native in v4 — no plugin needed. Style based on container width, not viewport.

```html
<div class="@container">
	<div class="flex flex-col @md:flex-row">Switches at container md</div>
</div>
```

| Variant | Container Width |
| ------- | --------------- |
| `@xs:`  | 320px (20rem)   |
| `@sm:`  | 384px (24rem)   |
| `@md:`  | 448px (28rem)   |
| `@lg:`  | 512px (32rem)   |
| `@xl:`  | 576px (36rem)   |

### Named Containers

```html
<div class="@container/sidebar">
	<div class="@sm/sidebar:text-lg">Targets sidebar container</div>
</div>
```

**Best practice:** Container queries for component micro-layouts; media queries for page-level layout.

---

## Animations

### Define in @theme

```css
@theme {
	--animate-fade-in: fade-in 0.3s ease-out;
	--ease-snappy: cubic-bezier(0.2, 0, 0, 1);

	@keyframes fade-in {
		from {
			opacity: 0;
		}
		to {
			opacity: 1;
		}
	}
}
```

Usage: `animate-fade-in`, `ease-snappy`

### @starting-style (Enter Animations)

```html
<div class="transition-opacity starting:opacity-0">Fades in on mount</div>
```

### v4 Transition Changes

- `transform` removed from default transitions — use individual properties: `transition-[opacity,scale]`
- `outline-color` now in transitions — set color unconditionally: `outline-cyan-500 transition`

---

## v3-to-v4 Migration

### Renamed Utilities

| v3             | v4                                 |
| -------------- | ---------------------------------- |
| `shadow-sm`    | `shadow-xs`                        |
| `shadow`       | `shadow-sm`                        |
| `rounded-sm`   | `rounded-xs`                       |
| `rounded`      | `rounded-sm`                       |
| `blur-sm`      | `blur-xs`                          |
| `blur`         | `blur-sm`                          |
| `outline-none` | `outline-hidden`                   |
| `ring`         | `ring-3` (ring is now 1px default) |

### Default Changes

| Property     | v3         | v4                  |
| ------------ | ---------- | ------------------- |
| Border color | `gray-200` | `currentColor`      |
| Ring width   | 3px        | 1px                 |
| Placeholder  | `gray-400` | 50% opacity of text |

### Removed (Use Opacity Modifiers)

`bg-opacity-50` → `bg-black/50`, `text-opacity-50` → `text-black/50`

**Browser requirements:** Safari 16.4+, Chrome 111+, Firefox 128+ (uses `@property`, `color-mix()`)

---

## Deep Dive

**Official Documentation:**

- [Tailwind CSS v4 Blog Post](https://tailwindcss.com/blog/tailwindcss-v4)
- [Theme Variables](https://tailwindcss.com/docs/theme)
- [Adding Custom Styles](https://tailwindcss.com/docs/adding-custom-styles)
- [Dark Mode](https://tailwindcss.com/docs/dark-mode)
- [Responsive Design](https://tailwindcss.com/docs/responsive-design)
- [Upgrade Guide (v3 to v4)](https://tailwindcss.com/docs/upgrade-guide)

**External Resources:**

- [Class Variance Authority (CVA)](https://cva.style/docs)
- [tailwind-merge](https://github.com/dcastil/tailwind-merge)

**Related Skill Files:**

| File                           | Focus                                                           |
| ------------------------------ | --------------------------------------------------------------- |
| [REACT.md](REACT.md)           | Component patterns, hooks, CSS `content-visibility` performance |
| [STATE.md](STATE.md)           | State management, Context, useReducer                           |
| [ROUTING.md](ROUTING.md)       | TanStack Router, route loaders, navigation                      |
| [QUERY_EDEN.md](QUERY_EDEN.md) | TanStack Query + Eden Treaty data fetching                      |
| [VALIDATION.md](VALIDATION.md) | Zod forms, search params, API responses                         |
| [TESTING.md](TESTING.md)       | Vitest + React Testing Library + Playwright                     |
| [WORKFLOW.md](WORKFLOW.md)     | Project structure, conventions, checklists                      |
