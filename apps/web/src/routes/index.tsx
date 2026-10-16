import { createFileRoute } from '@tanstack/react-router';

/**
 * **The ground** — the home page with nothing open over it. The frame in `__root.tsx` draws it
 * whole; there is no page here, so the rail marks no cell and the logo carries the mark.
 */
export const Route = createFileRoute('/')({
	component: () => null,
});
