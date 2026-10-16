# drizzle-fixtures Documentation

Type-safe database fixtures for Drizzle ORM with traits, relationships, and lifecycle hooks.

## Table of Contents

- [Introduction](#introduction)
- [Core Concepts](#core-concepts)
- [Basic Usage](#basic-usage)
- [Field Resolvers](#field-resolvers)
- [Resolver Override Behavior](#resolver-override-behavior)
- [Traits](#traits)
- [Lifecycle Hooks](#lifecycle-hooks)
- [Fixture Composition](#fixture-composition)
- [Factory Composition](#factory-composition)
- [Transaction-Based Cleanup](#transaction-based-cleanup)
- [TypeScript Support](#typescript-support)
- [Migration from v1.x](#migration-from-v1x)

## Introduction

`drizzle-fixtures` provides a powerful, type-safe way to create test data and database fixtures using Drizzle ORM schemas. Unlike traditional factory libraries that use string-based table references, this library directly uses your Drizzle table objects, ensuring full type safety throughout.

### Key Benefits

- **Type inference** - Field types are automatically inferred from your Drizzle schema
- **Resolver-based API** - Fields are functions that can access context (sequence, use helper)
- **Trait system** - Define reusable data variations with optional typed parameters
- **Lifecycle hooks** - Hook into every stage of data generation and persistence
- **Relationship resolution** - Build complex related data with the `use` helper
- **Transaction support** - Clean up test data using database transactions

## Core Concepts

### FixtureFunction

A fixture is created using `createFixture()` which returns a `FixtureFunction`. This function accepts a database connection (or transaction), returning a `FixtureBuilder`.

```typescript
const userFixture = createFixture<typeof users>({
	table: users,
	fields: {
		/* ... */
	},
});

// FixtureFunction: (db) => FixtureBuilder
const builder = userFixture(db);

// Or with a transaction for cleanup
await db.transaction(async tx => {
	const builder = userFixture(tx);
	// ...
});
```

### FixtureBuilder

The builder provides a fluent API for configuring and creating fixtures:

- `.trait(name, params?)` - Apply a trait
- `.build(overrides?)` - Create object in memory (not persisted)
- `.buildList(count, overrides?)` - Create multiple in-memory objects
- `.create(overrides?)` - Create and persist to database
- `.createList(count, overrides?)` - Create and persist multiple objects
- `.dryRun()` - Preview without side effects
- `.explain()` - Get configuration explanation

## Basic Usage

### Creating a Fixture

```typescript
import { createFixture } from '@jswob/drizzle-fixtures';
import { users } from './schema';
import { db } from './db';

const userFixture = createFixture<typeof users>({
	table: users,
	fields: {
		name: () => 'Test User',
		email: ({ sequence }) => `user${sequence}@example.com`,
		role: () => 'user',
	},
});
```

### Building vs Creating

```typescript
// build() - Creates object in memory only (no database insert)
const memoryUser = await userFixture(db).build();

// create() - Creates object AND persists to database
const dbUser = await userFixture(db).create();

// Override specific fields
const customUser = await userFixture(db).build({
	name: 'John Doe',
	email: 'john@example.com',
});

// Create multiple records
const users = await userFixture(db).createList(5);
```

## Field Resolvers

Field resolvers are functions that generate field values. They receive a context object with:

- `sequence` - Auto-incrementing number (unique per fixture call)
- `use` - Helper to compose with other fixtures

### Static Values

```typescript
fields: {
  role: () => 'user',
  isActive: () => true,
  createdAt: () => new Date(),
}
```

### Using Sequence

```typescript
fields: {
  email: ({ sequence }) => `user${sequence}@example.com`,
  username: ({ sequence }) => `user_${sequence}`,
}
```

### Using Other Fixtures (Relationships)

```typescript
const postFixture = createFixture<typeof posts>({
	table: posts,
	fields: {
		title: () => 'My Post',
		content: () => 'Post content here',
		// Use create() to persist the user for FK constraint
		userId: async ({ use }) => {
			const user = await use(userFixture).create();
			return user.id;
		},
	},
});
```

**Important**: When resolving foreign key relationships, use `create()` instead of `build()` to ensure the referenced record exists in the database.

## Resolver Override Behavior

Each field resolver runs **at most once**. When the same field is defined in multiple places, only the highest priority definition executes:

1. **Direct values** passed to `build()`/`create()` - highest priority, no resolver runs
2. **Trait field resolvers** - override base field resolvers
3. **Base field resolvers** - only run if not overridden

```typescript
// Example: base resolver for 'role' NEVER runs, only trait resolver executes
await fixture(db).trait('admin').build();

// Example: NO resolver runs at all, direct value is used
await fixture(db).trait('admin').build({ role: 'custom' });
```

This behavior prevents unnecessary computation and ensures predictable field resolution.

## Traits

Traits define reusable data variations. They can override field resolvers and optionally accept typed parameters.

### Basic Trait Definition

```typescript
type UserTraits = {
	admin: never; // No parameters
	withRole: { role: string }; // Required parameter
};

const userFixture = createFixture<typeof users, UserTraits>({
	table: users,
	fields: {
		name: () => 'Regular User',
		email: ({ sequence }) => `user${sequence}@example.com`,
		role: () => 'user',
	},
	traits: {
		admin: {
			fields: {
				role: () => 'admin',
			},
		},
		withRole: {
			fields: {
				role: () => 'custom',
			},
			afterMake: ({ params }) => {
				console.log('Applied role:', params.role);
				return {};
			},
		},
	},
});
```

### Applying Traits

```typescript
// Trait without parameters
const adminUser = await userFixture(db).trait('admin').build();

// Trait with parameters: role is 'custom'; the params reach only the trait's afterMake
const custom = await userFixture(db).trait('withRole', { role: 'moderator' }).build();

// Combine multiple traits
const complexUser = await userFixture(db)
	.trait('admin')
	.trait('withRole', { role: 'super-admin' })
	.build({ name: 'John Doe' });
```

For more details, see [Traits Documentation](./traits.md).

## Lifecycle Hooks

Hooks allow you to execute code at different stages of fixture creation:

```typescript
const userFixture = createFixture<typeof users>({
	table: users,
	fields: {
		/* ... */
	},
	hooks: {
		beforeMake: ({ mode }) => {
			console.log('Mode:', mode); // 'build' or 'create'
		},
		afterMake: ({ data, mode, use }) => {
			console.log('Generated:', data);
		},
		beforeCreate: ({ data }) => {
			console.log('About to insert:', data);
		},
		afterCreate: ({ data }) => {
			console.log('Inserted:', data);
		},
	},
});
```

For more details, see [Hooks Documentation](./hooks.md).

## Fixture Composition

The `use` helper enables building complex relationships between fixtures.

### Basic Composition

```typescript
const userFixture = createFixture<typeof users>({
	table: users,
	fields: {
		name: () => 'Author',
		email: ({ sequence }) => `author${sequence}@example.com`,
	},
});

const postFixture = createFixture<typeof posts>({
	table: posts,
	fields: {
		title: () => 'My Post',
		content: () => 'Content here',
		userId: async ({ use }) => {
			const user = await use(userFixture).create();
			return user.id;
		},
	},
});

// Creating a post automatically creates a user
const post = await postFixture(db).create();
```

For more details, see [Context Documentation](./context.md).

## Factory Composition

Use `composeFactory` to manage multiple fixtures bound to a database connection:

```typescript
import { composeFactory } from '@jswob/drizzle-fixtures';

const factory = composeFactory({
	user: userFixture,
	post: postFixture,
	comment: commentFixture,
});

// Create a session bound to database
const session = factory(db);

// Use fixtures through the session
const user = await session.user.create();
const post = await session.post.create({ userId: user.id });
```

For more details, see [Context Documentation](./context.md).

## Transaction-Based Cleanup

For test isolation, wrap your tests in a transaction and rollback:

```typescript
describe('User Posts', () => {
	it('creates posts for user', async () => {
		await db.transaction(async tx => {
			// Bind factory to transaction
			const session = factory(tx);

			// Create fixtures - all operations use the transaction
			const user = await session.user.create();
			const post = await session.post.create({ userId: user.id });

			// Run assertions
			expect(post.userId).toBe(user.id);

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

## TypeScript Support

The library provides full type safety based on your Drizzle schemas.

### Schema-Based Inference

```typescript
import { pgTable, uuid, varchar } from 'drizzle-orm/pg-core';

export const users = pgTable('users', {
	id: uuid('id').primaryKey().defaultRandom(),
	name: varchar('name', { length: 255 }).notNull(),
	role: varchar('role', { length: 50 }).notNull(),
});

const userFixture = createFixture<typeof users>({
	table: users,
	fields: {
		name: () => 'Test User', // Must return string
		role: () => 'user', // Must return string
		// invalidField: () => 'test',  // TypeScript error
	},
});

const user = await userFixture(db).build();
// user.id: string
// user.name: string
// user.role: string
```

### Type-Safe Traits

```typescript
interface UserTraits {
	admin: never; // No parameters
	withOrg: { orgId: string }; // Required parameter
}

const userFixture = createFixture<typeof users, UserTraits>({
	table: users,
	fields: {
		/* ... */
	},
	traits: {
		admin: {
			/* ... */
		},
		withOrg: {
			/* ... */
		},
	},
});

// Type-safe trait application
userFixture(db).trait('admin').build(); // OK
userFixture(db).trait('withOrg', { orgId: '1' }); // OK - params required
userFixture(db).trait('withOrg'); // Error: params missing
userFixture(db).trait('invalid'); // Error: trait not found
```

### Trait Augmentations

Traits can augment the return type:

```typescript
type UserAugmentations = {
	withPosts: { posts: Post[] };
};

const userFixture = createFixture<typeof users, UserTraits, UserAugmentations>({
	// ...
	traits: {
		withPosts: {
			afterMake: async ({ use }) => {
				const posts = await use(postFixture).buildList(3);
				return { posts }; // Typed augmentation
			},
		},
	},
});

const user = await userFixture(db).trait('withPosts').build();

// user.posts is typed as Post[]
```

## Migration from v1.x

If you were using `RollbackTracker` in v1.x, migrate to transaction-based cleanup:

**Before (v1.x):**

```typescript
const rollbackTracker = new RollbackTracker(db);

const user = await userFixture(db, { rollbackTracker }).create();
const post = await postFixture(db, { rollbackTracker }).create({ userId: user.id });

// Cleanup
await rollbackTracker.rollback();
```

**After (v2.x):**

```typescript
await db.transaction(async tx => {
	const user = await userFixture(tx).create();
	const post = await postFixture(tx).create({ userId: user.id });

	// ... test assertions ...

	// Cleanup via transaction rollback
	tx.rollback();
});
```

### Removed APIs

The following APIs have been removed in v2.x:

- `RollbackTracker` class - Use `db.transaction()` instead
- `session.rollback()` - Use `tx.rollback()` instead
- `session.clear()` - No longer needed
- `session.getRollbackTracker()` - No longer needed
- `onRollback` lifecycle hook - No longer supported
- `rollbackTracker` option in fixture function - No longer needed

## Related Documentation

- [Traits](./traits.md) - Deep dive into the trait system
- [Lifecycle Hooks](./hooks.md) - Hook into fixture creation stages
- [Context & Composition](./context.md) - use helper, composeFactory
- [API Reference](./api-reference.md) - Complete API documentation
