import { createRouter } from '@tanstack/react-router';

import { routeTree } from './routeTree.gen';

/**
 * The workbench's own router instance, built from `design/routeTree.gen.ts`.
 *
 * It is deliberately not registered via the `Register` interface: the shared
 * components render real `<Link>`s to app paths like `/timeline`, which are not
 * workbench routes. Leaving the router unregistered keeps `to` loosely typed so
 * those components render here without type errors or navigation warnings.
 */
export const createDesignRouter = () =>
	createRouter({
		routeTree,
		defaultPreload: false,
	});
