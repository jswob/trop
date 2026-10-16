# @jswob/drizzle-utils

## 0.1.1

### Patch Changes

- 58a0ba9: The packages take any `0.x` Drizzle from the one they were built on (`drizzle-orm` from 0.44.4,
  `drizzle-kit` from 0.31.4), so an app moving to drizzle-orm 0.45 keeps one copy of it instead of
  installing a second one for these packages. `withPreparedStatements` prepares Drizzle's queries
  again on 0.45, which passes `prepare: false` for every query not named with `.prepare()`.
