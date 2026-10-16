import { useEffect, useLayoutEffect, useRef, useState, type ReactNode } from 'react';

import { frameUrl, type FrameKind } from '../lib/frame-url';
import { stageEntryById, stageKindOf } from '../lib/registry';
import { splitFrames } from '../lib/stage-search';
import type { Subject } from '../lib/subject';
import { useElementSize } from '../lib/use-element-size';
import { DEFAULT_VIEWPORT, VIEWPORTS, VIEWPORT_NAMES, type ViewportName } from '../lib/viewports';

import { Frame } from './Frame';
import { Ops, WIDTH_LABEL, acrossOf } from './Ops';
import { Page } from './Page';
import { listCells, stateCells } from './cells';
import { useChrome } from './chrome';
import { statesFit, widthsFit } from './fit';
import { keyAction } from './keys';
import { cursorOf, hide, only, parseBenchSearch, shownOf, type BenchSearch } from './search';
import { triedFor } from './tokens';

/** The space between two frames. Part of the layout, so it never scales with them. */
const GAP = 10;

/** One line of edge print over a frame. A caption that wraps is measured, and the fit follows. */
const CAPTION = 22;

/** The line that names the state, over the three widths of it. */
const STANDING = 28;

/** Below this the three widths do not sit side by side at a size worth reading, so they stack. */
const STACK_BELOW = 700;

const sized = (name: ViewportName): string =>
	`${String(VIEWPORTS[name].width)} × ${String(VIEWPORTS[name].height)}`;

const percent = (scale: number): string => `${String(Math.round(scale * 100))}%`;

/**
 * A place on the stage, in one of two modes.
 *
 * **States** draws every state of it at one width, as large as the stage allows with all of them
 * on it, or one at a time. **Widths** draws one state at the three widths, at one scale, so a
 * breakpoint is judged in a single look. Each frame is a real viewport at its true size.
 *
 * Everything the stage shows is in the address. Hiding a state or showing one alone is on that
 * state's own caption, because it is about that state.
 *
 * An address with a `frames` list draws those frames instead, whichever subjects they are of: a
 * candidate beside the place it would replace. Leaving for another place drops the list.
 */
