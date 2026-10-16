type UnregisteredRoute = {
	useParams: () => unknown;
	useSearch: () => unknown;
	useNavigate: () => unknown;
};

type TypedRoute<TParams, TSearch> = {
	params: TParams;
	search: TSearch;
	navigate: (options: { search?: TSearch; replace?: boolean }) => void;
};

/**
 * Params, search and navigation for a workbench route, typed.
 *
 * The workbench router is deliberately not registered via TanStack's `Register`
 * interface — see `design/router.ts` — so its hooks come back as `any`. This puts
 * the types back at the single boundary where they are lost, rather than
 * scattering casts (and eslint disables) through every route.
 */
export function useTypedRoute<TParams, TSearch>(
	route: UnregisteredRoute
): TypedRoute<TParams, TSearch> {
	return {
		params: route.useParams() as TParams,
		search: route.useSearch() as TSearch,
		navigate: route.useNavigate() as TypedRoute<TParams, TSearch>['navigate'],
	};
}
