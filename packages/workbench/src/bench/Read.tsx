import type { PartReading } from './measure';
import type { Specimen } from './specimen';

/**
 * A component's named parts with what the page reports for each.
 *
 * The figures are the ones `Frame` read off the live component a moment ago: change a prop, the
 * appearance or the width and they follow. A part the state does not draw has no row.
 */
export function PartsTable({
	spec,
	state,
	readings,
}: {
	spec: Specimen;
	/** The state the figures are of, by its label. */
	state: string;
	readings: readonly PartReading[];
}) {
	const isOf = (name: string): string => spec.parts.find(part => part.name === name)?.is ?? '';

	return (
		<section className="wb-parts" aria-label="Parts">
			<h2 className="wb-h">Parts of {state}, as the page reports them</h2>
			<div className="wb-part wb-part-head" aria-hidden="true">
				<span />
				<span>Part</span>
				<span>What it is</span>
				<span>Box</span>
				<span>Padding</span>
				<span>Type</span>
				<span>Ink on its ground</span>
			</div>
			{readings.length === 0 && <p className="wb-none">Reading the frame.</p>}
			{readings.map(part => (
				<div key={part.name} className="wb-part">
					<i>{part.tag}</i>
					<b>{part.name}</b>
					<span>{isOf(part.name)}</span>
					<span>
						{part.width} × {part.height}
					</span>
					<span>
						{part.padding !== null
							? part.padding.join(' ')
							: part.edge !== null
								? `edge ${part.edge}`
								: 'none'}
					</span>
					<span>{part.type ?? 'a glyph'}</span>
					<span>
						<i className="wb-dot" style={{ background: part.ink }} />
						{part.ink} on <i className="wb-dot" style={{ background: part.ground }} />
						{part.ground}, {part.contrast}
					</span>
				</div>
			))}
		</section>
	);
}

/** What a component's page says under its frames: a line a prop, what it holds, what it refuses. */
export function Read({ spec, callers }: { spec: Specimen; callers: number }) {
	const values = (names: readonly string[], kind: string): string =>
		kind === 'text' ? 'text' : names.join(', ');

	return (
		<div className="wb-read">
			<section aria-label="Props">
				<h2 className="wb-h">Props</h2>
				{spec.props.map(prop => (
					<div key={prop.name} className="wb-prow">
						<b>{prop.name}</b>
						<span>{values(prop.names, prop.kind)}</span>
						<span>{prop.is}</span>
					</div>
				))}
			</section>
			<section aria-label="It holds">
				<h2 className="wb-h">It holds</h2>
				{spec.holds.map(line => (
					<p key={line} className="wb-line">
						{line}
					</p>
				))}
			</section>
			<section aria-label="It refuses">
				<h2 className="wb-h">It refuses</h2>
				{spec.refuses.map(line => (
					<p key={line} className="wb-line">
						{line}
					</p>
				))}
				<p className="wb-callers">
					Called by {callers} {callers === 1 ? 'file' : 'files'}, counted from the source.
				</p>
			</section>
		</div>
	);
}
