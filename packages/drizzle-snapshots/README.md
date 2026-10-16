# @jswob/drizzle-snapshots

A dev CLI for a Drizzle app whose PostgreSQL runs in Docker Compose: snapshots of the dev
database's volume, a schema push that can be undone, and the setup of the test and e2e databases.

```bash
bun add -d @jswob/drizzle-snapshots drizzle-kit
```

Requires Bun and Docker. It ships TypeScript source and runs under Bun.

## Plug in

`drizzle-snapshots.config.ts`, beside the app's `drizzle.config.ts`:

```ts
import { defineConfig } from '@jswob/drizzle-snapshots';
import { dbTarget } from './src/db/target'; // createDbTargets from @jswob/drizzle-utils

export default defineConfig({
	dbTarget,
	migrate: 'src/db/migrate.ts',
	seed: 'src/db/seed.ts',
	rawConstraints: [], // constraints Drizzle cannot declare, put back after each push
});
```

Run it from the app's folder with the environment loaded:

```json
"db:push:safe": "bun --env-file=../../.env drizzle-snapshots push",
"db:undo": "bun --env-file=../../.env drizzle-snapshots undo",
"test:setup": "bun --env-file=../../.env drizzle-snapshots test-setup"
```

## Commands

| Command                                    | What it does                                                    |
| ------------------------------------------ | --------------------------------------------------------------- |
| `snapshot [name]`                          | Copy the dev database's volume; `base` is a protected baseline  |
| `restore [name]`, `undo`                   | Put a snapshot back; `undo` restores the one taken by the push  |
| `list`, `clean [--force]`                  | List snapshots; prune old ones                                  |
| `push`                                     | Snapshot, `drizzle-kit push`, then re-apply the raw constraints |
| `constraints`                              | Re-apply the raw constraints alone                              |
| `test-setup`                               | A fresh test database: migrations, then any newer schema        |
| `e2e-setup`, `e2e-restore`, `e2e-teardown` | The e2e database, seeded once and restored between runs         |
| `schema-check`                             | Fail when a staged schema change has no migration (pre-commit)  |

## Conventions

- `COMPOSE_PROJECT_NAME` is set, and every volume name starts with it, so two projects on one
  machine never share a snapshot.
- Containers are `<project>-postgres` (or `<project>-$POSTGRES_CONTAINER_NAME`),
  `<project>-postgres-test` and `<project>-postgres-e2e`.
- `drizzle.config.test.ts` and `drizzle.config.e2e.ts` stand beside `drizzle.config.ts`.
- The test and e2e databases are reached by name, never through `DATABASE_URL`.
- drizzle-kit runs as `bun --bun drizzle-kit`, so a drizzle config may import TypeScript packages.

## License

MIT
