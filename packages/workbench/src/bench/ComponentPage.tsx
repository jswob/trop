import { useEffect, useState } from 'react';

import { frameUrl } from '../lib/frame-url';
import { DEFAULT_VIEWPORT, VIEWPORTS } from '../lib/viewports';

import { Frame } from './Frame';
import { Ops, WIDTH_LABEL } from './Ops';
import { Page } from './Page';
import { PropLine } from './PropLine';
import { PartsTable, Read } from './Read';
import { useChrome } from './chrome';
import { keyAction } from './keys';
import type { PartReading } from './measure';
import { hide, only, parseBenchSearch, shownOf, withOption, type BenchSearch } from './search';
import { formatTurned, parseTurned, turn, valueNames, type Specimen } from './specimen';
import { triedFor } from './tokens';

/**
 * A component's page: its states across the stage at true size, each under its
 * line of edge print, and under them what the specimen says in words.
 *
 * Every frame is the component live, in a viewport of the width the bar names and cut to the
 * component, so nothing is scaled. One state alone gets a line of switches for its props. `Parts`
 * outlines what the specimen names and prints what the page measures for it.
 */
export function ComponentPage({
	spec,
	name,
	callers,
	search: address,
	onChange,
}: {
	spec: Specimen;
	name: string;
	/** How many files render it, counted from the source. */
	callers: number;
	search: BenchSearch;
	onChange: (next: BenchSearch) => void;
}) {
	const { tried } = useChrome();
	const search = parseBenchSearch(address);
	const ids = spec.states.map(state => state.id);
	const shown = shownOf(ids, search);
	const alone = shown.length === 1 ? shown[0] : undefined;
	const viewport = search.width ?? DEFAULT_VIEWPORT;
	const theme = search.theme ?? 'dark';
	const outlined = search.parts === 'on' && spec.parts.length > 0;
	const turned = alone === undefined ? {} : parseTurned(search.args);

	const [readings, setReadings] = useState<Record<string, PartReading[]>>({});

	useEffect(() => {
		const onKey = (event: KeyboardEvent) => {
			if (event.metaKey || event.ctrlKey || event.altKey) return;

			const target = event.target;
			if (
				target instanceof HTMLElement &&
				(target.isContentEditable || ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName))
			) {
				return;
			}

			const next = keyAction(event.key, ids, search, {
				modes: false,
				parts: spec.parts.length > 0,
			});
			if (next === undefined) return;

			event.preventDefault();
			onChange(next);
		};

		window.addEventListener('keydown', onKey);
		return () => window.removeEventListener('keydown', onKey);
	});

	const srcOf = (id: string): string => {
		const src = frameUrl(
			'kit',
			spec.subjectId,
			id,
			'bare',
			{ t: search.t, rate: search.rate },
			theme
		);
		return id === alone && search.args !== undefined
			? `${src}&args=${encodeURIComponent(search.args)}`
			: src;
	};

	const measured = alone ?? shown[0];
	const counted =
		shown.length === ids.length
			? `${String(ids.length)} ${ids.length === 1 ? 'state' : 'states'}`
			: `${String(shown.length)} of ${String(ids.length)} states`;

	return (
		<Page
			name={name}
			description={spec.is}
			readout={`${counted} at true size, ${WIDTH_LABEL[viewport].toLowerCase()} width, ${String(VIEWPORTS[viewport].width)} px`}
			ops={
				<Ops
					ids={ids}
					search={search}
					onChange={onChange}
					modes={false}
					parts={spec.parts.length > 0}
				/>
			}
			below={
				alone !== undefined && (
					<PropLine
						spec={spec}
						drawn={spec.states.find(state => state.id === alone)?.drawn === true}
						declared={valueNames(spec, alone)}
						turned={turned}
						onTurn={(prop, value) =>
							onChange(
								withOption(search, {
									args: formatTurned(turn(turned, valueNames(spec, alone), prop, value)),
								})
							)
						}
						onReset={() => onChange(withOption(search, { args: undefined }))}
					/>
				)
			}
		>
			<div className="wb-stage">
				<div className="wb-flow">
					{spec.states
						.filter(state => shown.includes(state.id))
						.map(state => (
							<figure
								key={state.id}
								className={
									shown.length > 1 && search.at === state.id
										? 'wb-cell wb-cell-standing'
										: 'wb-cell'
								}
								aria-current={shown.length > 1 && search.at === state.id ? 'true' : undefined}
							>
								<figcaption className="wb-cap">
									<span className="wb-ix">{ids.indexOf(state.id) + 1}</span>
									<span className="wb-capname">{state.label}</span>
									<span className="wb-note">{state.note}</span>
									<span className="wb-acts">
										{shown.length > 1 && (
											<>
												<button type="button" onClick={() => onChange(only(search, state.id))}>
													Only
												</button>
												<button type="button" onClick={() => onChange(hide(ids, search, state.id))}>
													Hide
												</button>
											</>
										)}
										<a href={srcOf(state.id)} target="_blank" rel="noreferrer">
											Open
										</a>
									</span>
								</figcaption>
								<Frame
									src={srcOf(state.id)}
									title={`${name}, ${state.label}`}
									viewport={viewport}
									tried={triedFor(tried, theme)}
									parts={outlined ? spec.parts : undefined}
									onParts={read => setReadings(prev => ({ ...prev, [state.id]: read }))}
								/>
							</figure>
						))}
				</div>

				{outlined && measured !== undefined && (
					<PartsTable
						spec={spec}
						state={spec.states.find(state => state.id === measured)?.label ?? measured}
						readings={readings[measured] ?? []}
					/>
				)}

				<Read spec={spec} callers={callers} />
			</div>
		</Page>
	);
}
