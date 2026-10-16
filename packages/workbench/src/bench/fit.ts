type Size = { width: number; height: number };

export type Across = 'fit' | 'two' | 'one';

export type StatesLayout = {
	columns: number;
	/** The width a frame is drawn at, in whole pixels. Never more than its own. */
	frameWidth: number;
};

/**
 * The narrowest a frame is drawn before the stage scrolls instead.
 *
 * Below this a desktop frame is a ninth of its size and its caption is one word a line.
 */
export const MIN_FRAME_WIDTH = 160;

/**
 * How the states of a place sit on the stage at one width.
 *
 * `fit` tries every column count and keeps the one that draws the frames largest with all of them
 * on screen. `two` and `one` fill that many columns and let the stage scroll.
 */
export function statesFit({
	available,
	frame,
	count,
	gap,
	captionHeight,
	across,
}: {
	available: Size;
	frame: Size;
	count: number;
	gap: number;
	/** The room one caption takes over its frame. */
	captionHeight: number;
	across: Across;
}): StatesLayout {
	if (available.width <= 0 || available.height <= 0 || count <= 0) {
		return { columns: 1, frameWidth: frame.width };
	}

	const widthIn = (columns: number): number => (available.width - gap * (columns - 1)) / columns;

	if (across !== 'fit') {
		const columns = across === 'two' ? Math.min(2, count) : 1;
		return { columns, frameWidth: Math.floor(Math.min(frame.width, widthIn(columns))) };
	}

	const ratio = frame.height / frame.width;
	let best: StatesLayout = { columns: 1, frameWidth: 0 };

	for (let columns = 1; columns <= count; columns++) {
		const rows = Math.ceil(count / columns);
		const rowHeight = (available.height - gap * (rows - 1)) / rows;
		const width = Math.floor(
			Math.min(frame.width, widthIn(columns), (rowHeight - captionHeight) / ratio)
		);

		if (width > best.frameWidth) best = { columns, frameWidth: width };
	}

	if (best.frameWidth >= MIN_FRAME_WIDTH) return best;

	return {
		columns: Math.max(1, Math.floor((available.width + gap) / (MIN_FRAME_WIDTH + gap))),
		frameWidth: MIN_FRAME_WIDTH,
	};
}

/**
 * The one scale every width of a state is drawn at, side by side.
 *
 * One scale, so the three frames keep their true proportions to each other: a phone beside a
 * desktop is as much narrower as it really is.
 */
export function widthsFit({
	available,
	frames,
	gap,
	captionHeight,
}: {
	available: Size;
	frames: readonly Size[];
	gap: number;
	captionHeight: number;
}): number {
	if (available.width <= 0 || available.height <= 0 || frames.length === 0) return 1;

	const across = frames.reduce((sum, frame) => sum + frame.width, 0);
	const tallest = Math.max(...frames.map(frame => frame.height));

	return Math.max(
		0.05,
		Math.min(
			1,
			(available.width - gap * (frames.length - 1)) / across,
			(available.height - captionHeight) / tallest
		)
	);
}
