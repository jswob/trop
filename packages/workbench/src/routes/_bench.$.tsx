import { createFileRoute } from '@tanstack/react-router';

import { Page } from '../bench/Page';

export const Route = createFileRoute('/_bench/$')({
	component: BenchNoSuchPage,
});

/** An address the bench has no page at, said inside the chrome so the way on is beside it. */
function BenchNoSuchPage() {
	return (
		<Page name="No such page" description="The workbench has nothing at this address.">
			<div className="wb-stage">
				<p className="wb-missing">It was moved or removed. The sidebar lists what there is.</p>
			</div>
		</Page>
	);
}
