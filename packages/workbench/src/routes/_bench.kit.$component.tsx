import { createFileRoute } from '@tanstack/react-router';

import { ComponentPage } from '../bench/ComponentPage';
import { Page } from '../bench/Page';
import { Read } from '../bench/Read';
import { Stage } from '../bench/Stage';
import { kitName } from '../bench/nav';
import { parseBenchSearch, type BenchSearch } from '../bench/search';
import { callersFor } from '../lib/kit-callers';
import { kitEntryById, specimenById, stageEntryById } from '../lib/registry';
import { useTypedRoute } from '../lib/typed-route';

export const Route = createFileRoute('/_bench/kit/$component')({
	component: BenchComponentPage,
	validateSearch: parseBenchSearch,
});

/**
 * One kit component's page, drawn from its specimen.
 *
 * A component drawn at its own size gets the sheet. One that needs a whole screen is staged as a
 * place is, with what the specimen holds and refuses under its frames. Every component of the kit has one
 * (the app's `specimens/index.test`).
 */
function BenchComponentPage() {
	const { params, search, navigate } = useTypedRoute<{ component: string }, BenchSearch>(Route);
	const entry = kitEntryById(params.component);
	const spec = specimenById(params.component);
	const onChange = (next: BenchSearch) => navigate({ search: next, replace: true });

	if (entry === undefined || spec === undefined) {
		return (
			<Page
				name="No such component"
				description={`The kit holds nothing called ${params.component}.`}
			>
				<div className="wb-stage">
					<p className="wb-missing">It was renamed or removed. Components lists what there is.</p>
				</div>
			</Page>
		);
	}

	const name = kitName(entry.name);
	const callers = callersFor(entry).length;

	const staged = spec.stands === 'screen' ? stageEntryById(spec.subjectId) : undefined;

	if (staged !== undefined) {
		return (
			<Stage
				kind="kit"
				subject={staged}
				name={name}
				footer={<Read spec={spec} callers={callers} />}
				search={search}
				onChange={onChange}
			/>
		);
	}

	return (
		<ComponentPage spec={spec} name={name} callers={callers} search={search} onChange={onChange} />
	);
}
