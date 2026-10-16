# Context & Composition

This document covers the context utilities that enable fixture composition: the `use` helper, `composeFactory`, and transaction-based cleanup patterns.

## Table of Contents

- [The use Helper](#the-use-helper)
- [composeFactory](#composefactory)
- [Transaction-Based Cleanup](#transaction-based-cleanup)

## The use Helper

The `use` helper enables fixtures to compose with other fixtures. It's available in field resolvers and hooks.

### Basic Usage

```typescript
const postFixture = createFixture<typeof posts>({
	table: posts,
	fields: {
		title: () => 'My Post',
		content: () => 'Post content',
		userId: async ({ use }) => {
			const user = await use(userFixture).create();
			return user.id;
		},
	},
});
```

### Propagating Database Context

When you use `use()`, child fixtures automatically use the same database connection (or transaction):

```typescript
await db.transaction(async tx => {
	// Creating a post also creates a user via use()
	// Both operations use the same transaction
	const post = await postFixture(tx).create();

	// Transaction rollback cleans up both post AND user
	tx.rollback();
});
```

### Builder Methods

The `use()` helper returns a fixture builder with all standard methods:

```typescript
userId: async ({ use }) => {
	// Create and persist
	const user = await use(userFixture).create();

	// Or build in memory only
	const memUser = await use(userFixture).build();

	// Apply traits
	const admin = await use(userFixture).trait('admin').create();

	// Create multiple
	const users = await use(userFixture).createList(3);

	return user.id;
};
```

### In Hooks

The `use` helper is also available in `afterMake`:

```typescript
hooks: {
  afterMake: async ({ data, use }) => {
    // Create related data after the main fixture is built
    const profile = await use(profileFixture).create({
      userId: data.id,
    });
    return { profile };
  },
}
```

## composeFactory

The `composeFactory` function creates a factory that manages multiple fixtures bound to a database connection.

### Creating a Factory

```typescript
import { composeFactory } from '@jswob/drizzle-fixtures';

const factory = composeFactory({
	user: userFixture,
	post: postFixture,
	comment: commentFixture,
});
```

### Creating a Session

Each factory call creates a session bound to the database connection:

```typescript
const session = factory(db);

// Use fixtures through the session
const user = await session.user.create();
const post = await session.post.create({ userId: user.id });
const comment = await session.comment.create({
	postId: post.id,
	authorId: user.id,
});
```

### With Transactions

For test cleanup, bind the factory to a transaction:

```typescript
await db.transaction(async tx => {
	const session = factory(tx);

	const user = await session.user.create();
	const post = await session.post.create({ userId: user.id });

	// All fixtures use the transaction
	// Rollback cleans up everything
	tx.rollback();
});
```

### Multiple Independent Sessions

```typescript
const session1 = factory(db);
const session2 = factory(db);

// These are completely independent
await session1.user.create();
await session2.user.create();
```

## Transaction-Based Cleanup

The recommended approach for test data cleanup is to use database transactions with rollback.

### Why Transactions?

Using `db.transaction()` with rollback provides:

- **Guaranteed cleanup** - Database handles deletion order automatically
- **Isolation** - Tests don't interfere with each other
- **Performance** - No manual DELETE queries needed
- **Simplicity** - No tracking mechanism required

### Typical Test Pattern

```typescript
import { describe, it, expect } from 'vitest';
import { db } from './db';
import { factory } from './fixtures';

describe('User Posts', () => {
	it('creates posts for user', async () => {
		await db.transaction(async tx => {
			// Bind factory to transaction
			const session = factory(tx);

			// Create test data
			const user = await session.user.create();
			const post = await session.post.create({ userId: user.id });

			// Run assertions
			expect(post.userId).toBe(user.id);

			// Transaction rollback cleans up all created data
			tx.rollback();
		});
		// Data is automatically cleaned up - nothing persists
	});

	it('creates admin user', async () => {
		await db.transaction(async tx => {
			const session = factory(tx);

			const admin = await session.user.trait('admin').create();

			expect(admin.role).toBe('admin');

			tx.rollback();
		});
	});
});
```

### Using Individual Fixtures

You can also use individual fixtures directly with transactions:

```typescript
await db.transaction(async tx => {
	const user = await userFixture(tx).create();
	const post = await postFixture(tx).create({ userId: user.id });

	// Run assertions
	expect(post.userId).toBe(user.id);

	tx.rollback();
});
```

### Shared Setup with beforeEach

For tests that need common setup:

```typescript
describe('Post operations', () => {
	it('test 1', async () => {
		await db.transaction(async tx => {
			const session = factory(tx);
			const user = await session.user.create();
			// ... test code ...
			tx.rollback();
		});
	});

	it('test 2', async () => {
		await db.transaction(async tx => {
			const session = factory(tx);
			const user = await session.user.create();
			// ... test code ...
			tx.rollback();
		});
	});
});
```

### Helper Function Pattern

For cleaner test code, create a helper function:

```typescript
async function withTestData<T>(
	callback: (session: ReturnType<typeof factory>) => Promise<T>
): Promise<T> {
	return db.transaction(async tx => {
		const session = factory(tx);
		try {
			return await callback(session);
		} finally {
			tx.rollback();
		}
	});
}

// Usage
it('creates user', async () => {
	await withTestData(async session => {
		const user = await session.user.create();
		expect(user.name).toBe('Test User');
	});
});
```

## Migration from v1.x

If you were using `RollbackTracker` or session methods like `rollback()`, `clear()`, or `getRollbackTracker()`, migrate to the transaction pattern:

**Before (v1.x):**

```typescript
const session = factory(db);
const user = await session.user.create();
await session.rollback(); // Manually tracked cleanup
```

**After (v2.x):**

```typescript
await db.transaction(async tx => {
	const session = factory(tx);
	const user = await session.user.create();
	// ... test code ...
	tx.rollback(); // Transaction handles cleanup
});
```

See the [Migration Guide](./index.md#migration-from-v1x) in the main documentation for more details.

## Related Documentation

- [Main Guide](./index.md) - Core concepts and basic usage
- [Traits](./traits.md) - Using traits with composed fixtures
- [Lifecycle Hooks](./hooks.md) - Hook into fixture creation stages
- [API Reference](./api-reference.md) - Complete API documentation
