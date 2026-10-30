# Data layout: tables, names, collections, walk storage

How names, tables and the walk's state are stored (D2-3 to D2-7, BUILDER §3 and §7; RULEBOOK group 3). Plain std types only: no collection, arena, interner or persistent-map crate. Each would bring a banned dependency (hashbrown, rand) or a panic you can't see.

## Contents

1. Ids and tables
2. Names: the interner
3. Collections
4. Walk storage and checkpoints
5. Order: what may decide output

## 1. Ids and tables

- **One `Table<I, T>` for every id-keyed store** (D2-4), about 40 lines: `push`, `get` and `iter`, and **no `Index` impl**, so `table[id]` doesn't compile. `get(id)` returns `Result<&T, Internal>`. It is the one expected own generic; R-124's report lists it once.
- Ids are own newtypes (`ItemId`, `PlaceId`), never bare `usize` in anything hashed or printed (R-44: fixed-width types only).
- Spans are an own `Span { start, end }` of byte offsets with checked constructors.

**Bad / good**

```rust
// Bad: indexing (R-5); a crash on a bad id
let item = &items[id.0];
// Good: the miss is an internal error carried with `?`
let item = items.get(id)?;
```

## 2. Names: the interner

- **One hand-written interner** (D2-3), about 50 lines: a `Symbol(u32)` per distinct text, built over a `BTreeMap` from text to symbol.
- **Its order is the text's:** `Ord` compares `as_str()`, as rustc's `StableCompare for Symbol` does. An id's number never decides output order (R-47).
- It exists for memo keys, which hold key values (things, keys of open indices, labels; seam F): a fixed-size symbol keeps a key small and cheap to compare.
- T's property test that reorders files guards the order.

## 3. Collections (D2-5, K8-6)

| Data                                                                        | Use                                                                                   |
| --------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| maps and sets in general                                                    | `BTreeMap` / `BTreeSet` (kept in key order, like a `Map` that always iterates sorted) |
| tables frozen after load                                                    | a sorted boxed slice (`Box<[T]>`), searched by binary search                          |
| day- or position-keyed data: a recurrence's column per (def, context), logs | a dense `Vec`                                                                         |
| insertion order that **is** the meaning                                     | `IndexMap`, each use with a reason (R-37 allows hashbrown only under it)              |

- **No hash map or set, the memo's included** (R-36, R-37; K8-6). The heavy memo misses are day-keyed, which a dense column serves. A hash memo is revisited only if the first engine profile shows sparse memo lookups dominating, and then through a rules change (R-112).
- No small-vector crate.
- Sort with `sort_by_key` on a total key; never `sort_unstable` (R-43). A comparator that isn't a total order may crash.

## 4. Walk storage and checkpoints (D2-6)

- **Checkpoints**, taken monthly at first (the interval is measured at S), are `Vec<Rc<PlaceState>>`, copied on write. `Rc` is a shared pointer with a reference count, like several variables holding one JavaScript object; a place's state is cloned only when a day changes it. A checkpoint costs O(places); a day costs O(the places it changes).
- **Pins and scenarios** are small layers over a base, never full copies.
- **A place that keeps parts** (an `owned` or `batched` kind) holds its parts as a sorted label vector; every other place holds a plain balance, so the labels' cost is paid only where a kind opts in (seam, BUILD-NOTES).
- A day column sits beside each log, for `bal(x, d)` by binary search and `flows(x, span)` by prefix sums.

## 5. Order: what may decide output

- Output order never comes from an internal id, a memory address, insertion order or a derived `Ord` (R-47). Use explicit sort keys: the text order of stable ids, the spec's canonical order, or the written `ties by`. Since 2.1 this covers labels in ties, `same` classes and member prefixes.
- Every `derive(PartialOrd, Ord)` on the engine's own types is listed for review (R-118).
- Output is written from typed structs and ordered `Vec`s, never from a generic JSON value or a map keyed by strings (R-46).
- The CLI reads the reality's directory tree, collects every relative path first, then sorts the **full paths by code point** (E §3.1 l.118), never each directory's names on its own (R-40): `a/b.tally` and `a.x.tally` order differently, and the hash changes.
