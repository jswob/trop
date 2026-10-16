# Web

React SPA frontend, drawn with the UI package (`@app/ui`). It talks to the API through Eden Treaty,
a type-safe HTTP client derived from the Elysia server's type export.

---

## The app

**The ground is the home page**: the app's name and what each destination is for. From `lg` it
stands beside every page; below `lg` every page opens over it.

**Destinations open over the ground.** From `lg` they open from the rail on the split; below `lg`
they open from the slug, the bar floating at the foot.

| Destination | Address    | What it holds                                                 |
| ----------- | ---------- | ------------------------------------------------------------- |
| Notes       | `/notes`   | The example resource: write a note, read them all, delete one |
| Account     | `/account` | This device's sign-in and appearance, and how to end it       |

A record opened inside a destination has its own address, `/<destination>/<kind>/<id>`. A reload or
a new tab lands on it.

**Signing in** is drawn in place of the frame, with one shared password. **A read that fails** (the
server unreachable, or answering with an error) puts a centred screen up in place of the frame when
the ground's own data failed, and is said in the page's wait when only the page's did. `Try again`
asks again for what failed. Any unknown address lands on the ground.

---

## Where the code is

| Path                      | What                                                                              |
| ------------------------- | --------------------------------------------------------------------------------- |
| `packages/ui/src/kit/`    | The primitives: the split, the rail, lists, fields, buttons                       |
| `packages/ui/src/views/`  | Each page's drawing, and the frame                                                |
| `src/frame/`              | The channel a page declares through, and what each page tells the frame           |
| `src/routes/`             | File-based routes; `__root.tsx` draws the one frame around every page             |
| `src/routes/-components/` | The frame's wiring, the ground, and each page's hook                              |
| `src/hooks/api/`          | TanStack Query hooks over Eden Treaty, with key factories in `keys.ts`            |
| `apps/workbench/`         | The design workbench (`bun designs`): every place and component drawn on fixtures |

The UI package is drawn by this app and staged by the workbench. `apps/workbench/BANS.md` records
what was tried and rejected.

---

## API communication

Query hooks in `hooks/api/` follow TanStack Query conventions. A query hook exports both
`queryOptions()` and a `useQuery()` wrapper; a mutation hook uses `useMutation` and invalidates what
it changed. A failed request throws `ApiError` carrying the HTTP status, which the query client uses
to never retry a 4xx and the frame uses to say a read failed.

---

## Related documentation

- [CLAUDE.md](CLAUDE.md) — development guide and coding patterns
- [API README](../api/README.md) — backend API documentation
