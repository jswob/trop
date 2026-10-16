import { createFileRoute, redirect } from '@tanstack/react-router';

import { asSearch } from '../bench/address';
import { carry, parseBenchSearch } from '../bench/search';
import { setup } from '../lib/bound';
import { groundView } from '../lib/registry';

/**
 * The workbench opens on the ground, which is where the app opens.
 *
 * Sent there before anything draws. A component that navigated instead re-rendered with the shell
 * on every pass and logged "Maximum update depth exceeded" a hundred times on the way.
 */
export const Route = createFileRoute('/_bench/')({
	beforeLoad: ({ location }) => {
		const first = groundView() ?? setup.places[0];

		// eslint-disable-next-line @typescript-eslint/only-throw-error -- Reason: TanStack Router redirects by throwing the object `redirect()` returns
		throw redirect({
			to: first ? `/views/${first.id}` : '/kit',
			search: asSearch(carry(parseBenchSearch(location.search))),
			replace: true,
		});
	},
});
