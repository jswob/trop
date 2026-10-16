import { useEffect, useState, type ReactNode } from 'react';

import { useChrome } from './chrome';

const FOLD = (
	<svg
		width="15"
		height="15"
		viewBox="0 0 15 15"
		fill="none"
		stroke="currentColor"
		strokeWidth="1.3"
	>
		<rect x="1.5" y="2.5" width="12" height="10" />
		<path d="M5.5 2.5v10" />
	</svg>
);

/**
 * One page of the bench: the bar, a line of words, and what the page draws under them.
 *
 * The words have a line of their own because the six switches leave no room beside them. While a
 * token value is being tried the line says so, on every page, with the way back. Under
 * 1024px the bar is a sheet behind `Options` and the sidebar a drawer behind `Places`, with the
 * place's name between the two.
 */
export function Page({
	name,
	description,
	readout,
	ops,
	below,
	notes,
	children,
}: {
	name: string;
	description: string;
	/** What the stage is showing, in a few words, at the end of the line. */
	readout?: string;
	/** The switches. A page that is read rather than staged has none. */
	ops?: ReactNode;
	/** A second line under the bar: the props of the one state showing. */
	below?: ReactNode;
	/** What is open under the stage, taking its height from it: a candidate's notes. */
	notes?: ReactNode;
	children: ReactNode;
}) {
	const { folded, toggleFolded, setDrawer, tried, setTried } = useChrome();
	const trying = Object.keys(tried).length;
	const [sheet, setSheet] = useState(false);

	useEffect(() => {
		if (!sheet) return;

		const onKey = (event: KeyboardEvent) => {
			if (event.key === 'Escape') setSheet(false);
		};

		/** The sheet is the bar under a finger; at a width with a bar there is nothing for it to be. */
		const wide =
			typeof window.matchMedia === 'function' ? window.matchMedia('(min-width: 1024px)') : null;
		const shut = () => {
			if (wide?.matches === true) setSheet(false);
		};

		window.addEventListener('keydown', onKey);
		wide?.addEventListener('change', shut);
		return () => {
			window.removeEventListener('keydown', onKey);
			wide?.removeEventListener('change', shut);
		};
	}, [sheet]);

	return (
		<>
			<div className="wb-touch">
				<button type="button" onClick={() => setDrawer(true)}>
					Places
				</button>
				<h1>{name}</h1>
				{ops !== undefined && (
					<button type="button" aria-expanded={sheet} onClick={() => setSheet(!sheet)}>
						Options
					</button>
				)}
			</div>

			<div className="wb-bar" data-open={sheet ? '' : undefined}>
				<button
					type="button"
					className="wb-fold"
					aria-label={folded ? 'Open the sidebar' : 'Fold the sidebar'}
					aria-pressed={!folded}
					title="Sidebar, \"
					onClick={toggleFolded}
				>
					{FOLD}
				</button>
				{ops}
			</div>
			{sheet && (
				<button
					type="button"
					className="wb-veil wb-veil-touch"
					aria-label="Close the options"
					onClick={() => setSheet(false)}
				/>
			)}

			<div className="wb-body" inert={sheet}>
				{below}
				<div className="wb-words">
					<h1 className="wb-name">{name}</h1>
					<p className="wb-desc">{description}</p>
					{trying > 0 ? (
						<span className="wb-readout wb-try">
							Trying {trying} {trying === 1 ? 'value' : 'values'}
							<button type="button" className="wb-reset" onClick={() => setTried({})}>
								Reset
							</button>
						</span>
					) : (
						readout !== undefined && <span className="wb-readout">{readout}</span>
					)}
				</div>

				{children}
				{notes !== undefined && (
					<aside className="wb-notes" aria-label="Notes">
						{notes}
					</aside>
				)}
			</div>
		</>
	);
}
