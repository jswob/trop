/**
 * The kit: the components every view and page is drawn with. A module lands here through
 * `bun designs:x promote kit/<Name> --to app`; the workbench's `apps/workbench/kit/index.ts`
 * re-exports it from here, so there is one copy. The verb appends a promoted module's exports to
 * `./core`.
 */
export * from './core';
