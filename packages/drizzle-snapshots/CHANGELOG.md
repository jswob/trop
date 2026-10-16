# @jswob/drizzle-snapshots

## 0.1.2

### Patch Changes

- 38871f3: An error the CLI throws over a failed shell command or a failed raw constraint now carries the
  original as its `cause`, so the whole of what went wrong reaches the stack trace.

## 0.1.1

### Patch Changes

- 58a0ba9: The packages take any `0.x` Drizzle from the one they were built on (`drizzle-orm` from 0.44.4,
  `drizzle-kit` from 0.31.4), so an app moving to drizzle-orm 0.45 keeps one copy of it instead of
  installing a second one for these packages. `withPreparedStatements` prepares Drizzle's queries
  again on 0.45, which passes `prepare: false` for every query not named with `.prepare()`.
