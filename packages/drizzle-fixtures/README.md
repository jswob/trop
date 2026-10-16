# @jswob/drizzle-fixtures

Type-safe database fixtures for Drizzle ORM with traits, relationships, and lifecycle hooks.

## Features

- **Type-safe** - Automatically typed based on your Drizzle table schemas
- **Trait system** - Define reusable data variations with typed parameters
- **Lifecycle hooks** - Control data generation and persistence at every step
- **Relationship composition** - Build complex related data with the `use` helper
- **Transaction support** - Clean up test data using database transactions
- **Factory composition** - Manage multiple fixtures bound to a database connection

## Installation

```bash
bun add @jswob/drizzle-fixtures
```

## Quick Start

```typescript
import { createFixture, composeFactory } from '@jswob/drizzle-fixtures';
import { users, posts } from './schema';
import { db } from './db';

// Define fixtures
const userFixture = createFixture<typeof users>({
	table: users,
	fields: {
		name: () => 'Test User',
		email: ({ sequence }) => `user${sequence}@example.com`,
		role: () => 'user',
	},
});

const postFixture = createFixture<typeof posts>({
	table: posts,
	fields: {
		title: () => 'Test Post',
		userId: ({ use }) =>
			use(userFixture)
				.create()
				.then(u => u.id),
	},
});

// Compose into a factory
const factory = composeFactory({
	user: userFixture,
	post: postFixture,
});

// Create in-memory object
const user = await userFixture(db).build();

// Create and persist to database
const savedUser = await userFixture(db).create();

// Create multiple records
const users = await userFixture(db).createList(5);
```

## Transaction-Based Cleanup

For test isolation, wrap your tests in a transaction and rollback:

```typescript
import { composeFactory } from '@jswob/drizzle-fixtures';

describe('my tests', () => {
	it('should create user', async () => {
		await db.transaction(async tx => {
			// Bind factory to transaction
			const session = factory(tx);

			// Create fixtures - all operations use the transaction
			const user = await session.user.create();
			const post = await session.post.create({ userId: user.id });

			// Run assertions
			expect(user.name).toBe('Test User');

			// Transaction rollback cleans up all created data
			tx.rollback();
		});
		// Data is automatically cleaned up - nothing persists
	});
});
```

### Why Transactions?

Using `db.transaction()` with rollback provides:

- **Guaranteed cleanup** - Database handles deletion order automatically
- **Isolation** - Tests don't interfere with each other
- **Performance** - No manual DELETE queries needed
- **Simplicity** - No tracking mechanism required

## API Overview

### createFixture(config)

Creates a fixture function for a Drizzle table.

```typescript
const userFixture = createFixture<typeof users>({
	table: users,
	fields: {
		name: () => 'Default Name',
		email: ({ sequence }) => `user${sequence}@test.com`,
	},
	traits: {
		admin: {
			fields: { role: 'admin' },
		},
	},
	hooks: {
		beforeCreate: ({ data }) => console.log('Creating:', data),
	},
});
```

### composeFactory(fixtures)

Composes multiple fixtures into a factory that can be bound to a database.

```typescript
const factory = composeFactory({
	user: userFixture,
	post: postFixture,
});

const session = factory(db); // or factory(tx) for transactions
const user = await session.user.create();
```

### FixtureBuilder Methods

- `.trait(name, params?)` - Apply a trait
- `.build(overrides?)` - Create in-memory object
- `.create(overrides?)` - Create and persist to database
- `.buildList(count, overrides?)` - Create multiple in-memory objects
- `.createList(count, overrides?)` - Create and persist multiple records

## Migration from v1.x (RollbackTracker)

If you were using `RollbackTracker`, migrate to transactions:

**Before:**

```typescript
const rollbackTracker = new RollbackTracker(db);
const user = await userFixture(db, { rollbackTracker }).create();
await rollbackTracker.rollback();
```

**After:**

```typescript
await db.transaction(async tx => {
	const user = await userFixture(tx).create();
	// ... test code ...
	tx.rollback();
});
```

### Removed APIs

- `RollbackTracker` class - Use `db.transaction()` instead
- `session.rollback()` - Use `tx.rollback()` instead
- `session.clear()` - No longer needed
- `session.getRollbackTracker()` - No longer needed
- `onRollback` lifecycle hook - No longer supported

## Documentation

For comprehensive documentation, see:

- **[Main Guide](./docs/index.md)** - Core concepts, basic usage, and feature overview
- **[Traits](./docs/traits.md)** - Defining and using traits for data variations
- **[Lifecycle Hooks](./docs/hooks.md)** - beforeMake, afterMake, beforeCreate, afterCreate
- **[Context & Composition](./docs/context.md)** - use helper, composeFactory

## License

MIT
