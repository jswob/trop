type Typed = { tagName?: string; isContentEditable?: boolean } | null;

/**
 * Hands the keys pressed inside a frame to the page around it.
 *
 * A frame is the app, live, and the app takes the focus: half the places open with it already
 * inside the frame, and one press on a frame puts it there. A key the bar prints then went to the
 * frame and did nothing. So the frame's own window repeats each key on the page's, unless the
 * frame is being typed into, and gives up its own default when the page took the key.
 *
 * The target is read by its tag rather than by `instanceof`: an element in a frame is not an
 * instance of this window's `HTMLElement`.
 */
export function forwardKeys(inner: Window, outer: Window): void {
	inner.addEventListener('keydown', pressed => {
		const target = pressed.target as Typed;
		const typing =
			target?.isContentEditable === true ||
			['INPUT', 'TEXTAREA', 'SELECT'].includes(target?.tagName ?? '');
		if (typing) return;

		const repeated = new KeyboardEvent('keydown', {
			key: pressed.key,
			code: pressed.code,
			metaKey: pressed.metaKey,
			ctrlKey: pressed.ctrlKey,
			altKey: pressed.altKey,
			shiftKey: pressed.shiftKey,
			bubbles: true,
			cancelable: true,
		});

		outer.dispatchEvent(repeated);
		if (repeated.defaultPrevented) pressed.preventDefault();
	});
}
