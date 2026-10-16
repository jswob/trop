/**
 * The kit, as a candidate or a board imports it: `../kit`.
 *
 * Every settled component stands in the UI package (`@app/ui/kit`), and this barrel re-exports the
 * whole of it, so there is one copy and nothing here to keep in step. A component that only the
 * workbench draws is written beside this file and exported below; `workbench promote <id> --to kit`
 * adds the line. It enters the kit on its second caller, never its first.
 */
export * from '@app/ui/kit';
