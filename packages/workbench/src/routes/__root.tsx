import { createRootRoute, Outlet } from '@tanstack/react-router';

import { setup } from '../lib/bound';

/**
 * Only the app's providers live here, not the workbench chrome: `frame` routes render
 * a subject alone in an iframe and must not inherit a sidebar, while still
 * needing whatever the app's components read from above them.
 *
 * The chrome is `bench/Shell`, which the `_bench` pathless layout mounts.
 */
export const Route = createRootRoute({
	component: WorkbenchRoot,
});

function WorkbenchRoot() {
	return (
		<setup.Wrap>
			<Outlet />
		</setup.Wrap>
	);
}
