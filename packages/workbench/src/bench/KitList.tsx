import { Link } from '@tanstack/react-router';

import { setup } from '../lib/bound';
import { callersFor } from '../lib/kit-callers';

import { Page } from './Page';
import { asSearch } from './address';
import { useChrome } from './chrome';
import { kitName } from './nav';

/**
 * The kit's catalogue, as a page that is read rather than staged.
 *
 * It has the bar's fold and the line of words and no switches, because nothing on it is a frame.
 * The caller count is read off the source, as it is everywhere a component's promotion turns on it.
 * Every row is the way to its component's page, and takes the bar's options with it.
 */
export function KitList() {
	const { carried } = useChrome();

	return (
		<Page
			name="Components"
			description="Every component the kit holds, and how many files call it."
			readout={`${String(setup.kit.entries.length)} components`}
		>
			<div className="wb-stage">
				<div className="wb-list">
					{setup.kit.entries.map(entry => {
						const callers = callersFor(entry).length;
						return (
							<Link
								key={entry.id}
								to={`/kit/${entry.id}`}
								search={asSearch(carried)}
								className="wb-list-row"
							>
								<b>{kitName(entry.name)}</b>
								<span>{entry.line}</span>
								<span>
									{callers} {callers === 1 ? 'caller' : 'callers'}
								</span>
							</Link>
						);
					})}
				</div>
			</div>
		</Page>
	);
}
