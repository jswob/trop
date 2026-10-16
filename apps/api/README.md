# API

Elysia API server. Owns the PostgreSQL database via Drizzle ORM and serves as the data layer for
the React frontend, which reaches it through Eden Treaty.

---

## Module Structure

The API is organized into **modules** at `src/modules/`. Each module encapsulates one concern with
its route handler (`index.ts`), optional service layer (`service.ts`), and collocated tests
(`[name].test.ts`).

| Module  | Prefix       | Description                                         |
| ------- | ------------ | --------------------------------------------------- |
| `auth`  | `/api/auth`  | Sign-in (issues a JWT)                              |
| `wake`  | `/api/wake`  | Starts the machine and wakes an idle database       |
| `notes` | `/api/notes` | The example resource: list, read, add, change, drop |

Each routed module exports a `create[Name]Route(db)` factory function that returns an Elysia
instance (`auth` takes no `db`). `src/index.ts` is the shell — the logger, CORS, the auth guard in
`src/middleware/security.ts`, sign-in, wake and `/health` — and mounts the app's own modules from
`routes(db)` in `src/routes.ts`.

---

## API Endpoints

Errors from services map to HTTP statuses: `NOT_FOUND` → **404**, `VALIDATION` → **422**,
`CONFLICT` → **409**, `INTERNAL` → **500**. Error bodies are `{ error: string }`. Names and titles
are trimmed before they are checked.

### Authentication

| Method | Path              | Description                      |
| ------ | ----------------- | -------------------------------- |
| `POST` | `/api/auth/login` | Sign in with the shared password |

POST takes `{ password }` and returns `{ token }`, a JWT valid for a week. Returns **401** for a
wrong password and **500** when `AUTH_PASSWORD` or `JWT_SECRET` is not set.

**Auth guard** — When `JWT_SECRET` is set, every route except `GET /health`, `GET /api/wake` and
`POST /api/auth/login` requires a valid token, sent as `Authorization: Bearer <token>`. A route read
by an `EventSource`, which sends no headers, can take it as a `?token=` query parameter instead:
list its path in `queryTokenPaths` (`src/routes.ts`), empty by default. A missing, expired or
invalid token returns **401**. When `JWT_SECRET` is unset the guard is a no-op.

### Health and wake

| Method | Path        | Description                                       |
| ------ | ----------- | ------------------------------------------------- |
| `GET`  | `/health`   | Health check JSON; never touches the database     |
| `GET`  | `/api/wake` | Wakes the database, at most once every 15 minutes |

`/health` returns `{ status: 'ok', timestamp }`. Public; Docker and Fly probe it. The web asks
`/api/wake` when it shows sign-in, so the machine and the database are up by the time the password
is typed.

### Notes

The example resource. `docs/removing-notes.md` at the repo root lists every file it takes, so an app removes
it in one commit.

| Method   | Path             | Description                               |
| -------- | ---------------- | ----------------------------------------- |
| `GET`    | `/api/notes`     | Every note, the one changed last first    |
| `GET`    | `/api/notes/:id` | One note; **404** when there is none      |
| `POST`   | `/api/notes`     | Add a note: `{ title, body? }`; **201**   |
| `PATCH`  | `/api/notes/:id` | Change a note's `title` and/or `body`     |
| `DELETE` | `/api/notes/:id` | Delete a note; **404** when there is none |

A title is 1–120 characters after trimming; a body is any text and defaults to `''`.

---

## Data Model

| Table   | Purpose                                                  |
| ------- | -------------------------------------------------------- |
| `notes` | The example resource: a title, a body and the timestamps |

Shared columns (`uuidPrimaryKey`, `timestamps`) are in `src/db/schema/common.ts`; one file per
table, re-exported from `src/db/schema/index.ts`, which is also the `@app/api/types` export.
