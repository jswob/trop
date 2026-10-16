# Removing notes

**Notes** is the example resource: a table, its API module, the web's hooks and route, a page and
its workbench place. Build the app's first resource beside it, copying its shape. Then take notes
out in one commit, this file with it.

**Delete** this file, and:

| Where     | Files                                                                                                                                                                                    |
| --------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| API       | `apps/api/src/db/schema/notes.ts`, `apps/api/src/modules/notes/`, `apps/api/src/test/fixtures/note.fixture.ts`                                                                           |
| Web       | `apps/web/src/hooks/api/useNotes.ts` and its test, `apps/web/src/routes/notes.tsx`, `apps/web/src/routes/-components/use-notes-page.ts` and its test, `apps/web/e2e/tests/notes.spec.ts` |
| UI        | `packages/ui/src/views/notes.tsx`                                                                                                                                                        |
| Workbench | `apps/workbench/views/notes-places.tsx`, `apps/workbench/fixtures/notes.ts`                                                                                                              |

**Edit:**

| File                                                                                                                                         | Change                                                                      |
| -------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------- |
| `apps/api/src/routes.ts`                                                                                                                     | Drop `createNotesRoute`; mount the app's own modules                        |
| `apps/api/src/db/schema/index.ts`                                                                                                            | Drop `export * from './notes'`                                              |
| `apps/api/src/test/fixtures/index.ts`                                                                                                        | Drop `note: noteFixture`                                                    |
| `apps/api/src/db/seed-tables.ts`                                                                                                             | `SEED_TABLES` lists the app's tables; `seedFallback` seeds them, or nothing |
| `apps/api/src/db/seed-core.test.ts`                                                                                                          | Point its one table at one of the app's                                     |
| `apps/web/src/hooks/api/keys.ts`, `index.ts`                                                                                                 | Drop `noteKeys` and the notes hooks from the barrel                         |
| `packages/ui/src/views/destinations.ts`                                                                                                      | Replace the `notes` destination with the app's; `RESTING` names one of them |
| `apps/web/src/frame/pages.ts`                                                                                                                | `ROUTED` names the destinations that have a route                           |
| `apps/workbench/views/index.ts`                                                                                                              | Drop `NOTES_PLACES`; list the app's places                                  |
| `apps/workbench/fixtures/rail-facts.ts`                                                                                                      | The rail's counts come from the app's fixtures                              |
| `packages/ui`, `apps/web`, `apps/workbench` tests                                                                                            | Those that use `notes` as their sample destination take one of the app's    |
| `CLAUDE.md`, `README.md`, `apps/api/{CLAUDE,README}.md`, `apps/web/CLAUDE.md`, `.claude/skills/elysia-drizzle-api-best-practices/PROJECT.md` | Their lines on notes point at the app's first resource instead              |

Then `bun db:push:safe`, `bun lint` and `bun run test` (`bun test` is Bun's own runner, not the script). The route tree regenerates on `bun dev`.
