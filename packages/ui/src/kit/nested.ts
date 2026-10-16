import type { LucideIcon } from 'lucide-react';

/**
 * **A page opened inside the open page.**
 *
 * A nested page is a record the open page shows in its own place: a note, opened from its title in
 * the page's list. It takes the page's place whole and shares nothing with it, entering and leaving
 * it never asks, and the way back always goes to the page it was opened from, exactly as it was
 * left. One level only.
 *
 * The frame says so in two drawings from this one declaration, and a page draws neither:
 *
 * - **from `lg`**, `kit/PageSpread` sets the whole flow one 48px step in, head included, with the
 *   page's own glyph pinned at the top of the step as the way back and one `ui-rule` thread under it
 *   to the flow's last line;
 * - **below `lg`**, `kit/RailShell` grows the slug a **level row** above its own, in the one box:
 *   the record's name in the 44px label column, and the page's glyph as its one press back. The
 *   slug's own row stays the page's — the hold's row, whose body still peeks, or the resting bar.
 */
export type NestedPage = {
	/** Which record is open, so opening another arrives under the page's own wipe. */
	id: string;
	/** What the nested page is called: the record's own name. */
	name: string;
	/** The page it was opened from, as the rail names it — `Notes`. */
	page: string;
	/** That page's own glyph. The way back wears it, at every width. */
	icon: LucideIcon;
	/** Back to the page, exactly as it was left. */
	onBack: () => void;
};
