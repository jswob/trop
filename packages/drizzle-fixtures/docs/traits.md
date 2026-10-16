# Traits

Traits define reusable data variations that can be applied to fixtures. They provide a clean way to create different "flavors" of test data without duplicating fixture definitions.

## Table of Contents

- [Defining Trait Types](#defining-trait-types)
- [Basic Traits](#basic-traits)
- [Traits with Parameters](#traits-with-parameters)
- [Trait Augmentations](#trait-augmentations)
- [Applying Traits](#applying-traits)
- [Chaining Traits](#chaining-traits)
- [Trait Field Resolution](#trait-field-resolution)

## Defining Trait Types

Traits are defined using a TypeScript interface where:

- Keys are trait names
- Values are either `never` (no parameters) or an object type (required parameters)

```typescript
type UserTraits = {
	admin: never; // No parameters
	verified: never; // No parameters
	withRole: { role: string }; // Required parameter
	withAge: { age: number }; // Required parameter
};
```

## Basic Traits

Traits without parameters override field resolvers:

```typescript
const userFixture = createFixture<typeof users, UserTraits>({
	table: users,
	fields: {
		name: () => 'Regular User',
		email: ({ sequence }) => `user${sequence}@example.com`,
		role: () => 'user',
		isVerified: () => false,
	},
	traits: {
		admin: {
			fields: {
				role: () => 'admin',
			},
		},
		verified: {
			fields: {
				isVerified: () => true,
			},
		},
	},
});

// Usage
const admin = await userFixture(db).trait('admin').build();
// admin.role === 'admin'
```

## Traits with Parameters

Traits can accept typed parameters. Only the trait's `afterMake` receives them: a trait's field
resolvers get `{ use, sequence }`, like the base fields. A field whose value the caller picks is an
override (`.build({ role: 'moderator' })`), not a trait parameter.

```typescript
type UserTraits = {
	withPosts: { count: number };
};

const userFixture = createFixture<typeof users, UserTraits>({
	table: users,
	fields: {
		/* ... */
	},
	traits: {
		withPosts: {
			afterMake: async ({ data, params, use }) => {
				for (let i = 0; i < params.count; i++) {
					await use(postFixture).create({ userId: data.id });
				}
				return {};
			},
		},
	},
});

// Parameters are required
const author = await userFixture(db).trait('withPosts', { count: 3 }).create();
```

## Trait Augmentations

Traits can augment the returned type with additional data:

```typescript
type UserTraits = {
	withPosts: { count?: number };
	withProfile: never;
};

type UserAugmentations = {
	withPosts: { posts: Post[] };
	withProfile: { profile: Profile };
};

const userFixture = createFixture<typeof users, UserTraits, UserAugmentations>({
	table: users,
	fields: {
		/* ... */
	},
	traits: {
		withPosts: {
			afterMake: async ({ data, params, use }) => {
				const count = params?.count ?? 3;
				const posts = await Promise.all(
					Array.from({ length: count }, () => use(postFixture).create({ userId: data.id }))
				);
				return { posts }; // This becomes user.posts
			},
		},
		withProfile: {
			afterMake: async ({ data, use }) => {
				const profile = await use(profileFixture).create({ userId: data.id });
				return { profile }; // This becomes user.profile
			},
		},
	},
});

// Return type includes augmented data
const user = await userFixture(db).trait('withPosts', { count: 5 }).build();

console.log(user.posts); // Post[] (typed!)
console.log(user.posts.length); // 5
```

## Applying Traits

Use the `.trait()` method to apply traits:

```typescript
// Without parameters
await userFixture(db).trait('admin').build();

// With parameters
await userFixture(db).trait('withRole', { role: 'moderator' }).build();
```

## Chaining Traits

Multiple traits can be chained:

```typescript
const user = await userFixture(db)
	.trait('admin')
	.trait('verified')
	.trait('withPosts', { count: 2 })
	.build({ name: 'John Doe' }); // Can still override fields

// user is: admin + verified + has 2 posts + name is 'John Doe'
```

When traits define the same field, **later traits win**:

```typescript
const user = await userFixture(db)
	.trait('admin') // role: 'admin'
	.trait('withRole', { role: 'super' }) // role: 'super' (wins)
	.build();

// user.role === 'super'
```

## Trait Field Resolution

Understanding how trait fields interact with base fields:

1. **Direct values** (passed to `build()`/`create()`) - highest priority
2. **Trait field resolvers** - override base resolvers
3. **Base field resolvers** - only run if not overridden

```typescript
const fixture = createFixture<typeof users, { admin: never }>({
	table: users,
	fields: {
		role: () => {
			console.log('Base resolver'); // Only runs if no trait/override
			return 'user';
		},
	},
	traits: {
		admin: {
			fields: {
				role: () => {
					console.log('Trait resolver'); // Runs when trait applied
					return 'admin';
				},
			},
		},
	},
});

// Case 1: No trait - base resolver runs
await fixture(db).build();
// Output: "Base resolver"

// Case 2: Trait applied - only trait resolver runs
await fixture(db).trait('admin').build();
// Output: "Trait resolver"

// Case 3: Direct value - NO resolver runs
await fixture(db).trait('admin').build({ role: 'custom' });
// Output: (nothing - role is 'custom')
```

This design ensures efficient execution - resolvers that would be overridden never execute.

## Related Documentation

- [Main Guide](./index.md) - Core concepts and basic usage
- [Lifecycle Hooks](./hooks.md) - Hook into fixture creation stages
- [Context & Composition](./context.md) - use helper and composition
- [API Reference](./api-reference.md) - Complete API documentation
