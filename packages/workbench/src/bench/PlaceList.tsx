import { Link } from '@tanstack/react-router';

import { setup } from '../lib/bound';
import { undrawnSections } from '../lib/registry';

import { Page } from './Page';
import { asSearch } from './address';
import { useChrome } from './chrome';

/**
 * Every place the app has drawn, as a page that is read rather than staged.
 *
 * In the rail's order, so it reads as a map of the app: what each place is, and how many states it
 * lists. A destination with no place under it has not been drawn, and one line says which. Every
 * row is the way to its place, and takes the bar's options with it.
 */
export function PlaceList() {
	const { carried } = useChrome();
	const undrawn = undrawnSections().map(section => section.label);

	return (
		<Page
			name="Places"
			description="Every place the app has drawn, in the rail’s order, and how many states each lists."
			readout={`${String(setup.places.length)} places`}
		>
			<div className="wb-stage">
				<div className="wb-list">
					{setup.places.map(view => (
						<Link
							key={view.id}
							to={`/views/${view.id}`}
							search={asSearch(carried)}
							className="wb-list-row"
						>
							<b>{view.title}</b>
							<span>{view.description}</span>
							<span>
								{view.variants.length} {view.variants.length === 1 ? 'state' : 'states'}
							</span>
						</Link>
					))}
				</div>
				{undrawn.length > 0 && (
					<p className="wb-missing">
						Not drawn yet: {undrawn.join(', ')}. Each is a cell in the rail with no page behind it.
					</p>
				)}
			</div>
		</Page>
	);
}
