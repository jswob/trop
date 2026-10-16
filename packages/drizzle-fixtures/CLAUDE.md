# Drizzle Fixtures

Type-safe database fixtures for Drizzle ORM with traits, relationships, and transaction support.

## Key Exports

- **`createFixture(config)`** — defines a fixture with table, fields, traits, and hooks
- **`composeFactory(fixtures)`** — combines fixtures into a single factory object
- **`createUseHelper(options)`** — creates `use()` helper for Vitest test context integration
- **Error classes** — `FixtureError`, `TraitNotFoundError`, `FieldResolverError`, `CircularDependencyError`, `DatabaseOperationError`, `ValidationError`, `HookExecutionError`
- **Utilities** — `resolveFields`, `mergeResolvers`, `createSequenceCounter`, `getTableName`, `getPrimaryKeyColumn`

## Documentation

See `docs/` for detailed guides: `index.md` (overview), `api-reference.md`, `traits.md`, `hooks.md`, `context.md`.
