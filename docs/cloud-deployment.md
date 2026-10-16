# Cloud Deployment (Vercel + Fly.io + Neon)

Cloud deployment for the app: Vercel (web), Fly.io (API), Neon (Postgres).

---

## Architecture Overview

```mermaid
graph LR
    Browser -->|HTTPS| Web["Web (Vercel)<br/>Static SPA"]
    Web -->|HTTPS| API["API (Fly.io)<br/>Elysia + Bun"]
    API --> DB["PostgreSQL<br/>(Neon)"]
```

- **Web**: Vite build, SPA rewrites, immutable asset cache
- **API**: shared-cpu-1x, 256 MB, auto-stop/start, scales to 0 when idle
- **Postgres**: Neon serverless, scales to 0 after 5 min
- Deployments automated via GitHub Actions on push to `master`

---

## Environment Variables Reference

### By Service

#### API (Elysia / Bun)

| Variable        | Required In      | Default                      | Description                                            |
| --------------- | ---------------- | ---------------------------- | ------------------------------------------------------ |
| `DATABASE_URL`  | Cloud production | none                         | Neon pooled PostgreSQL connection string               |
| `DATABASE_SSL`  | --               | auto-detected from URL       | Force SSL: `true` / `false` (Neon uses SSL by default) |
| `API_PORT`      | All              | `3001` (dev) / `3000` (prod) | Elysia listen port                                     |
| `NODE_ENV`      | Cloud production | --                           | Set to `production` in deployed envs                   |
| `AUTH_PASSWORD` | Cloud production | unset (no auth)              | Login password for web authentication                  |
| `JWT_SECRET`    | Cloud production | unset (no auth)              | HMAC-SHA256 secret for JWT signing                     |
| `WEB_ORIGIN`    | Cloud production | unset (allow all)            | Vercel URL for CORS allowlist                          |

#### Web (React SPA / Vite)

| Variable       | Required In | Default                             | Description               |
| -------------- | ----------- | ----------------------------------- | ------------------------- |
| `VITE_API_URL` | Cloud       | `http://localhost:{API_PORT}` (dev) | API base URL (build-time) |

Build-time injection: `VITE_API_URL` injected at Vite build via env in the deploy workflow.

### By Platform

#### GitHub Actions Secrets

| Secret              | Used By    | Description                              |
| ------------------- | ---------- | ---------------------------------------- |
| `FLY_API_TOKEN`     | deploy-api | Fly.io deployment token                  |
| `DATABASE_URL`      | deploy-api | Neon pooled PostgreSQL connection string |
| `AUTH_PASSWORD`     | deploy-api | Login password for authentication        |
| `JWT_SECRET`        | deploy-api | JWT signing secret                       |
| `VERCEL_TOKEN`      | deploy-web | Vercel deployment token                  |
| `VERCEL_ORG_ID`     | deploy-web | Vercel org ID                            |
| `VERCEL_PROJECT_ID` | deploy-web | Vercel project ID                        |

#### GitHub Actions Variables

| Variable       | Used By                | Description                                         |
| -------------- | ---------------------- | --------------------------------------------------- |
| `FLY_APP_NAME` | deploy-api, deploy-web | Fly.io app name (e.g. `app-api`)                    |
| `WEB_ORIGIN`   | deploy-api             | Vercel URL for CORS (e.g. `https://app.vercel.app`) |

#### Fly.io

**Secrets** (via `flyctl secrets set`, staged by GitHub Actions): `DATABASE_URL` (Neon connection string), `AUTH_PASSWORD`, `JWT_SECRET`, `WEB_ORIGIN`

**Environment** (in `fly-api.toml` `[env]`): `NODE_ENV=production`, `API_PORT=3000`, `MIGRATE_ON_START=false` (the image's entrypoint skips its own migration; `release_command` has run it)

**Optional**: `DATABASE_PREPARE=false` (secret or `[env]`) sends queries unprepared again without a new build -- for a pooler that does not carry prepared statements

**Networking**: Shared IPv4 (free) + IPv6. No dedicated IPv4.

#### Neon

**Connection strings** (from Neon dashboard):

- **Pooled** (for runtime): `postgresql://user:pass@ep-xxx-pooler.<region>.neon.tech/dbname?sslmode=require`
- **Direct** (for migrations/drizzle-kit): `postgresql://user:pass@ep-xxx.<region>.neon.tech/dbname?sslmode=require`

The pooled connection string is used as `DATABASE_URL` in production. SSL is enforced by Neon.

#### Vercel

Passed as env in deploy workflow: `VITE_API_URL=https://{FLY_APP_NAME}.fly.dev`.

---

## Configuration Files Reference

| File                                  | Purpose                                                              |
| ------------------------------------- | -------------------------------------------------------------------- |
| `.github/workflows/deploy.yml`        | CD pipeline                                                          |
| `deploy/fly-api.toml`                 | Fly.io config: region, VM, health checks, release command, auto-stop |
| `docker/api/Dockerfile`               | The API's image                                                      |
| `apps/web/vercel.json`                | Vercel: build command, SPA rewrites, cache                           |
| `apps/api/drizzle.config.ts`          | Drizzle Kit config                                                   |
| `apps/api/src/db/connection.ts`       | DB connection: Neon via `DATABASE_URL`, SSL from `databaseSsl`       |
| `apps/api/src/db/migrate.ts`          | Migration runner                                                     |
| `apps/api/src/middleware/security.ts` | JWT authentication guard                                             |
| `apps/api/src/modules/auth/index.ts`  | Login/logout/check endpoints                                         |
| `apps/web/src/env.ts`                 | Zod-validated `VITE_` env vars                                       |
| `apps/web/vite.config.ts`             | Vite config: env, API URL injection, dev port                        |

---

## Service Communication and Security

**Web -> API**: Eden Treaty (type-safe HTTP client). Auth via JWT Bearer token (`Authorization: Bearer <token>`). CORS restricted to `WEB_ORIGIN` in production.

**Authentication** (`apps/api/src/modules/auth/index.ts` + `apps/api/src/middleware/security.ts`):

1. User logs in via `POST /api/auth/login` with password -> receives JWT (one week expiry)
2. JWT sent as `Authorization: Bearer <token>` header on all requests. A route read by an `EventSource`, which sends no headers, can take the token as `?token=` instead: list its path in `queryTokenPaths` (`apps/api/src/routes.ts`), which is empty by default
3. Security middleware validates JWT on every request (except `/health`, `/api/wake`, `/api/auth/login`)
4. Auth disabled when `JWT_SECRET` env var is unset (frictionless local dev)

---

## Database

**Neon Serverless Postgres**: the free tier scales to zero after 5 min of inactivity. Wake-up latency ~300-500ms on first query after idle.

**Connection strings**: Neon provides two connection strings per project:

- **Pooled** (via PgBouncer, `-pooler` in hostname): use for runtime queries (`DATABASE_URL`)
- **Direct** (no pooler): use for migrations and `drizzle-kit` operations

**Migrations**: run before each deployment via `release_command = "bun src/db/migrate.ts"` in `fly-api.toml` -- new version doesn't receive traffic until migrations succeed. Migrations use the pooled connection via `DATABASE_URL` (standard Drizzle `migrate()` works with PgBouncer's protocol-level prepared statement support).

