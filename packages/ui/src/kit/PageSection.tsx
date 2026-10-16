import { createContext, useContext } from 'react';
import type { ReactNode } from 'react';

/**
 * **A section of a page, and what the column rule is allowed to cut.**
 *
 * A page set as a spread (`kit/PageSpread`) is one flow in however many columns of the measure the
 * glass takes, and *where that flow breaks* is the only thing about it a caller needs to say. A
 * policy applied to content **by tag name, from outside** — `[&_section]:break-inside-avoid` on the
 * page — makes every `<section>` atomic, so the tallest section moves whole or not at all and the
 * column it leaves can stand half empty, with no way for one section to answer back.
 *
 * So the separator is the page's own subcomponent. It carries three things a page would otherwise
 * write at every call site — the band, the rhythm below it, and a **declared break policy**:
 *
 * - **`whole`** (the default) — the section is one block. It moves to the next column entire, which
 *   is what a section with nothing to divide wants.
 * - **`units`** — the section may cross a rule, and it names the pieces the rule may not fall
 *   inside. The band is carried down to whatever follows it, so a column that opens a section
 *   opens on the section's **name**, and every `PageUnit` inside it stays in one leaf.
 *
 * **There is no third setting.** The band kept with nothing below it protected lets the rule land
 * wherever the packing puts it: one item cut between its own rows, the next column opening on rows
 * with nothing naming whose they are, a month grid halved across a week. `units` with nothing marked
 * is that behaviour by another route, which is why a section that declares it has to name its
 * pieces.
 *
 * **What CSS will not do**, named so it is not re-attempted: there is no running header in multicol.
 * A section continued into the next column cannot repeat its own band as *Projects, cont.*
 * That means measuring where the break landed and drawing a second band from JS — the same class of
 * work as `useSpread` — and it is not built.
 */

/** What the column rule is allowed to cut in one section. */
export type SectionBreak = 'whole' | 'units';

/**
 * The policy, handed down rather than drilled.
 *
 * A unit is always inside the section that owns it, so the answer is that section's own subtree and
 * not app state. Drilling it through every caller's markup is how the second caller gets it subtly
 * wrong, and a page can put several components between a section and its units.
 */
const Policy = createContext<SectionBreak>('whole');

/**
 * The rhythm, and the two policies.
 *
 * `pb-7` in both: 28px spent **below** a section and never above it, so two columns start on the
 * same line. It is written here rather than by the page, so `PageColumn` below `lg` and the spread
 * from it put the rhythm in the same place: below a section, at every width.
 */
const BREAKS: Record<SectionBreak, string> = {
	whole: 'break-inside-avoid pb-7',
	units: 'pb-7',
};

/**
 * A section boundary: a word, what the section comes to, and one hairline.
 *
 * A hairline marks a section and nothing else. The aside is a fact the section owns, the way a list
 * carries its own count, and never an action flung to the far edge from its subject. It is a string
 * so every caller's aside is the same `text-ui-body text-ui-ink-3` span.
 *
 * **`break-after-avoid` is what makes `units` safe.** Without it the band is an ordinary block and
 * lands at the foot of a column perfectly happily, leaving its own heading on one leaf and
 * everything it names on the next. It is inert under `whole`, where nothing breaks anyway.
 */
function Band({ title, aside }: { title: string; aside: string | undefined }) {
	return (
		<div className="flex items-end gap-4 border-b border-ui-rule pb-1.5 break-after-avoid">
			<h2 className="min-w-0 truncate text-ui-lead font-medium text-ui-ink">{title}</h2>
			{aside === undefined ? null : (
				<span className="ml-auto shrink-0 text-ui-body text-ui-ink-3">{aside}</span>
			)}
		</div>
	);
}

/** A section of a page: its band, its rhythm, and what happens to it at a column rule. */
export function PageSection({
	title,
	aside,
	breaks = 'whole',
	children,
}: {
	title: string;
	/** What the section comes to — a count, a span, a tally. Never an action. */
	aside?: string;
	/** Defaults to `whole`: one block, moved entire. */
	breaks?: SectionBreak;
	children: ReactNode;
}) {
	return (
		<Policy value={breaks}>
			<section className={BREAKS[breaks]}>
				<Band title={title} aside={aside} />
				{children}
			</section>
		</Policy>
	);
}

/**
 * A piece of a section the column rule may not fall inside — under `units`, and only there.
 *
 * Under `whole` the section is already one block and a unit inside it can never be reached, so it
 * draws nothing. It is the **same element** either way, so moving a section between the two
 * policies never moves anything in the box model: what changes is where the break lands and nothing
 * else.
 *
 * It lives here rather than beside `PageSection` in the kit's index because it is not an
 * independent component — outside a `PageSection` it has no policy to read and is a bare `div`. A
 * caller reaching for it alone has the wrong thing.
 */
export function PageUnit({ children }: { children: ReactNode }) {
	const breaks = useContext(Policy);

	return <div className={breaks === 'units' ? 'break-inside-avoid' : undefined}>{children}</div>;
}
