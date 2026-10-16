# Query Efficiency - Round Trips, Batching, Bulk Writes

How to shape the database work of a route or service so its cost does not grow with the data.

**When to Use:** Writing or reviewing any code that reads or writes more than one row or more than one table — list endpoints, nested reads (parents with children), record saves, copies, bulk edits — and whenever an endpoint feels slow.

**Why this matters:** the database is in another building. Every statement is a network round trip of a few milliseconds, and the queries themselves run in well under a millisecond, so the number of statements a request sends, one after another, _is_ its response time. Drizzle over postgres.js sends its queries unprepared, and an unprepared statement that carries parameters costs **two** round trips (the driver describes it first, then runs it) and holds its connection meanwhile. Prepared — `withPreparedStatements` from `@jswob/drizzle-utils` turns that on for everything Drizzle sends — a statement pays the describe once per connection and is one round trip after that; `PROJECT.md` says which this project does. Either way the count is the cost: a list that sends three statements per row sends 1,500 for 500 rows and takes seconds, while the same list read as a handful of statements sent together answers in tens of milliseconds. Indexes and caching are rarely the problem.

## Table of Contents

- [The Rule](#the-rule)
- [Read a Tree in One Pass](#read-a-tree-in-one-pass)
- [Send Independent Queries Together](#send-independent-queries-together)
- [Bulk Writes](#bulk-writes)
- [Check First, Then Write](#check-first-then-write)
- [Do Not Ask Twice](#do-not-ask-twice)
- [Transactions and Helpers](#transactions-and-helpers)
- [Order, Columns, Indexes](#order-columns-indexes)
- [Checklist](#checklist)
- [Deep Dive](#deep-dive)

---

## The Rule

**The number of statements a request sends must not depend on how many rows it handles.** One post or fifty, one tag or five hundred: the same count. A query budget test pins it (TESTING.md § Query Budget Tests).

A query is in the wrong place when it is awaited inside a `for`, a `.map()`, or any helper called once per row:

```typescript
// N+1: three statements per post (author, tags, comments), 1,500 for a list of 500
for (const post of postRows) {
	const details = await getPostDetails(db, post.id);
}

// Still N+1. Promise.all makes the statements overlap, it does not make them fewer:
// the pool has a fixed number of connections (postgres.js: 10 by default), so the rest queue behind them.
await Promise.all(postRows.map(post => getPostDetails(db, post.id)));
```

The same goes for writes: an `insert`, `update` or `delete` per row is one statement per row. Read the set once, decide in memory, write the set once.

> **Caveat:** A helper that takes one id and queries is an N+1 waiting for its first loop. Give data helpers a **set** to work on (a scope or a list) from the start — `getPostsWithDetails(db, scope)`, `copyPostsToAuthors(tx, targets)`, `insertPostTags(tx, posts)` — and let the single case pass a set of one.

---

## Read a Tree in One Pass

**Key Concepts:**

- One query per **table**, never per parent row
- Scope the children by the parent's own condition — a join or `inArray(column, subquery)` — not by a list of ids fetched a moment ago
- Reads scoped that way do not depend on each other, so they go out together (next section)
- Group the rows in memory with a `Map`

```typescript
// src/modules/posts/post-helpers.ts — posts with their author's name and tag names
export async function getPostsWithDetails(db: DatabaseConnection, scope: SQL | undefined) {
	const [postRows, tagRows, commentRows] = await Promise.all([
		db
			.select({ post: posts, authorName: authors.name }) // the name comes with the row
			.from(posts)
			.innerJoin(authors, eq(posts.authorId, authors.id))
			.where(scope),
		db
			.select({ postId: postTags.postId, tagName: tags.name })
			.from(postTags)
			.innerJoin(posts, eq(postTags.postId, posts.id)) // the scope reaches the children
			.innerJoin(tags, eq(postTags.tagId, tags.id))
			.where(scope)
			.orderBy(asc(tags.name)),
		// ...comments, the same way
	]);

	const tagNamesByPostId = new Map<string, string[]>();
	for (const row of tagRows) {
		const list = tagNamesByPostId.get(row.postId) ?? [];
		list.push(row.tagName);
		tagNamesByPostId.set(row.postId, list);
	}
	// ...assemble
}

// Callers say which posts; the query count stays three
await getPostsWithDetails(db, eq(posts.authorId, authorId));
await getPostsWithDetails(db, isNotNull(posts.publishedAt));
```

**Choosing the scope:**

| The set of parents is...            | Scope the children with                                                   |
| ----------------------------------- | ------------------------------------------------------------------------- |
| every row (a list of everything)    | no filter at all — read each table whole                                  |
| rows matching a condition           | the same condition through a join, or `inArray(child.parentId, subquery)` |
| ids the request sent (body, params) | `inArray(column, ids)` — guard the empty list                             |
| ids you just read from the database | not a list: use the condition that found them (rows above)                |

```typescript
import { alias } from 'drizzle-orm/pg-core';

// A subquery keeps the statement text and its one parameter the same for any number of rows
const taggedPostIds = db
	.select({ id: postTags.postId })
	.from(postTags)
	.where(eq(postTags.tagId, tagId));

await db
	.select({ postId: comments.postId, count: count() })
	.from(comments)
	.where(inArray(comments.postId, taggedPostIds))
	.groupBy(comments.postId); // one grouped count, not a count() per post

// The same table joined twice: who wrote each comment, and whose post it is on
const postAuthor = alias(authors, 'post_author');
await db
	.select({ comment: comments, commenterName: authors.name, postAuthorName: postAuthor.name })
	.from(comments)
	.innerJoin(authors, eq(comments.authorId, authors.id))
	.innerJoin(posts, eq(comments.postId, posts.id))
	.innerJoin(postAuthor, eq(posts.authorId, postAuthor.id))
	.where(inArray(comments.postId, taggedPostIds));
```

**Names and other lookups** come with the row through a join (`alias()` when the same table is joined twice), not from a second query. **Counts per parent** are one `groupBy`, not a `count()` in a loop — as a grouped query of its own beside the parents' (it keeps the parents' query plain), or folded into the parents' query as a left-joined aggregate. Both are a fixed number of statements; pick the one that reads better.

`getPostsWithDetails` is a helper, so it returns the rows. A service function that a route calls still returns `ServiceResult<T>` (ELYSIA.md § Service Layer Pattern), even for a read that cannot fail.

> **Caveat:** `inArray(column, [])` with an empty array — skip the query (`ids.length > 0 ? await ... : []`). `.values([])` on an insert throws.

---

## Send Independent Queries Together

**Key Concepts:**

- Queries that do not need each other's results go in one `Promise.all` — one wait instead of one per query
- On the pool (`db`) they overlap on separate connections: five reads take the time of one (measured: 8 ms together, 37 ms in a row)
- Inside a transaction (`tx`) there is one connection. Unprepared, statements with parameters run on it one after another, so `Promise.all` there saves no time; prepared, they go out together. Either way it says the statements are independent, and **in a transaction the number of statements is what counts**
- Judge the results in the order the checks must fail, after they are all in — the order of the reads stops mattering

```typescript
// Before: three waits, each read needing only the request's own ids
const [post] = await db.select(...).from(posts)...;     // then
const [tag] = await db.select().from(tags)...;          // then
const [link] = await db.select().from(postTags)...;

// After: one wait. The existing link rides on the post's query as a left join.
const [[post], [tag]] = await Promise.all([
	db
		.select({ postId: posts.id, linkedTagId: postTags.tagId /* ... */ })
		.from(posts)
		.leftJoin(
			postTags,
			and(
				eq(postTags.postId, posts.id),
				eq(postTags.tagId, tagId)
			)
		)
		.where(eq(posts.id, postId)),
	db.select({ id: tags.id, archived: tags.archived }).from(tags)...,
]);

if (!post) return fail('NOT_FOUND', 'Post not found'); // same order of refusals as before
if (!tag) return fail('NOT_FOUND', 'Tag not found');
if (post.linkedTagId !== null) return fail('CONFLICT', 'Post already has this tag');
```

**When not to:**

- A statement that needs another's result or effect waits for it: delete before the insert that reuses a unique key, parents before children, a write before `markAuthorStatsStale`.
- Two helpers that each open a transaction (a savepoint, when nested) do not run side by side.
- An external call (another service over HTTP) is never inside a `Promise.all` with open database work.

> **Caveat:** A chain of reads (`A`, then `B` filtered by `A`'s ids, then `C` filtered by `B`'s) is usually three independent reads in disguise: scope `B` and `C` by `A`'s condition (previous section) and the chain becomes one wait.

---

## Bulk Writes

**Key Concepts:**

- One statement per **table and kind of change** (insert / update / delete), however many rows
- Many rows, same change: `inArray` in the `where`
- Many rows, a different value each: `caseBy()` from `@jswob/drizzle-utils`
- Parents with children: mint the parents' ids in the app (a `uuid` key), then two multi-row inserts

```typescript
import { randomUUID } from 'node:crypto';
import { caseBy, dbNow } from '@jswob/drizzle-utils';

// Same change for many rows
await tx.delete(posts).where(inArray(posts.id, goneIds));
await tx.update(posts).set({ archived: true }).where(inArray(posts.id, toArchive));

// A different value per row, one statement (Drizzle's "update many with different values")
await tx
	.update(posts)
	.set({
		title: caseBy(
			posts.id,
			posts.title,
			inputs.map(p => [p.id, p.title])
		),
		sortOrder: caseBy(
			posts.id,
			posts.sortOrder,
			inputs.map(p => [p.id, p.sortOrder ?? 0])
		),
		updatedAt: dbNow(),
	})
	.where(
		inArray(
			posts.id,
			inputs.map(p => p.id)
		)
	); // the where must list exactly the keys

// Parents and children: no insert-one-read-its-id loop, no reliance on RETURNING order
const created = inputs.map(input => ({ ...input, postId: randomUUID() }));
await tx.insert(posts).values(created.map(input => ({ id: input.postId, authorId /* ... */ })));
await insertPostTags(tx, created); // the tag links of every post: one insert
```

**Replace a set** (a record's part sent whole): one delete scoped by the parent, one insert of the new rows.

**Change part of a set** (switch some rows on, others off): read the rows once, work out in memory which change, then at most one statement per direction:

```typescript
const rows = await db
	.select({ id: posts.id, archived: posts.archived, publishedAt: posts.publishedAt })
	.from(posts)
	.where(/* all the author's posts */);
const toArchive = rows.filter(row => !row.archived && isStale(row)).map(row => row.id);
const toRestore = rows.filter(row => row.archived && !isStale(row)).map(row => row.id);
await Promise.all([
	toArchive.length > 0
		? db.update(posts).set({ archived: true }).where(inArray(posts.id, toArchive))
		: undefined,
	toRestore.length > 0
		? db.update(posts).set({ archived: false }).where(inArray(posts.id, toRestore))
		: undefined,
]);
```

**Copy rows** (a post duplicated for other authors, with its tag links): read the sources once, build every copy in memory, insert once per table — `PROJECT.md` points to the project's own example. `db.insert(t).select(query)` copies inside the database in one statement, but Drizzle requires the select to list **every** column of the target in table order, so it only fits when the shapes already match.

> **Caveat:** `caseBy` casts each value to the target column's SQL type (Postgres reads an untyped `CASE` of parameters as text). A row whose key is missing from the pairs would be set to NULL, so keep the `where` to the same keys.

> **Caveat:** A multi-row statement carries one parameter per value and Postgres allows 65,535 per statement. Request-sized sets are far below it; chunk an import of thousands of rows.

---

## Check First, Then Write

A save that validates as it goes (write post 1, check post 2, write post 2...) cannot be batched, and its refusals arrive halfway through the writes. Shape every multi-row save in three steps:

1. **Read** everything the checks and the writes decide from, together (one `Promise.all`).
2. **Check** in memory, in the order refusals must be reported. Nothing has been written yet.
3. **Write** set by set (previous section).

```typescript
// src/modules/posts/record.ts — savePosts, abridged
const [movedToAuthors, knownTags, savedPosts] = await Promise.all([...]); // 1. read

for (const id of movedToAuthorIds) if (!authorsById.get(id)) refuse(`Unknown author: ${id}`); // 2. check
for (const input of entries.flatMap(entry => entry.tags)) must(checkTagInput(input, tagsByName));
for (const post of [...kept, ...added]) { /* slugs unique per author; one pinned post at most */ }

await db.delete(posts).where(inArray(posts.id, gone)); // 3. write
```

Validation helpers follow the same split: one function that **fetches** what many inputs need in one query (`fetchTagInfo(db, tagInputs)`), and a pure one that **checks** a single input against it (`checkTagInput(input, tagMap)`), called in a loop that sends nothing.

---

## Do Not Ask Twice

| Instead of...                                                          | Do                                                                            |
| ---------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| a `select` after an insert/update to build the response                | `.returning()`, joined in memory with rows already read for the checks        |
| a `select` to see the row exists, then `update` / `delete`             | the write with `.returning()`; no row back means `NOT_FOUND`                  |
| a `select` to see a name is free, then `insert`                        | the insert, catching `isUniqueViolation(error)` — the constraint is the check |
| each part of a record re-reading the same parent                       | the save checks once; the parts trust the transaction they run in             |
| a derived-data flag (`markAuthorStatsStale`) in every part of one save | once per request, in the save's transaction, after the writes                 |
| reading rows again that an earlier step of the request has read        | pass them down                                                                |

```typescript
// The update itself says whether the row exists
const [updated] = await tx
	.update(posts)
	.set({ updatedAt: dbNow() })
	.where(eq(posts.id, id))
	.returning();
if (!updated) throw new RecordRefused({ code: 'NOT_FOUND', message: 'Post not found' }); // throwing rolls the save back
```

> **Caveat:** Keep a read-before-write only when its refusal has to come before another check's (a missing post before a duplicate-slug conflict) — and then send it in the same `Promise.all` as that other check's read.

---

## Transactions and Helpers

- A save that writes more than one statement runs in **one** `db.transaction`, opened by the route's service.
- A helper that writes takes the connection it is given and opens no transaction of its own; its JSDoc says so. A `db.transaction` inside a transaction is a savepoint — two more statements, and it cannot run beside its siblings.
- Keep transactions short: no call to another service or any other HTTP request while one is open.
- Throwing rolls everything back, so "check first, then write" is about query count and clear refusals, not about atomicity.

---

## Order, Columns, Indexes

- **Order:** rows come back in no particular order unless the query has an `orderBy`. When the order is part of the answer (tags by name, comments by `createdAt`, posts by publish date), say it in the query.
- **Columns:** an existence check selects `{ id }`, not the row. A list endpoint that ships every column of every nested row grows with the data too — a few hundred records with their children easily reach hundreds of KB of JSON — so select what the screen reads.
- **Counting:** `count()` in SQL, never `.length` of fetched rows you do not otherwise need.
- **Indexes:** Postgres does not index foreign key columns on its own. Every column a query filters or joins on gets an `index(...)` in the table definition; give each foreign key one when the table is written (DRIZZLE.md § Schema Definition).

---

## Checklist

Before calling a route or service done:

1. No query inside a loop, a `.map()`, or a helper called per row — reads and writes alike.
2. Reads that do not depend on each other share one `Promise.all`.
3. Each table is written once per kind of change; per-row values go through `caseBy`.
4. No select-back after a write, no existence read before one, nothing read twice.
5. A query budget test pins the count and shows it does not grow (TESTING.md § Query Budget Tests).

---

## Deep Dive

**Official Documentation:**

- [Update many rows with different values](https://orm.drizzle.team/docs/guides/update-many-with-different-value)
- [Upsert many rows (`excluded`)](https://orm.drizzle.team/docs/guides/upsert)
- [Insert into ... select](https://orm.drizzle.team/docs/insert#insert-into--select)
- [Joins and aliases](https://orm.drizzle.team/docs/joins)
- [postgres.js — pipelining](https://github.com/porsager/postgres#transactions)

**In this project:** `PROJECT.md` lists the project's own examples — a tree read in one pass, set-based writes and copies, a whole-record save in a fixed number of statements — and where `countQueries` lives. `caseBy`, `dbNow`, `isUniqueViolation` and `withPreparedStatements` come from `@jswob/drizzle-utils`.

**Related Files:**

- SKILL.md — routing hub with decision tree, pattern index, common pitfalls
- DRIZZLE.md — the query builder itself: select, insert, update, delete, transactions
- TESTING.md — query budget tests with `countQueries`
- ELYSIA.md — service layer pattern the examples sit in