**Prepared statements**: the API sends its queries as protocol-level prepared statements (`withPreparedStatements` from `@jswob/drizzle-utils`), which Neon's pooler carries (PgBouncer 1.22+, `max_prepared_statements=1000`). A repeated query costs one round trip on a warm connection instead of two. `DATABASE_PREPARE=false` turns it off.

**Staying asleep**: nothing in the API polls the database. Pooled connections close after 20 s idle, and `/health` (asked by Fly every 10 s) does not touch the database -- so Neon scales to zero while the API idles, and the VM is suspended with no connection open. The web asks `GET /api/wake` when it shows sign-in: that starts the VM and, at most once in 15 minutes, wakes the database with one `select 1`. Work the API does in the background starts in `startBackground` (`apps/api/src/routes.ts`); keep it asleep while idle too.

**Connection priority** (`apps/api/src/db/connection.ts`): (1) `DATABASE_URL` if set, (2) constructed from component vars, (3) in production `DATABASE_URL` is required -- server throws on startup if missing. SSL comes from `databaseSsl` (`@jswob/drizzle-utils`), which `drizzle.config.ts` also asks: `DATABASE_SSL` when set, otherwise `sslmode=require` in the connection string, which Neon's has.

---

## Quick Start

**Prerequisites**: GitHub repo with Actions, Fly.io (`flyctl`), Neon account, Vercel account.

1. **Neon**: Create a project in the region nearest your Fly app -> copy pooled connection string as `DATABASE_URL`
2. **Fly.io**: `flyctl apps create <name>` -> `fly ips allocate-v6 -a <name>` -> `fly ips allocate-v4 --shared -a <name>`; put the name in `deploy/fly-api.toml` and in the `FLY_APP_NAME` variable
3. **Generate** `AUTH_PASSWORD` (your login password) and `JWT_SECRET` (`openssl rand -hex 32`)
4. **GitHub Actions**: set secrets and variables per tables above (see [docs/ci-cd.md](ci-cd.md) for full CI/CD setup). Add `DATABASE_URL` with the Neon pooled connection string.
5. **Vercel**: import repo, root directory `apps/web`, framework preset Vite (env vars passed via env in workflow)
6. **Deploy**: push to `master` -- workflow detects changes, deploys API -> web
7. **Verify**: `curl https://<name>.fly.dev/health` -> `{"status":"ok",...}`

---

## Troubleshooting

**Sign-in fails** -- Check `AUTH_PASSWORD` and `JWT_SECRET` are set as Fly.io secrets. Both are required for authentication.

**API returns 401** -- JWT token expired (one week) or invalid. Sign in again in the web app.

**Fly.io deploy fails at release_command** -- migration failed. Check `DATABASE_URL` is set as a Fly.io secret (via GitHub Actions) and points to a valid Neon database. Check migration SQL in `apps/api/drizzle/`.

**Vercel build fails** -- verify root directory is `apps/web`, `VITE_API_URL` passed via env. Needs `apps/api/src/` for Eden Treaty types (full repo clone required).

**API cold start slow** -- expected with `min_machines_running=0`. Fly.io suspends the idle VM and resumes it on the next request (`auto_stop_machines = "suspend"`, a few hundred ms). A deploy or a host move discards the snapshot, so the first request after one boots from cold (seconds). If unacceptable, set `min_machines_running=1` in `fly-api.toml` (~$2/month extra).

**API_PORT mismatch** -- dev defaults to 3001, production to 3000 (set in `fly-api.toml` `[env]`). No action needed unless overridden.

**Database connection timeout after idle** -- Neon scales to zero after 5 min of inactivity. First query after idle takes ~300-500ms for wake-up. The `connect_timeout: 15` setting in `connection.ts` accommodates this. If timeouts persist, check Neon dashboard for compute hour usage.

**Neon compute hours exhausted** -- When the free tier's compute hours run out, compute suspends until next billing cycle. Monitor usage in Neon Console > Billing.
