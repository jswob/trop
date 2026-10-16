import { useEffect, useRef, useState } from 'react';

import { positionIn } from '../lib/frame-position';
import { VIEWPORTS, type ViewportName } from '../lib/viewports';

import { forwardKeys } from './frame-keys';
import { measureParts, type PartReading } from './measure';

type Box = { x: number; y: number; width: number; height: number };

const holds = (outer: PartReading, inner: PartReading): boolean =>
	outer !== inner &&
	inner.x >= outer.x &&
	inner.y >= outer.y &&
	inner.x + inner.width <= outer.x + outer.width &&
	inner.y + inner.height <= outer.y + outer.height;

/**
 * Where a part's letter sits, so that no letter lands on another part: over the corner of one
 * that holds others, under one that is held, and beside one that stands on its own.
 */
function tagPlace(part: PartReading, all: readonly PartReading[]): string {
	if (all.some(other => holds(part, other))) return 'wb-part-over';
	if (all.some(other => holds(other, part))) return 'wb-part-under';
	return 'wb-part-beside';
}

/** How often a frame is asked what it holds. A frame's document tells nobody when it changes. */
const ASK_EVERY = 120;

/**
 * One frame on the bench: the app, live, in a real viewport of its own.
 *
 * It is drawn one of two ways. **Fitted**, it is given a width and scaled to it, as a place is.
 * **Hugged**, it is drawn at its true size and cut to the specimen inside it, so a button is
 * the size a button is, while the viewport stays the one the bar names and the component still
 * stands at that breakpoint.
 *
 * The frame is another document, and it gets a new one whenever its address changes. So the bench
 * asks it, a few times a second, what it holds: a new document is listened to for keys, the values
 * being tried are written onto its root, and a hugged frame is measured, as are its named parts.
 *
 * A frame that walks keeps where it stands in its own address (`lib/frame-position`), which is not
 * the `src` it was given. The bench reads that too, and says so whenever it changes.
 */
export function Frame({
	src,
	title,
	viewport,
	width,
	tried,
	parts,
	onParts,
	onPosition,
}: {
	src: string;
	title: string;
	viewport: ViewportName;
	/** The width it is fitted to. Absent, it is hugged at its true size. */
	width?: number;
	/** Token values being tried, by custom property, for the appearance this frame is drawn in. */
	tried?: Readonly<Record<string, string>>;
	/** The parts to outline and measure. Absent or empty, none are. */
	parts?: readonly { name: string; select: string }[];
	onParts?: (readings: PartReading[]) => void;
	/** Told where the frame stands, first as it is found and then whenever it moves. */
	onPosition?: (where: string | null) => void;
}) {
	const size = VIEWPORTS[viewport];
	const frameRef = useRef<HTMLIFrameElement>(null);
	const [box, setBox] = useState<Box | null>(null);
	const [readings, setReadings] = useState<PartReading[]>([]);

	const hug = width === undefined;
	const asked = useRef({ tried, parts, onParts, onPosition, hug });
	useEffect(() => {
		asked.current = { tried, parts, onParts, onPosition, hug };
	});

	useEffect(() => {
		let heard: Document | null = null;
		let applied = new Set<string>();
		let read = '[]';
		let stood: string | null | undefined;

		const ask = () => {
			const frame = frameRef.current;
			const inner = frame?.contentWindow ?? null;
			const held = frame?.contentDocument ?? null;
			// A document on its way in has no root yet; it is asked again in a moment.
			if (inner === null || held === null || held.documentElement === null) return;

			if (held !== heard) {
				heard = held;
				applied = new Set();
				forwardKeys(inner, window);
			}

			const stands = positionIn(inner.location.search);
			if (stands !== stood) {
				stood = stands;
				asked.current.onPosition?.(stands);
			}

			const want = asked.current.tried ?? {};
			const style = held.documentElement.style;
			for (const name of applied) {
				if (!(name in want)) {
					style.removeProperty(name);
					applied.delete(name);
				}
			}
			for (const [name, value] of Object.entries(want)) {
				if (style.getPropertyValue(name) !== value) style.setProperty(name, value);
				applied.add(name);
			}

			const root = held.querySelector('[data-specimen]');

			if (asked.current.hug && root !== null) {
				const rect = root.getBoundingClientRect();
				const next = {
					x: Math.round(rect.left),
					y: Math.round(rect.top),
					width: Math.ceil(rect.width),
					height: Math.ceil(rect.height),
				};
				setBox(prev =>
					prev !== null &&
					prev.x === next.x &&
					prev.y === next.y &&
					prev.width === next.width &&
					prev.height === next.height
						? prev
						: next
				);
			}

			const wanted = asked.current.parts ?? [];
			const now = root !== null && wanted.length > 0 ? measureParts(root, wanted) : [];
			const written = JSON.stringify(now);
			if (written !== read) {
				read = written;
				setReadings(now);
				asked.current.onParts?.(now);
			}
		};

		ask();
		const watch = window.setInterval(ask, ASK_EVERY);
		return () => window.clearInterval(watch);
	}, []);

	if (!hug) {
		const scale = width / size.width;

		return (
			<div className="wb-frame" style={{ width, height: Math.round(size.height * scale) }}>
				<iframe
					ref={frameRef}
					title={title}
					src={src}
					width={size.width}
					height={size.height}
					style={{ transform: `scale(${String(scale)})` }}
				/>
			</div>
		);
	}

	return (
		<div
			className="wb-frame wb-frame-hug"
			style={{ width: box?.width ?? 240, height: box?.height ?? 80 }}
		>
			<iframe
				ref={frameRef}
				title={title}
				src={src}
				width={size.width}
				height={size.height}
				style={{ marginLeft: -(box?.x ?? 0), marginTop: -(box?.y ?? 0) }}
			/>
			{readings.map(part => (
				<i
					key={part.name}
					className={`wb-part-box ${tagPlace(part, readings)}`}
					style={{
						left: part.x - 2,
						top: part.y - 2,
						width: part.width + 4,
						height: part.height + 4,
					}}
				>
					<b>{part.tag}</b>
				</i>
			))}
		</div>
	);
}
