import { createFileRoute } from '@tanstack/react-router';

import { Stage } from '../bench/Stage';
import { parseBenchSearch, type BenchSearch } from '../bench/search';
import { setup } from '../lib/bound';
import { useTypedRoute } from '../lib/typed-route';

export const Route = createFileRoute('/_bench/kit/stress')({
	component: BenchStressPage,
	validateSearch: parseBenchSearch,
});

/** Every primitive on one sheet, on the bench's stage. */
function BenchStressPage() {
	const { search, navigate } = useTypedRoute<Record<string, never>, BenchSearch>(Route);

	return (
		<Stage
			kind="kit"
			subject={setup.kit.stress}
			search={search}
			onChange={next => navigate({ search: next, replace: true })}
		/>
	);
}
