# API Reference

Complete API documentation for `@jswob/drizzle-fixtures`.

## Table of Contents

- [createFixture](#createfixture)
- [FixtureBuilder](#fixturebuilder)
- [composeFactory](#composefactory)
- [Error Classes](#error-classes)
- [Type Exports](#type-exports)

## createFixture

Creates a fixture definition for generating test data.

```typescript
function createFixture<
	TTable extends Table,
	TTraits extends Record<string, unknown> = Record<string, never>,
	TAugmentations extends Partial<Record<keyof TTraits, Record<string, unknown>>> = Record<
		string,
		never
	>,
>(
	config: FixtureConfig<TTable, TTraits, TAugmentations>
): FixtureFunction<TTable, TTraits, TAugmentations>;
```

### Type Parameters

| Parameter        | Description                               |
| ---------------- | ----------------------------------------- |
| `TTable`         | Drizzle table type (e.g., `typeof users`) |
| `TTraits`        | Trait definitions object type (optional)  |
| `TAugmentations` | Trait augmentation types (optional)       |

### Config

```typescript
interface FixtureConfig<TTable, TTraits, TAugmentations> {
	table: TTable;
	fields: FieldResolvers<TTable>;
	traits?: TraitsConfig<TTable, TTraits, TAugmentations>;
	hooks?: LifecycleHooks<InferSelectModel<TTable>>;
}
```

| Property | Type                     | Description                  |
| -------- | ------------------------ | ---------------------------- |
| `table`  | `TTable`                 | Drizzle table object         |
| `fields` | `FieldResolvers<TTable>` | Base field resolvers         |
| `traits` | `TraitsConfig`           | Trait definitions (optional) |
| `hooks`  | `LifecycleHooks`         | Lifecycle hooks (optional)   |

### Returns

`FixtureFunction` - A function `(db) => FixtureBuilder`

### Example

```typescript
const userFixture = createFixture<typeof users>({
	table: users,
	fields: {
		name: () => 'Test User',
		email: ({ sequence }) => `user${sequence}@example.com`,
	},
});

// Use with database connection
const user = await userFixture(db).create();

// Use with transaction
await db.transaction(async tx => {
	const user = await userFixture(tx).create();
	tx.rollback();
});
```

## FixtureBuilder

Builder returned by calling a fixture function.

### Methods

#### trait(name, params?)

Apply a trait to the fixture.

```typescript
.trait<K extends keyof TTraits>(
  name: K,
  params?: TTraits[K] extends never ? never : TTraits[K]
): FixtureBuilder
```

#### build(overrides?)

Create an object in memory (not persisted to database).

```typescript
.build(overrides?: Partial<InferInsertModel<TTable>>): Promise<TModel & Augmentations>
```

#### buildList(count, overrides?)

Create multiple in-memory objects.

```typescript
.buildList(
  count: number,
  overrides?: Partial<InferInsertModel<TTable>>
): Promise<Array<TModel & Augmentations>>
```

#### create(overrides?)

Create and persist an object to the database.

```typescript
.create(overrides?: Partial<InferInsertModel<TTable>>): Promise<TModel & Augmentations>
```

#### createList(count, overrides?)

Create and persist multiple objects.

```typescript
.createList(
  count: number,
  overrides?: Partial<InferInsertModel<TTable>>
): Promise<Array<TModel & Augmentations>>
```

#### dryRun()

Preview what would be created without side effects.

```typescript
.dryRun(): Promise<{ data: TModel; traits: string[] }>
```

#### explain()

Get explanation of fixture configuration.

```typescript
.explain(): FixtureExplanation
```

## composeFactory

Compose multiple fixtures bound to a database connection.

```typescript
function composeFactory<T extends FixturesMap>(fixtures: T): ComposedFactory<T>;
```

### Parameters

| Parameter  | Type          | Description                               |
| ---------- | ------------- | ----------------------------------------- |
| `fixtures` | `FixturesMap` | Object mapping names to fixture functions |

### Returns

`ComposedFactory` - A function that accepts `(db)` and returns a `BoundFactory`.

### BoundFactory

The session object returned by calling a composed factory.

```typescript
interface BoundFactory<T> {
	[K in keyof T]: FixtureBuilder; // Access fixtures by name
}
```

### Example

```typescript
const factory = composeFactory({
	user: userFixture,
	post: postFixture,
});

// Create session bound to database
const session = factory(db);

// Access fixtures
const user = await session.user.create();
const post = await session.post.create({ userId: user.id });

// Use with transactions for cleanup
await db.transaction(async tx => {
	const session = factory(tx);
	const user = await session.user.create();
	// ...
	tx.rollback();
});
```

## Error Classes

All errors extend `FixtureError`.

### FixtureError

Base error class for all fixture-related errors.

```typescript
class FixtureError extends Error {
	constructor(message: string);
}
```

### TraitNotFoundError

Thrown when applying a trait that doesn't exist.

```typescript
class TraitNotFoundError extends FixtureError {
	constructor(traitName: string, availableTraits: string[]);
}
```

### FieldResolverError

Thrown when a field resolver fails.

```typescript
class FieldResolverError extends FixtureError {
	constructor(fieldName: string, originalError: Error);
}
```

### CircularDependencyError

Thrown when fixtures create a circular dependency.

```typescript
class CircularDependencyError extends FixtureError {
	constructor(path: string[]);
}
```

### DatabaseOperationError

Thrown when a database operation fails.

```typescript
class DatabaseOperationError extends FixtureError {
	constructor(operation: string, originalError: Error);
}
```

### ValidationError

Thrown when validation fails.

```typescript
class ValidationError extends FixtureError {
	constructor(message: string);
}
```

### HookExecutionError

Thrown when a lifecycle hook fails.

```typescript
class HookExecutionError extends FixtureError {
	hookName: string;
	constructor(hookName: string, originalError: Error);
}
```

## Type Exports

### Core Types

```typescript
export type {
	FixtureMode,
	DrizzleDatabase,
	FieldResolverContext,
	FieldResolver,
	FieldResolvers,
	FixtureConfig,
	FixtureFunction,
	FixtureBuilder,
	FixtureExplanation,
	UseHelper,
} from '@jswob/drizzle-fixtures';
```

### Trait Types

```typescript
export type {
	TraitParams,
	TraitDefinition,
	TraitsConfig,
	TraitAugmentations,
	TraitAfterMakeContext,
	ComputeAugmentedType,
} from '@jswob/drizzle-fixtures';
```

### Hook Types

```typescript
export type {
	LifecycleHooks,
	BeforeMakeContext,
	AfterMakeContext,
	BeforeCreateContext,
	AfterCreateContext,
} from '@jswob/drizzle-fixtures';
```

### Context Types

```typescript
export type { UseContextOptions } from '@jswob/drizzle-fixtures';
```

### Factory Types

```typescript
export type { FixturesMap, ComposedFactory, BoundFactory } from '@jswob/drizzle-fixtures';
```

### Utility Types

```typescript
export type { Prettify } from '@jswob/drizzle-fixtures';
```

## Related Documentation

- [Main Guide](./index.md) - Core concepts and basic usage
- [Traits](./traits.md) - Trait system deep dive
- [Lifecycle Hooks](./hooks.md) - Hook details
- [Context & Composition](./context.md) - use helper and composeFactory
