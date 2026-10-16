import { createFileRoute } from '@tanstack/react-router';

import { Page } from '../bench/Page';
import { ProtoNotes } from '../bench/ProtoNotes';
import { Stage } from '../bench/Stage';
import { parseBenchSearch, type BenchSearch } from '../bench/search';
import { proposedShapeFor } from '../lib/proto-shape';
import { protoById } from '../lib/registry';
import { useTypedRoute } from '../lib/typed-route';

export const Route = createFileRoute('/_bench/proto/$name')({
	component: BenchProtoPage,
	validateSearch: parseBenchSearch,
});

/** One candidate on the bench's stage, with its notes behind the bar's switch. */
function BenchProtoPage() {
	const { params, search, navigate } = useTypedRoute<{ name: string }, BenchSearch>(Route);
	const proto = protoById(params.name);

	if (!proto) {
		return (
			<Page name="No such candidate" description={`Nothing unsettled is called ${params.name}.`}>
				<div className="wb-stage">
					<p className="wb-missing">It settled or was deleted. The sidebar lists what is left.</p>
				</div>
			</Page>
		);
	}

	return (
		<Stage
			kind="proto"
			subject={proto}
			notes={<ProtoNotes bet={proto.description} shape={proposedShapeFor(proto.id)} />}
			search={search}
			onChange={next => navigate({ search: next, replace: true })}
		/>
	);
}
