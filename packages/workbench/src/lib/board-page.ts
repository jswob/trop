import type { Appearance } from './appearance';

import { VIEWPORTS } from './viewports';

/** The document `workbench board` writes beside a board's page; every frame on the board opens it. */
export const FRAME_FILE = 'frame.html';

export type BoardFrame = {
	/** What the frame draws, as `lib/board-spec` reads it: `people:person?theme=light`. */
	spec: string;
	width: number;
	height: number;
};

/** How far a frame is shrunk to sit in the room its box has. Never enlarged past its true size. */
export function fitScale(available: number, width: number): number {
	return available <= 0 ? 1 : Math.min(1, available / width);
}

function positive(value: string | undefined, fallback: number): number {
	const parsed = Number(value);
	return Number.isFinite(parsed) && parsed > 0 ? parsed : fallback;
}

/** The frame a box asks for: `data-frame`, and the viewport in `data-width` and `data-height`. */
export function frameBox(element: HTMLElement): BoardFrame {
	return {
		spec: element.dataset['frame'] ?? '',
		width: positive(element.dataset['width'], VIEWPORTS.mobile.width),
		height: positive(element.dataset['height'], VIEWPORTS.mobile.height),
	};
}

function named(spec: string, theme: Appearance): string {
	if (theme === 'dark') return spec;

	const [address = '', query = ''] = spec.split('?', 2);
	const params = new URLSearchParams(query);
	params.set('theme', theme);
	return `${address}?${params.toString()}`;
}

export type Board = {
	/** Puts a frame in a box added after the board started. */
	mount: (element: HTMLElement) => void;
	/** Turns every frame on the board over, and every frame mounted from then on. */
	theme: (next: Appearance) => void;
};

/**
 * Turns every `[data-frame]` box on a board into a real frame.
 *
 * ```html
 * <div data-frame="people:person" data-width="1440" data-height="900"></div>
 * ```
 *
 * **The frame is a genuine viewport at its true size**, for the reason the workbench's own stage
 * loads iframes: a width that is only a box on a wide page fires none of the design's breakpoints,
 * and a board that cannot show the phone rule cannot be judged on it. The frame is laid out at
 * `data-width` by `data-height` and scaled down with a transform to fit its box, so the box may be
 * any width the board's own layout gives it.
 */
export function startBoard(doc: Document): Board {
	let current: Appearance = 'dark';
	const frames = new Set<HTMLIFrameElement>();
	const view = doc.defaultView;

	const mount = (element: HTMLElement): void => {
		const { spec, width, height } = frameBox(element);
		if (spec === '' || element.querySelector('iframe') !== null) return;

		const frame = doc.createElement('iframe');
		frame.setAttribute('name', named(spec, current));
		frame.setAttribute('src', FRAME_FILE);
		frame.setAttribute('title', element.dataset['title'] ?? spec);
		frame.setAttribute('loading', 'lazy');
		Object.assign(frame.style, {
			position: 'absolute',
			top: '0',
			left: '0',
			width: `${width}px`,
			height: `${height}px`,
			border: '0',
			transformOrigin: '0 0',
		});
		Object.assign(element.style, {
			position: 'relative',
			overflow: 'hidden',
			width: '100%',
			maxWidth: `${width}px`,
			aspectRatio: `${width} / ${height}`,
		});

		const fit = (): void => {
			frame.style.transform = `scale(${fitScale(element.clientWidth, width)})`;
		};
		fit();
		if (view !== null && 'ResizeObserver' in view) new view.ResizeObserver(fit).observe(element);

		element.append(frame);
		frames.add(frame);
	};

	doc.querySelectorAll<HTMLElement>('[data-frame]').forEach(mount);

	return {
		mount,
		theme: next => {
			current = next;
			for (const frame of frames) frame.contentWindow?.postMessage({ boardTheme: next }, '*');
		},
	};
}
