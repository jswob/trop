import { useCallback, useEffect, useRef, useState } from 'react';
import type { PointerEvent as ReactPointerEvent, ReactNode } from 'react';

/**
 * A column that scrolls, whose scrollbar is the rule that ends its page.
 *
 * The column's own bar is switched off and the hairline dividing it from whatever sits to
 * its right reports the scroll instead: 1px of `ui-rule` down the full height, 2px of
 * `ui-edge` over the share on screen. One mark does both jobs and the scroll costs the
 * layout no width, so nothing can hang in the space between the two columns — there is
 * nothing drawn there to hang.
 *
 * The thumb's length is the share on screen and its offset is how far down that share
 * sits, which is the arithmetic every scrollbar does. `ResizeObserver` watches the box and
 * its content because both change without a scroll event: the window resizes, a sheet
 * folds, a row is added.
 *
 * Below `lg` there is no column beside this one and so no rule to carry a thumb; the thin
 * bar comes back at the edge of the glass, where a phone puts it anyway.
 *
 * `rule` is where the hairline is drawn, and `'none'` is for a scroller whose rule is
 * drawn by an ancestor — the panel, whose rule has to run the full height of the column
 * including the head, while the thumb may only report the part that scrolls. The thumb
 * lands on that ancestor's border either way, which is why `edge` is separate.
 */

/** Below this a thumb stops reading as a position and starts reading as a tick. */
const THUMB_MIN = 24;

const RULE = {
	right: 'lg:border-r lg:border-ui-rule',
	left: 'lg:border-l lg:border-ui-rule',
	none: '',
} as const;

export function RuledScroll({
	className = '',
	scrollerClassName = '',
	edge = 'right',
	rule = edge,
	children,
}: {
	className?: string;
	/** Applied to the scrolling box itself — padding belongs here, inside the thumb's line. */
	scrollerClassName?: string;
	/** Which side the thumb rides. */
	edge?: 'left' | 'right';
	/** Which side the hairline is drawn on, or `'none'` when an ancestor draws it. */
	rule?: 'left' | 'right' | 'none';
	children: ReactNode;
}) {
	const scroller = useRef<HTMLDivElement>(null);
	const drag = useRef<{ y: number; top: number } | null>(null);
	const [thumb, setThumb] = useState<{ top: number; height: number } | null>(null);

	const measure = useCallback(() => {
		const box = scroller.current;
		if (!box) return;

		const travel = box.scrollHeight - box.clientHeight;
		if (travel < 1) {
			setThumb(null);
			return;
		}

		const height = Math.max(THUMB_MIN, (box.clientHeight / box.scrollHeight) * box.clientHeight);
		setThumb({ top: ((box.clientHeight - height) * box.scrollTop) / travel, height });
	}, []);

	useEffect(() => {
		const box = scroller.current;
		if (!box) return;

		measure();
		box.addEventListener('scroll', measure, { passive: true });

		const observer = new ResizeObserver(measure);
		observer.observe(box);
		if (box.firstElementChild) observer.observe(box.firstElementChild);

		return () => {
			box.removeEventListener('scroll', measure);
			observer.disconnect();
		};
	}, [measure]);

	function grab(event: ReactPointerEvent<HTMLSpanElement>) {
		const box = scroller.current;
		if (!box) return;
		event.currentTarget.setPointerCapture(event.pointerId);
		drag.current = { y: event.clientY, top: box.scrollTop };
	}

	function move(event: ReactPointerEvent<HTMLSpanElement>) {
		const box = scroller.current;
		const from = drag.current;
		if (!box || !from || !thumb) return;

		const travel = box.clientHeight - thumb.height;
		if (travel <= 0) return;

		box.scrollTop =
			from.top + ((event.clientY - from.y) * (box.scrollHeight - box.clientHeight)) / travel;
	}

	function release(event: ReactPointerEvent<HTMLSpanElement>) {
		event.currentTarget.releasePointerCapture(event.pointerId);
		drag.current = null;
	}

	return (
		/*
		 * `min-w-0`: this root is a flex item wherever it is used — a half of the split, a
		 * column of the rail — and a flex item's automatic minimum is its **min-content**,
		 * not zero. Without it the column refuses to shrink below the widest thing in it, so
		 * at 390 a 437px min-content pushed the frame document into a horizontal scroll and
		 * Chrome put a native bar on both axes. The scroller inside already had it; the box
		 * around the scroller is the one that has to give.
		 */
		<div className={`relative flex min-h-0 min-w-0 ${RULE[rule]} ${className}`}>
			{/*
			 * `relative`: a scroller only clips what it is the containing block for, and a
			 * `static` one is not that for anything absolutely positioned. `sr-only` is
			 * `position: absolute` with no offsets, so every visually-hidden label in the page
			 * sat at its static position — hundreds of pixels down the scrolled content —
			 * resolved against the *root* below, and escaped the clip. One of them pushed the
			 * frame document into a 101px scroll with a native bar to match. The thumb still
			 * rides the root, because it has to stand on the rule rather than travel with the
			 * text.
			 */}
			<div
				ref={scroller}
				className={`ui-scroll relative min-w-0 flex-1 overflow-x-hidden overflow-y-auto lg:ui-scroll-none ${scrollerClassName}`}
			>
				{children}
			</div>
			{thumb ? (
				<span
					aria-hidden
					onPointerDown={grab}
					onPointerMove={move}
					onPointerUp={release}
					onPointerCancel={release}
					style={{ top: thumb.top, height: thumb.height }}
					className={`absolute hidden w-[11px] touch-none lg:block ${
						edge === 'right' ? '-right-[5px]' : '-left-[5px]'
					}`}
				>
					<span
						className={`absolute inset-y-0 w-0.5 bg-ui-edge ${edge === 'right' ? 'right-1' : 'left-1'}`}
					/>
				</span>
			) : null}
		</div>
	);
}
