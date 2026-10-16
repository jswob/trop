import type { Subject } from '@jswob/design-workbench';

/**
 * Every prototype: what is **unsettled**, and nothing else.
 *
 * A prototype is disposable. Once it settles it leaves, either into `/views` as a place or into
 * `/kit` as a component, and the candidates that lost are deleted with the harness they were drawn
 * in. What they taught goes into `apps/workbench/BANS.md`, which is the durable record; the
 * candidates themselves never were.
 *
 * **The space is empty.** Nothing is unsettled. `bun workbench new` adds an entry here.
 *
 * The moment a question is answered the answer stops being something to pick off a variant bar and
 * becomes how the thing behaves. **Do not recreate a dead candidate to "compare".**
 */
export const PROTOS: Subject[] = [];