export function Stage({
	kind,
	subject,
	search: address,
	onChange,
	name,
	footer,
	notes,
}: {
	kind: FrameKind;
	subject: Subject;
	/** What the page is called, where that is not the subject's own title. */
	name?: string;
	/** What follows the frames in the same scroll: a component's props and rules. */
	footer?: ReactNode;
	/** What the bar's `Notes` switch opens under the stage. A candidate has them; a place does not. */
	notes?: ReactNode;
	/** The address's search as the route hands it over, which may still hold a value nobody can read. */
	search: BenchSearch;
	onChange: (next: BenchSearch) => void;
}) {
	const search = parseBenchSearch(address);
	const areaRef = useRef<HTMLDivElement>(null);
	const area = useElementSize(areaRef);

	const { tried } = useChrome();
	const refs = splitFrames(search.frames);
	const list = refs === undefined ? undefined : listCells(refs, stageEntryById, stageKindOf);
	const cells = list?.cells ?? stateCells(kind, subject);
	const title = list === undefined ? (name ?? subject.title) : 'Side by side';
	const description =
		list === undefined
			? subject.description
			: list.missing.length > 0
				? `Frames of several subjects at one width. Nothing is registered as ${list.missing.join(', ')}.`
				: 'Frames of several subjects at one width, as the address lists them.';
	const noted = notes !== undefined && list === undefined;
	const below = list === undefined ? footer : undefined;
	const open = noted && search.notes === 'on' ? notes : undefined;
	const trying = triedFor(tried, search.theme ?? 'dark');
	const ids = cells.map(cell => cell.id);
	const total = ids.length;
	const shown = shownOf(ids, search);
	const cursor = cursorOf(ids, search);
	const widths = search.show === 'widths';
	const viewport = search.width ?? DEFAULT_VIEWPORT;
	const across = acrossOf(search.scale);

	/**
	 * A caption wraps when its frame is narrow, and how far depends on its words. So its height is
	 * read off the page rather than guessed, and only ever grows for one arrangement: a fit that
	 * followed it down again would have the two chasing each other.
	 */
	const arrangement = [
		subject.id,
		search.frames,
		shown.join(),
		viewport,
		across,
		area.width,
		area.height,
	].join('|');
	const [measured, setMeasured] = useState({ arrangement, height: CAPTION });
	const captionHeight = measured.arrangement === arrangement ? measured.height : CAPTION;

	useLayoutEffect(() => {
		const captions = areaRef.current?.querySelectorAll<HTMLElement>('.wb-grid .wb-cap') ?? [];
		const tallest = Math.max(CAPTION, ...[...captions].map(caption => caption.offsetHeight));

		if (tallest > captionHeight) setMeasured({ arrangement, height: tallest });
	}, [arrangement, captionHeight]);

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

			const next = keyAction(event.key, ids, search, { notes: noted });
			if (next === undefined) return;

			event.preventDefault();
			onChange(next);
		};

		window.addEventListener('keydown', onKey);
		return () => window.removeEventListener('keydown', onKey);
	});

	/**
	 * Where each frame has walked to, by the address it was drawn from. A frame's `src` never
	 * follows it, or the frame would be loaded again on every step; `Open` does.
	 */
	const [walked, setWalked] = useState<Readonly<Record<string, string | null>>>({});
	const onWalk = (src: string, where: string | null) => {
		setWalked(was => ((was[src] ?? null) === where ? was : { ...was, [src]: where }));
	};

	const cellOf = (id: string) => cells.find(cell => cell.id === id);
	const addressOf = (id: string, where: string | null): string => {
		const cell = cellOf(id);
		if (cell === undefined) return '';

		return frameUrl(
			cell.kind,
			cell.subject,
			cell.variant,
			search.chrome ?? cell.chrome,
			{ t: search.t, rate: search.rate },
			search.theme,
			where
		);
	};
	const srcOf = (id: string): string => addressOf(id, null);
	/** The frame in a tab of its own, where it stands now. Of three widths, the one moved last. */
	const openOf = (id: string): string => addressOf(id, walked[srcOf(id)] ?? null);
	const labelOf = (id: string): string => cellOf(id)?.label ?? id;
	const noteOf = (id: string): string | undefined => cellOf(id)?.note;

	const ops = <Ops ids={ids} search={search} onChange={onChange} notes={noted} />;

	if (total === 0) {
		return (
			<Page name={title} description={description}>
				<div className="wb-stage">
					<p className="wb-missing">The list names no frame the workbench has.</p>
				</div>
			</Page>
		);
	}

	if (widths && cursor !== undefined) {
		const stacked = area.width > 0 && area.width < STACK_BELOW;
		const scale = widthsFit({
			available: area,
			frames: VIEWPORT_NAMES.map(name => VIEWPORTS[name]),
			gap: GAP,
			captionHeight: CAPTION + STANDING,
		});
		/** Side by side, one scale for all three. Stacked, each as wide as the stage lets it be. */
		const widthOf = (name: ViewportName): number =>
			stacked
				? Math.floor(Math.min(VIEWPORTS[name].width, area.width))
				: Math.floor(VIEWPORTS[name].width * scale);
		const order = stacked ? [...VIEWPORT_NAMES].reverse() : VIEWPORT_NAMES;

		return (
			<Page
				name={title}
				description={description}
				readout={`${list === undefined ? 'State' : 'Frame'} ${String(ids.indexOf(cursor) + 1)} of ${String(total)} at three widths${stacked ? '' : `, ${percent(scale)}`}`}
				ops={ops}
				notes={open}
			>
				<div ref={areaRef} className="wb-stage">
					<p className="wb-cap wb-standing">
						<span className="wb-ix">{ids.indexOf(cursor) + 1}</span>
						<span className="wb-capname">{labelOf(cursor)}</span>
						<span className="wb-note">{noteOf(cursor)}</span>
						<span className="wb-acts">
							<a href={openOf(cursor)} target="_blank" rel="noreferrer">
								Open
							</a>
						</span>
					</p>
					<div className={stacked ? 'wb-panes wb-panes-stacked' : 'wb-panes'} style={{ gap: GAP }}>
						{order.map(name => (
							<figure key={name} className="wb-cell" style={{ width: widthOf(name) }}>
								<figcaption className="wb-cap">
									<span className="wb-capname">{WIDTH_LABEL[name]}</span>
									<span className="wb-note">{sized(name)}</span>
								</figcaption>
								<Frame
									src={srcOf(cursor)}
									title={`${labelOf(cursor)}, ${WIDTH_LABEL[name]}`}
									viewport={name}
									tried={trying}
									width={widthOf(name)}
									onPosition={where => {
										onWalk(srcOf(cursor), where);
									}}
								/>
							</figure>
						))}
					</div>
					{below}
				</div>
			</Page>
		);
	}

	const layout = statesFit({
		available: area,
		frame: VIEWPORTS[viewport],
		count: shown.length,
		gap: GAP,
		captionHeight,
		across,
	});
	const unit = list === undefined ? 'state' : 'frame';
	const counted =
		shown.length === total
			? `${String(total)} ${total === 1 ? unit : `${unit}s`}`
			: `${String(shown.length)} of ${String(total)} ${unit}s`;

	return (
		<Page
			name={title}
			description={description}
			readout={`${counted} at ${sized(viewport)}, ${percent(layout.frameWidth / VIEWPORTS[viewport].width)}`}
			ops={ops}
			notes={open}
		>
			<div ref={areaRef} className="wb-stage">
				<div
					className="wb-grid"
					style={{
						gap: GAP,
						gridTemplateColumns: `repeat(${String(layout.columns)}, ${String(layout.frameWidth)}px)`,
					}}
				>
					{shown.map(id => {
						const standing = shown.length > 1 && search.at === id;

						return (
							<figure
								key={id}
								className={standing ? 'wb-cell wb-cell-standing' : 'wb-cell'}
								aria-current={standing ? 'true' : undefined}
							>
								<figcaption className="wb-cap">
									<span className="wb-ix">{ids.indexOf(id) + 1}</span>
									<span className="wb-capname">{labelOf(id)}</span>
									<span className="wb-note">{noteOf(id)}</span>
									<span className="wb-acts">
										{shown.length > 1 && (
											<>
												<button type="button" onClick={() => onChange(only(search, id))}>
													Only
												</button>
												<button type="button" onClick={() => onChange(hide(ids, search, id))}>
													Hide
												</button>
											</>
										)}
										<a href={openOf(id)} target="_blank" rel="noreferrer">
											Open
										</a>
									</span>
								</figcaption>
								<Frame
									src={srcOf(id)}
									title={`${labelOf(id)}, ${sized(viewport)}`}
									viewport={viewport}
									tried={trying}
									width={layout.frameWidth}
									onPosition={where => {
										onWalk(srcOf(id), where);
									}}
								/>
							</figure>
						);
					})}
				</div>
				{below}
			</div>
		</Page>
	);
}
