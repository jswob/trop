import { useEffect, useRef } from 'react';

/** What a press is a press on: everything a hand can press, as opposed to type into. */
const PRESSED = [
	'button',
	'a[href]',
	'summary',
	'input[type="checkbox"]',
	'input[type="radio"]',
	'[role="button"]',
	'[role="link"]',
	'[role="tab"]',
	'[role="menuitem"]',
	'[role="option"]',
	'[role="checkbox"]',
	'[role="switch"]',
].join(', ');

/** Whether the keyboard has nowhere to be — its control was taken off the page under it. */
export function focusLost(): boolean {
	return document.activeElement === null || document.activeElement === document.body;
}

/** Whether a control can no longer hold the keyboard: off the page, turned off, or not drawn. */
function gone(control: HTMLElement): boolean {
	return (
		!control.isConnected || control.matches(':disabled') || control.getClientRects().length === 0
	);
}

/** How long a press is watched for taking its control away late: a scroll ending, an answer landing. */
const WATCH_MS = 2000;

/** The head of the page that is drawn: the last `h1` that can hold the keyboard. */
export function pageHeadIn(root: ParentNode): HTMLElement | null {
	return (
		Array.from(root.querySelectorAll<HTMLElement>('h1[tabindex="-1"]'))
			.filter(head => head.getClientRects().length > 0)
			.at(-1) ?? null
	);
}

/**
 * **A press that takes its own control away does not leave the keyboard on `<body>`.**
 *
 * `Remove` takes its row with it, a plate is replaced by the run it starts, a way back leaves the
 * page it was on, a plate turns itself off while its errand runs. Each would leave the keyboard with
 * nowhere to be, so the next `Tab` would start from the top of the document. The page's head and the
 * slug's bar can hold the keyboard, and `land` names which; this is the one rule for every press,
 * written once.
 *
 * It acts a frame after a press on a control, and only when that control is gone and nothing else
 * took the keyboard: a page that hands it somewhere better itself is left alone, and so is a
 * control that is still there. Gone is off the page, turned off, or no longer drawn: a press that
 * hides the column its control stands in, as a link on the ledger does below `lg`, is the same press. A
 * control that keeps the keyboard and goes a moment later, as a pointer does once its scroll ends,
 * is watched until it goes or the keyboard moves on. `land` answers with nothing where the caller
 * is not what is drawn.
 *
 * **`away` is the caller put away under a screen that stands in its place**: the ground's failed
 * reads. The press that brings it back is on that screen, and the answer can come long
 * after the press, so no press is watched for it. The keyboard is landed when `away` ends, a frame
 * later and only if nothing holds it.
 */
export function useKeyboardKept(land: () => HTMLElement | null, away = false): void {
	const landing = useRef(land);
	useEffect(() => {
		landing.current = land;
	});

	const wasAway = useRef(away);
	useEffect(() => {
		const was = wasAway.current;
		wasAway.current = away;
		if (!was || away) return;
		const frame = requestAnimationFrame(() => {
			if (focusLost()) landing.current()?.focus({ preventScroll: true });
		});
		return () => {
			cancelAnimationFrame(frame);
		};
	}, [away]);

	useEffect(() => {
		let frame = 0;
		let timer = 0;
		let watch: MutationObserver | null = null;

		const stop = () => {
			cancelAnimationFrame(frame);
			window.clearTimeout(timer);
			watch?.disconnect();
			watch = null;
		};

		const onPress = (event: MouseEvent) => {
			const control =
				event.target instanceof Element ? event.target.closest<HTMLElement>(PRESSED) : null;
			if (control === null) return;

			/**
			 * Whether there is nothing left to wait for: the keyboard was landed, or has moved on. A
			 * control put out of sight still holds the keyboard until the browser lets go of it, a
			 * moment later and with no change to the page to watch for, so that counts as lost too.
			 */
			const settled = (): boolean => {
				const holds = document.activeElement === control;
				if ((holds || focusLost()) && gone(control)) {
					landing.current()?.focus({ preventScroll: true });
					return true;
				}
				return !holds;
			};

			stop();
			frame = requestAnimationFrame(() => {
				if (settled()) return;

				watch = new MutationObserver(() => {
					if (settled()) stop();
				});
				watch.observe(document.body, {
					childList: true,
					subtree: true,
					attributes: true,
					attributeFilter: ['disabled'],
				});
				timer = window.setTimeout(stop, WATCH_MS);
			});
		};

		window.addEventListener('click', onPress);
		return () => {
			stop();
			window.removeEventListener('click', onPress);
		};
	}, []);
}
