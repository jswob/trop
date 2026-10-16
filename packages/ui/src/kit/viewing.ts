import { createContext, useContext } from 'react';

/**
 * What a shell tells the ledger in its column, and the ink a coloured ground brings with it.
 *
 * They live here, apart from any ledger, so the shells (`RailShell`, `SplitShell`) and `PagedList`
 * stand without one.
 */

/**
 * **The ink ramp, re-set for a coloured ground.** Put it on whatever element carries the ground.
 *
 * `ui-ink-3` is `#7c8697` and measures **3.70:1** on `#1e2f44` against a 5.0 floor; `ui-ink-2` is
 * **5.34** there (APCA Lc −47.4, against the plain ground's own −51.4 for the same rung). So every
 * quiet line standing on a connected ground owes a rung — and the ground's own spans are not the
 * only quiet lines on it.
 *
 * It is a **custom-property override rather than a rung threaded through the component**, and that
 * is the whole point: Tailwind compiles `text-ui-ink-3` to `color: var(--color-ui-ink-3)`, so
 * re-setting the variable on the ground moves every descendant that reads it, including ink the
 * component does not own — `kit/Select`'s placeholder, `kit/Disclose`'s chevron, a status glyph.
 * Threading a rung reaches the component's own type and leaves those behind, still at 3.70:1 while
 * the names beside them are lifted to 5.34 — one rung with two values, 1.64 apart, on one ground.
 *
 * Nothing else in the ramp moves. `ui-ink` is 11.17 on the band and `ui-ink-2` is already the value
 * being moved to.
 */
export const CONNECTED_LIFT = '[--color-ui-ink-3:var(--color-ui-ink-2)]';

/**
 * **How much of the ledger is read** while the chrome holds something: all of it (`true`), none
 * (`false`), or only **the record the page has open** — whatever the connection beds, at its own
 * scale — with the rest of the ledger still worked (`'record'`). The page's own record is the one a
 * press on the ledger could change under the work, and nothing else on it is.
 */
export type Viewing = boolean | 'record';

/**
 * **The shell's word for it: everything in this column is being read, not worked.**
 *
 * It is a context rather than a prop the caller passes because the caller is not who knows. What
 * puts the ledger into view mode is the *frame* holding something — `kit/SplitShell`'s `held` — and
 * the ledger that goes in that column is built by the place, which has no idea what the chrome
 * around it is doing. A place that had to pass it could disagree with the frame, and the first state
 * where it disagreed would be a ledger quietly offering a press beside a page that is holding a
 * draft.
 *
 * Unprovided it is `false`, so every ledger outside a shell draws exactly what it drew before, and a
 * `viewing` prop on the component itself still wins for a specimen that wants to show the state on
 * its own.
 */
const ViewingContext = createContext<Viewing>(false);

export const ViewingProvider = ViewingContext.Provider;

/** What the shell around this column says of it. `false` outside a shell. */
export function useViewing(): Viewing {
	return useContext(ViewingContext);
}
