# @jswob/design-workbench

## 0.1.2

### Patch Changes

- 8f80f0b: Moves to Zod 4. The bench reads its address with Zod, so an app installing it gets `zod` 4 with it.
- Updated dependencies [5ceef0d]
  - @jswob/toolkit@0.1.4

## 0.1.1

### Patch Changes

- 370a527: The bench takes Vite 8 as well as Vite 7, so an app moving to Vite 8 (and `@vitejs/plugin-react` 6,
  which needs it) keeps one copy of Vite. The board kit still builds through `build.rollupOptions`,
  which Vite 8 reads as `rolldownOptions`.
