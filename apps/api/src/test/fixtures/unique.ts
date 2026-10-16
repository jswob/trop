/**
 * Uniqueness helpers for fixture values backed by a UNIQUE constraint.
 *
 * Two hazards make the obvious approaches unreliable:
 * - `composeFactory` mints a fresh `SequenceCounter` on every `factory.x` access,
 *   so the builder-scoped `sequence` restarts at 1 for each separate `create()`.
 * - Vitest runs test files in separate worker **processes**, so even a
 *   module-level counter restarts in each one, and two workers inserting
 *   concurrently collide.
 *
 * Faker alone is not enough either: `person.jobTitle()` and `word.noun()` draw
 * from small pools, so repeats are routine across a full suite.
 *
 * Combining the worker index with a process-monotonic counter makes a value
 * unique across the whole run.
 */

/** This vitest worker's index; `0` when running outside a worker pool. */
export const WORKER_INDEX = Number(process.env.VITEST_POOL_ID ?? 0);

let counter = 0;

/** A token unique across every worker and every call in a test run. */
export function uniqueSuffix(): string {
	counter += 1;
	return `w${WORKER_INDEX}-${counter}`;
}
