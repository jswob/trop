import { useCallback, useEffect, useSyncExternalStore, type RefObject } from 'react';

/**
 * The height a shell stands at.
 *
 * **Not `h-screen`.** `100vh` is the *large* viewport: on a phone browser it is the height the glass
 * would have if every collapsible toolbar were retracted, and it stays that height whether they are
 * or not. Safari on iOS opens with its address bar down and its tab bar up, so a shell written
 * `h-screen` is roughly 90px taller than the glass and the whole system's foot — the slug, the last
 * line of the page, the scroller's own bottom edge — is parked underneath the tab bar. Nothing can
 * bring it back, because the shells here deliberately do not let the document scroll: the scrolling
 * happens inside them, and an inner scroller cannot scroll its own container into view.
 *
 * `100dvh` is the viewport as it currently is, toolbars included. It does change as those toolbars
 * collapse, and a design that reflowed on every such change would be worse than the cut — but the
 * toolbars only collapse when the *document* scrolls, and in this system the document never scrolls.
 * So `dvh` is measured once at the width the phone actually shows and then holds still.
 *
 * Inside an iframe — which is how every frame on the stage is loaded — `dvh`, `vh` and `svh` are all
 * the frame's own height, so the workbench sees no change at all. This only ever shows up on glass.
 */
export const SCREEN_H = 'h-dvh';

/** Where a control lands when the keyboard brings it into view: this far below the scroller's top. */
const TYPING_TOP = 12;

/** The input types that raise a keyboard. `checkbox`, `radio`, `button` and friends do not. */
const TYPED = new Set([
	'text',
	'search',
	'email',
	'url',
	'tel',
	'number',
	'password',
	'date',
	'datetime-local',
	'month',
	'time',
	'week',
	'',
]);

/** Where the pointer is a finger, and so where a focused control brings a keyboard up. */
const FINGER = '(hover: none)';

function isTyping(node: EventTarget | null): node is HTMLElement {
	if (node instanceof HTMLTextAreaElement) return !node.readOnly;
	if (!(node instanceof HTMLInputElement)) return false;
	return !node.readOnly && TYPED.has(node.type);
}

/**
 * The nearest ancestor that actually scrolls vertically. `overflow-y: auto` on a box whose content
 * fits is not a scroller — writing `scrollTop` on it does nothing and the search has to carry on up.
 */
function scrollerOf(node: HTMLElement): HTMLElement | null {
	for (let up = node.parentElement; up !== null; up = up.parentElement) {
		const flow = getComputedStyle(up).overflowY;
		if ((flow === 'auto' || flow === 'scroll') && up.scrollHeight > up.clientHeight) return up;
	}
	return null;
}

/**
 * What a control is measured by when the keyboard comes for it.
 *
 * Every typing surface in this kit is a `<label>` wrapping the box — `Select`'s open filter is, and
 * `TextField`'s input *is* the box. So the label, where there is one, is the thing worth putting at
 * the top: the caret in a two-line box is on its last line, and scrolling *that* to the top takes
 * the first line off the screen, which is the line the hand is in the middle of writing.
 */
function anchorOf(node: HTMLElement): HTMLElement {
	return node.closest('label') ?? node;
}

/**
 * **The keyboard moves the page vertically and never horizontally.**
 *
 * Left alone, focusing a control on a phone does two things this system does not want. It scrolls
 * every scrollable ancestor by the *minimum* needed to reveal the control — which parks it at the
 * bottom edge, where the keyboard is about to be — and it scrolls them on **both axes**, so a page
 * that is already exactly as wide as the glass still jolts sideways.
 *
 * This takes the decision back. On focus it writes `scrollTop` on the one scroller the control is
 * in, putting the control's own box {@link TYPING_TOP}px under that scroller's top edge, and it
 * writes nothing else: `scrollLeft` is never touched, so there is no horizontal move to make.
 * Putting the control at the *top* rather than the nearest edge is also what keeps it clear of the
 * keyboard without anything here having to know how tall the keyboard is.
 *
 * It runs **only where the pointer is a finger** (`hover: none`). On a desktop the same rule would
 * yank the page on every Tab, and Tab through a form is exactly the case the browser's own
 * minimal-scroll behaviour gets right.
 *
 * The second pass is for the keyboard itself. iOS does not resize the layout viewport when the
 * keyboard opens — it shrinks the *visual* viewport and pans it — and that happens a beat after
 * focus, sometimes moving things again. `visualViewport`'s `resize` is the event that says the
 * keyboard has arrived, so the same scroll is written once more when it fires and then stopped.
 */
