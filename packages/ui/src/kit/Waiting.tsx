import { TextButton } from '@app/ui/kit/Button';
import { Figure } from '@app/ui/kit/Figure';
import {
	createContext,
	useContext,
	useLayoutEffect,
	useRef,
	type CSSProperties,
	type ReactNode,
} from 'react';
import { CONTROL_H } from '@app/ui/kit/control';
import { useInStep } from '@app/ui/kit/in-step';

/**
 * What a page draws while it waits, and how it says it is waiting rather than hung.
 *
 * A still placeholder is exactly what a stuck app looks like, which is the whole problem: the
 * drawing meant to say *hold on* said nothing at all. So the footprints sit under a **glaze** — a
 * train of bands passing down the page, top to bottom — and a note beside them prints the word and
 * counts the seconds.
 *
 * Reach for it wherever the client is waiting on something it asked for. It is not an empty state:
 * an empty screen knows what it holds and says so in words, and drawing footprints there promises
 * contents that are never coming.
 *
 * **The placeholder claims nothing it cannot know.** No hue, because a load has no measure and
 * inventing one so a waiting page can have some colour is a lie. No control ink — `ui-edge` is what
 * says *operable* and a footprint is not — and no slot drawn empty or filled, because that is a
 * state this drawing cannot yet know. The page head is not a placeholder either: it is real, and it
 * is first.
 *
 * The glaze itself is in `packages/ui/src/glaze.css`, because it is a real stylesheet animation and the
 * workbench pins motion by scrubbing what `getAnimations()` reports. This file is the geometry the
 * CSS needs, and the shapes it falls on.
 */

/** What a wait says once what it was waiting on has failed. */
export const READ_FAILED = 'Can’t reach the server.';

/** The press that asks again for what failed. */
export const TRY_AGAIN = 'Try again';

/**
 * **Whether the waits on screen have failed, and how to ask again.** `null` while nothing has.
 *
 * The frame knows a read has failed; a wait does not, and every page draws its own. So the one fact
 * travels down as context and every wait under it says it the same way: the glaze stops, the
 * seconds go, and the note reads {@link READ_FAILED} with {@link TRY_AGAIN} beside it. A wait with
 * no provider above it is the loader it always was.
 */
const WaitFailedContext = createContext<(() => void) | null>(null);

/** Whether the waits on screen have failed — for a wait that places its own note. */
export function useWaitFailed(): boolean {
	return useContext(WaitFailedContext) !== null;
}

/** Puts every wait under it into its failed state, `onRetry` behind its `Try again`. */
export function WaitFailed({
	onRetry,
	children,
}: {
	/** Asks again for what failed; `null` while nothing has, which leaves every wait loading. */
	onRetry: (() => void) | null;
	children: ReactNode;
}) {
	return <WaitFailedContext.Provider value={onRetry}>{children}</WaitFailedContext.Provider>;
}

/**
 * The pitch between one crest and the next, as a share of the body's height.
 *
 * A fraction rather than a fixed number of pixels, so a page carries the same number of waves
 * whatever height it folds to — the same gesture at every width rather than the same physical hand.
 * The lit part of a tile is 48% of it and the trough the other 52%, so some band is always on the
 * page and the crests still stay countable.
 */
const TILE = 0.7;

/**
 * The glazed body: it publishes the geometry, and the stylesheet does the rest.
 *
 * The measurement is the component's own rather than a hook beside it, because there is no state
 * here a caller could own — the numbers are read off the DOM the caller already described. What the
 * caller decides is what the footprints are and where they sit; this decides nothing but how far
 * down the page each one landed.
 *
 * **Every placeholder's offset is measured, never computed from the content.** The same rows are a
 * different height at 390 than at 1440, so a field positioned from the shape would be right at one
 * width and wrong at the other. The observer is what keeps that true through a fold.
 */
