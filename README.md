# bun-monorepo

A full-stack TypeScript monorepo: an Elysia API over PostgreSQL, a React web
app that reaches it through Eden Treaty, the app's own UI package, a design workbench that stages
its screens on fixtures, an LLM evals harness, and the Claude Code skills that work on all of it.

It ships with sign-in, an account page with the light/dark switch, and one example resource,
**notes**, wired from the table to the screen. `docs/removing-notes.md` lists every file the example
takes, so the app removes it in one commit once it has its own first resource.

<!-- template:start -->

This repo is the template apps start from (§ Starting an app from it), and the home of the
`@jswob/*` packages they take from npm.

<!-- template:end -->

## Tech Stack

| Layer     | Technology                                                          |
| --------- | ------------------------------------------------------------------- |
| Runtime   | [Bun](https://bun.sh)                                               |
| API       | [Elysia](https://elysiajs.com) (type-safe HTTP framework)           |
| Frontend  | [React](https://react.dev) + [Vite](https://vite.dev)               |
| Routing   | [TanStack Router](https://tanstack.com/router) (file-based)         |
| Data      | [TanStack Query](https://tanstack.com/query)                        |
| API Types | [Eden Treaty](https://elysiajs.com/eden/treaty/overview.html)       |
| Database  | PostgreSQL via [Drizzle ORM](https://orm.drizzle.team)              |
| Styling   | [Tailwind CSS v4](https://tailwindcss.com) + CVA                    |
| Testing   | [Vitest](https://vitest.dev) + [Playwright](https://playwright.dev) |
| Deploy    | Fly.io (API) · Vercel (web) · Neon (Postgres)                       |

## Quick Start

**Prerequisites:** Bun, Docker

```bash
bun install
bun setup          # copies .env.example to .env
bun db:up          # starts the dev, test and e2e databases
bun db:migrate
bun dev            # every *:dev script: the web app and the API
```

The web app runs at `http://localhost:5173` and the API at `http://localhost:3001`. Sign in with
`AUTH_PASSWORD` from `.env`; leave `JWT_SECRET` unset to run without sign-in.

Several apps on one machine each need their own `COMPOSE_PROJECT_NAME` and database ports in
`.env`: they name the containers and the snapshot volumes.

The databases run Postgres 18, whose image keeps its data under `/var/lib/postgresql/18/`, so the
volumes are mounted at `/var/lib/postgresql`. A volume made under Postgres 16 will not start
(`Error: in 18+, these Docker images are configured to store database data in a format …`), and
neither will a snapshot of one restored: `pg_dump` what you need with a 16 container first, then
`bun db:down`, remove the project's `postgres_*` volumes, `bun db:up`, migrate, and take a new
`bun db:base`.

## Project Structure

```
├── apps/api/                    # Elysia API server (PostgreSQL, Drizzle ORM)
├── apps/web/                    # React SPA (Vite, TanStack Router, TanStack Query)
├── apps/workbench/              # The design workbench's content: places, fixtures, the bans
├── apps/evals/                  # The app's LLM eval tasks and answer keys
├── packages/ui/                 # The app's UI: the kit, the views, the theme
├── deploy/ · docker/            # Fly config and the API's image
└── docs/                        # Deployment, CI/CD, removing the example
```

Each app and package has its own `CLAUDE.md`. The shared tooling is the `@jswob/*` packages:
`style-guide` (TypeScript and Prettier configs), `drizzle-fixtures` (test data), `drizzle-snapshots`
(the `db:*` snapshots, the safe push, test databases), `drizzle-utils` (Drizzle runtime helpers),
`llm-evals` (the evals harness), `toolkit` (copies the packages' Claude Code skills into `.claude/`)
and `design-workbench` (the design workbench's engine).

<!-- template:start -->

Here they are workspaces, and an app installs them from npm:

```
├── packages/config/             # @jswob/style-guide
├── packages/drizzle-fixtures/   # @jswob/drizzle-fixtures
├── packages/drizzle-snapshots/  # @jswob/drizzle-snapshots
├── packages/drizzle-utils/      # @jswob/drizzle-utils
├── packages/llm-evals/          # @jswob/llm-evals
├── packages/toolkit/            # @jswob/toolkit
├── packages/workbench/          # @jswob/design-workbench
├── renovate/                    # The presets an app takes them by
└── template/                    # How this template is cut, and bun setup's half
```

<!-- template:end -->

## Key Commands

| Command            | Description                                               |
| ------------------ | --------------------------------------------------------- |
| `bun dev`          | Start the web app and the API with hot reload             |
| `bun build`        | Build both apps for production                            |
| `bun run test`     | Every Vitest suite (sets up the test database first)      |
| `bun api:test`     | The API's tests; `web:test`, `ui:test`… for one workspace |
| `bun lint`         | ESLint, every workspace's typecheck and the toolkit check |
| `bun lint:fix`     | Auto-fix ESLint issues                                    |
| `bun db:up`        | Start the PostgreSQL containers                           |
| `bun db:push:safe` | Apply schema changes, snapshotting the database first     |
| `bun db:undo`      | Revert the last push                                      |
| `bun e2e`          | Playwright end-to-end tests (`bun e2e:setup` first)       |
| `bun designs`      | The design workbench at `http://localhost:5175`           |
| `bun workbench`    | The workbench's verbs (`check`, `probe`, `sweep`, `ban`…) |
| `bun evals:*`      | The LLM evals harness (`doctor`, `try`, `run`, `report`…) |

<!-- template:start -->

## Starting an app from it

1. Create the app's repo from this one (GitHub: Use this template) and clone it beside this
   checkout.
2. `bun setup <name>`, where the name is lowercase letters, digits and hyphens. It names the app:
   the root `package.json`, the workspaces' scope (`@app/` becomes `@<name>/`), the database names
   and `COMPOSE_PROJECT_NAME`, the Fly app (`<name>-api`), the page title. It gives the databases
   and the dev servers their own ports, so several apps run side by side. It takes the `@jswob/*`
   packages from npm instead of these sources, at the versions here, and deletes `template/`, this
   section, the packages' release (`.changeset/`, `release.yml`, `scripts/release.ts`) and
   `renovate/`. It writes `renovate.json` on a preset, names this checkout as the toolkit's home,
   records the template commit, then installs and syncs `.claude/`.

   | Option                               | Default                                          |
   | ------------------------------------ | ------------------------------------------------ |
   | `--title <title>`                    | The name in words (`my-app` → My App)            |
   | `--db-port <port>`                   | One from the name; the test and e2e ones follow  |
   | `--port <port>`                      | The API's, from the name; web, e2e, bench follow |
   | `--renovate auto\|manual\|next\|off` | `auto` (`renovate/README.md`)                    |
   | `--home <dir>`                       | `../bun-monorepo`                                |
   | `--no-install`                       | Install and sync                                 |

3. Review the change and commit it.

<!-- template:end -->

## Making it the app's

1. Build the first resource beside notes, copying its shape; then remove notes with the list in
   `docs/removing-notes.md`.
2. Put the app's own words in `toolkit.agnostic.words` in the root `package.json`, so `bun lint`
   fails when one leaks into what must not know the app: the folders in `toolkit.agnostic.paths`,
   which in an app are its patches of the `@jswob/*` packages, since those travel home.
3. Install the Renovate app on the repo, and on the Mend developer portal set its dependency updates
   from Silent to on, or no PR opens. `renovate.json` extends `config:best-practices`, the
   `third-party` preset that puts every other non-major update in one PR, and one of the toolkit's
   presets (`renovate/README.md` in bun-monorepo), and moves Bun's pin and its Docker image together.
4. Install the toolkit's GitHub App on the repo and give it the `TOOLKIT_APP_ID` variable and the
   `TOOLKIT_APP_KEY` secret (`docs/ci-cd.md` § The toolkit's GitHub App): a patch of a `@jswob/*`
   package then travels back to bun-monorepo as a pull request, and Renovate's updates of them carry
   the app's patches and sync its `.claude/`. `bun toolkit upstream`, `link` and `try` by hand use
   the checkout `toolkit.home` names in the root `package.json`.
5. Deploy: `docs/cloud-deployment.md`.

<!-- template:start -->

This repo is the home of the `@jswob/*` packages (MIT): they are workspaces here, and an app takes
them from npm by version. The skills they carry are copied into `.claude/` by `bun toolkit sync`.
`TEMPLATE.md` § Releasing the packages says how a change reaches npm.

<!-- template:end -->

## Deployment

`docs/cloud-deployment.md` sets up Fly.io, Vercel and Neon; `docs/ci-cd.md` describes the GitHub
Actions workflows: CI, deployment, an app's fixes to the `@jswob/*` packages travelling back, and
Renovate's updates of them finished.
