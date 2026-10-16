# Lifecycle Hooks

Lifecycle hooks allow you to execute code at specific stages of fixture creation. They're useful for logging, validation, side effects, and transformations.

## Table of Contents

- [Hook Overview](#hook-overview)
- [beforeMake](#beforemake)
- [afterMake](#aftermake)
- [beforeCreate](#beforecreate)
- [afterCreate](#aftercreate)
- [Complete Example](#complete-example)

## Hook Overview

| Hook           | When Called             | Mode         | Use Case                |
| -------------- | ----------------------- | ------------ | ----------------------- |
| `beforeMake`   | Before field resolution | build/create | Setup, validation       |
| `afterMake`    | After field resolution  | build/create | Transform data, logging |
| `beforeCreate` | Before database insert  | create only  | Final validation        |
| `afterCreate`  | After successful insert | create only  | Post-creation setup     |

## beforeMake

Called before any field resolvers execute.

```typescript
hooks: {
  beforeMake: ({ mode }) => {
    console.log('About to generate data');
    console.log('Mode:', mode); // 'build' or 'create'
  },
}
```

**Context:**

- `mode` - Whether this is a `'build'` or `'create'` operation

## afterMake

Called after all field resolvers have executed but before database insert. `data` is frozen and
the hook returns nothing: it observes the data, it cannot change it. To add to the result, use a
trait's `afterMake`, whose return value is merged in (see [Traits](./traits.md)).

```typescript
hooks: {
  afterMake: ({ data, mode, use }) => {
    console.log('Generated data:', data);
    console.log('Mode:', mode);

    // Can use other fixtures if needed
    // const related = await use(otherFixture).build();
  },
}
```

**Context:**

- `data` - The generated fixture data
- `mode` - Whether this is a `'build'` or `'create'` operation
- `use` - Helper to compose with other fixtures

## beforeCreate

Called just before inserting into the database. Only runs for `create()` operations.

```typescript
hooks: {
  beforeCreate: ({ data }) => {
    console.log('About to insert:', data.id);

    // Good place for final validation
    if (!data.email.includes('@')) {
      throw new Error('Invalid email');
    }
  },
}
```

**Context:**

- `data` - The data about to be inserted

## afterCreate

Called after successful database insert. Only runs for `create()` operations.

```typescript
hooks: {
  afterCreate: ({ data }) => {
    console.log('Successfully created:', data.id);

    // Good place for post-creation setup
    // e.g., creating related records, sending notifications
  },
}
```

**Context:**

- `data` - The inserted data (includes database-generated fields like `id`)

## Complete Example

```typescript
const userFixture = createFixture<typeof users>({
	table: users,
	fields: {
		name: () => 'Test User',
		email: ({ sequence }) => `user${sequence}@example.com`,
		status: () => 'pending',
	},
	hooks: {
		beforeMake: ({ mode }) => {
			console.log(`[beforeMake] Creating user (${mode} mode)`);
		},

		afterMake: ({ data, mode }) => {
			console.log(`[afterMake] User data ready:`, data.name);
			if (mode === 'create') {
				console.log('[afterMake] Will be persisted to database');
			}
		},

		beforeCreate: ({ data }) => {
			console.log(`[beforeCreate] Inserting:`, data.email);
		},

		afterCreate: ({ data }) => {
			console.log(`[afterCreate] Created with ID:`, data.id);
		},
	},
});

// Using build() triggers: beforeMake -> afterMake
const memUser = await userFixture(db).build();

// Using create() triggers: beforeMake -> afterMake -> beforeCreate -> afterCreate
const dbUser = await userFixture(db).create();
```

### Hook Execution Flow

**For `build()` operations:**

```
1. beforeMake({ mode: 'build' })
2. [field resolvers execute]
3. afterMake({ data, mode: 'build', use })
4. Return result
```

**For `create()` operations:**

```
1. beforeMake({ mode: 'create' })
2. [field resolvers execute]
3. afterMake({ data, mode: 'create', use })
4. beforeCreate({ data })
5. [database insert]
6. afterCreate({ data })
7. Return result
```

### Using Hooks with Transactions

Hooks work seamlessly with transaction-based cleanup:

```typescript
await db.transaction(async tx => {
	// Hooks execute within the transaction context
	const user = await userFixture(tx).create();

	// All hook operations use the same transaction
	// Rollback cleans up everything including hook side effects
	tx.rollback();
});
```

## Related Documentation

- [Main Guide](./index.md) - Core concepts and basic usage
- [Traits](./traits.md) - Trait-specific hooks with `afterMake`
- [Context & Composition](./context.md) - Transaction-based cleanup
- [API Reference](./api-reference.md) - Complete hook type definitions