export function Waiting({
	children,
	at,
	className = '',
}: {
	children: ReactNode;
	/**
	 * Pin the field at this point in its pass, `0` to `1`, instead of running it.
	 *
	 * For a specimen: a component whose whole message is motion has to be reviewable as a
	 * filmstrip, and a page that renders the finished state has nothing to pin. Undefined
	 * everywhere else, which is the running loader.
	 */
	at?: number;
	className?: string;
}) {
	const bodyRef = useRef<HTMLDivElement>(null);

	useLayoutEffect(() => {
		const body = bodyRef.current;
		if (!body) return;

		const measure = () => {
			const box = body.getBoundingClientRect();
			const tile = Math.round((box.height || 1) * TILE);
			body.style.setProperty('--ui-glaze-tile', `${String(tile)}px`);

			for (const bar of body.querySelectorAll<HTMLElement>('[data-ui-glaze-bar]')) {
				const rect = bar.getBoundingClientRect();

				/*
				 * A placeholder the fold has taken off the page reports an all-zero rect, which would
				 * otherwise be written down as an offset of minus the body's own distance from the
				 * viewport — a number about the window rather than about the page. Clearing it leaves the
				 * property unset, and the fold that brings the element back changes the body's height,
				 * which is what runs this again.
				 */
				if (rect.width === 0 && rect.height === 0) {
					bar.style.removeProperty('--ui-glaze-top');
					continue;
				}

				bar.style.setProperty('--ui-glaze-top', `${String(Math.round(rect.top - box.top))}px`);
			}
		};

		measure();
		const observer = new ResizeObserver(measure);
		observer.observe(body);
		return () => {
			observer.disconnect();
		};
	}, []);

	const failed = useContext(WaitFailedContext) !== null;
	useInStep(bodyRef, 'ui-glaze', at === undefined && !failed);

	return (
		<div
			ref={bodyRef}
			data-ui-glaze-body=""
			data-ui-glaze-at={at === undefined ? undefined : ''}
			data-ui-glaze-failed={failed ? '' : undefined}
			className={className}
			style={at === undefined ? undefined : ({ '--ui-glaze-at': at } as CSSProperties)}
		>
			{children}
		</div>
	);
}

/**
 * One footprint — a block, never a box, because nothing here is operable yet.
 *
 * It carries the glaze rather than an animation of its own: `ui-glaze-bar` paints the slice of the
 * body's field that crosses this element, so two placeholders that touch are two parts of one
 * gradient and a tall one has the gradient running through it.
 */
export function Placeholder({
	width,
	height = 9,
	className = '',
}: {
	width: number | string;
	height?: number | string;
	className?: string;
}) {
	return (
		<span
			aria-hidden
			data-ui-glaze-bar=""
			className={`block bg-ui-row ui-glaze-bar ${className}`}
			style={{ width: typeof width === 'number' ? `${String(width)}px` : width, height }}
		/>
	);
}

/**
 * A run of footprints at the row pitch, on the rules a run of rows would have drawn.
 *
 * The generic shape, for a page whose contents are not known well enough to draw truly — which is
 * every destination that has not been designed yet. A place that knows its own anatomy draws that
 * instead, out of `Placeholder`; the ledger does exactly that.
 *
 * The widths are arithmetic rather than random so the drawing is identical between two renders of
 * the same wait. A placeholder that reshuffles on every paint is a second animation nobody asked
 * for.
 */
export function PlaceholderRows({ rows = 8 }: { rows?: number }) {
	return (
		<div aria-hidden>
			{Array.from({ length: rows }, (_element, index) => (
				<div key={index} className={`flex ${CONTROL_H} items-center border-b border-ui-rule`}>
					<Placeholder width={100 + ((index * 37) % 140)} />
				</div>
			))}
		</div>
	);
}

/**
 * The word, and the seconds behind it.
 *
 * The glaze is the carrier, and a carrier with no text equivalent is the same failure as colour with
 * no shape behind it — so this is not optional furniture, it is the half of the message that
 * survives `prefers-reduced-motion` and a screen reader. `role="status"` is what announces it.
 *
 * The count is drawn by CSS rather than by a timer: a `setInterval` would be a second clock the
 * workbench cannot scrub, and the whole point of the figure is that it proves time is passing. It is
 * decoration for the reader who cannot see it — the word is what carries the state — and a figure so
 * it lines up like every other number in the system.
 */
export function WaitingNote({ className = '' }: { className?: string }) {
	const retry = useContext(WaitFailedContext);
	if (retry !== null) {
		return (
			<span role="status" className={`text-ui-body ${className}`}>
				<span className="text-ui-red">{READ_FAILED}</span>{' '}
				<TextButton inline ink="ink" onClick={retry}>
					{TRY_AGAIN}
				</TextButton>
			</span>
		);
	}
	return (
		<span role="status" className={`text-ui-body text-ui-ink-3 ${className}`}>
			loading{' '}
			<Figure>
				<span aria-hidden className="ui-elapsed" />
			</Figure>
		</span>
	);
}
