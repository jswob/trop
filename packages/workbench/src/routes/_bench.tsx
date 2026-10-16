import { createFileRoute } from '@tanstack/react-router';

import { BenchShell } from '../bench/Shell';

/**
 * The bench: the workbench's own chrome.
 *
 * A pathless layout, so every page but a frame is drawn in it and none of them carries its name in
 * the address. `/frame` stays outside: a frame owns its viewport.
 */
export const Route = createFileRoute('/_bench')({
	component: BenchShell,
});