export function useTypingScroll(): void {
	useEffect(() => {
		const touch = window.matchMedia(FINGER);

		const bring = (node: HTMLElement) => {
			const scroller = scrollerOf(node);
			if (scroller === null) return;
			const box = anchorOf(node).getBoundingClientRect();
			scroller.scrollTop += box.top - scroller.getBoundingClientRect().top - TYPING_TOP;
		};

		let waiting: (() => void) | null = null;

		const onFocus = (event: FocusEvent) => {
			waiting?.();
			if (!touch.matches || !isTyping(event.target)) return;

			const node = event.target;
			bring(node);

			/*
			 * Typed `VisualViewport | null`, but absent altogether under jsdom — hence the
			 * `undefined` arm, which the type does not admit and the runtime hands you anyway.
			 */
			const board: VisualViewport | null | undefined = window.visualViewport;
			if (board === null || board === undefined) return;
			const again = () => {
				waiting?.();
				bring(node);
			};
			board.addEventListener('resize', again);
			waiting = () => {
				board.removeEventListener('resize', again);
				waiting = null;
			};
		};

		document.addEventListener('focusin', onFocus);
		return () => {
			waiting?.();
			document.removeEventListener('focusin', onFocus);
		};
	}, []);

	const typing = useTyping();
	useEffect(() => {
		if (typing) return;
		settleDocument();
		const board: VisualViewport | null | undefined = window.visualViewport;
		if (board === null || board === undefined) return;
		board.addEventListener('resize', settleDocument);
		return () => {
			board.removeEventListener('resize', settleDocument);
		};
	}, [typing]);
}

/**
 * **The document goes back to its top once the keyboard has gone.** The shells never let the
 * document scroll (see {@link SCREEN_H}), but iOS pans it anyway to make room for a keyboard, and
 * it pans back only when the field is let go the ordinary way. A field that leaves the document
 * while focused — `Select`'s filter, shut by an outside press — takes the keyboard with it and
 * leaves the pan, so the page's ground showed under the shell where the slug floats. Written once
 * typing ends, and again as the keyboard's retreat resizes the visual viewport. Only where the
 * pointer is a finger, and only when the document has actually moved.
 */
function settleDocument(): void {
	if (typeof window.matchMedia !== 'function' || !window.matchMedia(FINGER).matches) return;
	if (window.scrollY === 0 && window.scrollX === 0) return;
	window.scrollTo(0, 0);
}

/**
 * Focus changes, and one change no focus event reports: **the focused control leaving the
 * document**. `Select`'s filter exists only while its list is open, and an outside press shuts the
 * list without handing focus back, so the field is removed while it holds focus — the keyboard goes,
 * focus falls to the body, and no browser fires `focusout` for it. So while a control that raises a
 * keyboard holds focus, the document is watched until it is gone.
 */
function subscribeTyping(change: () => void): () => void {
	let watch: MutationObserver | null = null;
	const unwatch = () => {
		watch?.disconnect();
		watch = null;
	};
	const onFocus = () => {
		unwatch();
		const held = document.activeElement;
		if (isTyping(held)) {
			watch = new MutationObserver(() => {
				if (held.isConnected) return;
				unwatch();
				change();
			});
			watch.observe(document, { childList: true, subtree: true });
		}
		change();
	};

	document.addEventListener('focusin', onFocus);
	document.addEventListener('focusout', onFocus);
	const finger = typeof window.matchMedia === 'function' ? window.matchMedia(FINGER) : null;
	finger?.addEventListener('change', change);
	onFocus();
	return () => {
		unwatch();
		document.removeEventListener('focusin', onFocus);
		document.removeEventListener('focusout', onFocus);
		finger?.removeEventListener('change', change);
	};
}

/**
 * **Whether the keyboard is up** — a control that raises one has focus, where the pointer is a
 * finger — and, given `within`, for a control inside it.
 *
 * What the chrome gives way to. {@link useTypingScroll} puts the control at the top of its scroller,
 * and the scroller's edges are where the chrome floats: pointers pinned to the column's head and
 * foot, and the slug over the foot below `lg`. Left standing, they cover the control the keyboard
 * came for. The same finger rule as the scroll, so the two never disagree about whether a
 * keyboard came: a landscape tablet has it at the wide layout, a narrow desktop window never does.
 */
export function useTyping(within?: RefObject<Element | null>): boolean {
	const read = useCallback(() => {
		if (typeof window.matchMedia !== 'function' || !window.matchMedia(FINGER).matches) return false;
		const active = document.activeElement;
		if (!isTyping(active)) return false;
		return within === undefined || within.current?.contains(active) === true;
	}, [within]);
	return useSyncExternalStore(subscribeTyping, read, () => false);
}

/**
 * How what the keyboard displaces goes, and comes back: slid off the edge it stands on as the
 * keyboard slides on, over 240ms eased out — one of the frame's own durations, since it moves a
 * boundary — and faded on the way. `visibility` rides along so a thing that has gone takes neither a
 * press nor a Tab; it flips only once the slide is done. None of it under reduced motion.
 */
export const TUCK =
	'transition-[translate,opacity,visibility] duration-[240ms] ease-out motion-reduce:transition-none';

/** What a tucked thing is, beside the way it slides. */
export const TUCKED = 'invisible opacity-0';
