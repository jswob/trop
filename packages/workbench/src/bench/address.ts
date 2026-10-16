import { useRouterState } from '@tanstack/react-router';

type Address = { pathname: string; search: Record<string, unknown> };

/**
 * Where the bench is, and the whole of the address's search.
 *
 * The workbench's router is not registered (see `design/router.ts`), so its state comes back
 * untyped. This is the one place the bench puts the types back.
 */
export function useAddress(): Address {
	const state = useRouterState() as unknown as { location: Address };

	return { pathname: state.location.pathname, search: state.location.search };
}

/**
 * A search, as a link takes it.
 *
 * An unregistered router types a link's `search` as one no route accepts, so a plain object does
 * not fit. The object is what the router reads at run time.
 */
export function asSearch(search: Record<string, unknown>): never {
	return search as never;
}
