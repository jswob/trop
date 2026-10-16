import type { ReactNode } from 'react';

/**
 * A number that has to line up with the numbers above and below it. Geist Mono with
 * tabular figures — the monospace is here because the digits align in a column, not
 * as decoration on a label.
 */
export function Figure({ children }: { children: ReactNode }) {
	return <span className="font-ui-mono tabular-nums">{children}</span>;
}
