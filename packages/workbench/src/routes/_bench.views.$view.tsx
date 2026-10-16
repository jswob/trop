import { createFileRoute } from '@tanstack/react-router';

import { Page } from '../bench/Page';
import { Stage } from '../bench/Stage';
import { parseBenchSearch, type BenchSearch } from '../bench/search';
import { viewById } from '../lib/registry';
import { useTypedRoute } from '../lib/typed-route';

export const Route = createFileRoute('/_bench/views/$view')({
	component: BenchViewPage,
	validateSearch: parseBenchSearch,
});

/** One place on the bench's stage. */
function BenchViewPage() {
	const { params, search, navigate } = useTypedRoute<{ view: string }, BenchSearch>(Route);
	const view = viewById(params.view);

	if (!view) {
		return (
			<Page name="No such place" description={`Nothing is registered as ${params.view}.`}>
				<div className="wb-stage">
					<p className="wb-missing">It was renamed or removed. The sidebar lists what there is.</p>
				</div>
			</Page>
		);
	}

	return (
		<Stage
			kind="view"
			subject={view}
			search={search}
			onChange={next => navigate({ search: next, replace: true })}
		/>
	);
}
