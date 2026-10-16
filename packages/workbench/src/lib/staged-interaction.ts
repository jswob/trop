/**
 * Drives an element as soon as it exists, for states that live in a component's
 * own `useState` and have no look-only route through the page.
 *
 * A plain mount effect is enough when the screen renders from the seeded cache
 * in one commit — a card's menu opens that way. It is not enough when the
 * target appears only after a later commit: a form may hydrate its sections in
 * an effect, so nothing is in the DOM when a sibling's effect runs. A `MutationObserver` waits for the commit that puts it there.
 *
 * `requestAnimationFrame` would be the obvious wait and is unusable here: it
 * never fires in a background tab, which is every tab a screenshot is taken of
 * (trap 17). Mutation callbacks are not throttled that way.
 *
 * Attributes are watched as well as children, because a commit that only enables
 * a control changes nothing else: the login form's submit button is `disabled`
 * until its password field is filled, and the fill is the commit before it.
 *
 * `find` carries the guard as well as the search — return `null` once the state
 * is already reached, so the action is idempotent however many times StrictMode
 * runs the effect (trap 31).
 *
 * @param find Returns the element to act on, or nothing if there is none yet.
 * @param act What to do with it once it is there.
 * @returns A teardown that stops watching.
 */
function actWhenPresent<T extends HTMLElement>(
	find: () => T | null | undefined,
	act: (target: T) => void
): () => void {
	let done = false;

	const attempt = (): boolean => {
		const target = find();
		if (!target) return false;
		act(target);
		return true;
	};

	if (attempt()) return () => undefined;

	const observer = new MutationObserver(() => {
		if (done) return;
		if (attempt()) {
			done = true;
			observer.disconnect();
		}
	});

	observer.observe(document.body, { childList: true, subtree: true, attributes: true });

	return () => {
		done = true;
		observer.disconnect();
	};
}

/** Clicks an element as soon as `find` returns one. */
export function clickWhenPresent(find: () => HTMLElement | null | undefined): () => void {
	return actWhenPresent(find, target => {
		target.click();
	});
}

/**
 * Types into a controlled input as soon as `find` returns one.
 *
 * Assigning `value` on a React-controlled input is overwritten on the next
 * render, because React tracks the last value it wrote. Going through the
 * prototype's own setter updates that tracker, and the bubbling `input` event is
 * what React's `onChange` is actually listening for.
 *
 * Trap 31's guard is built in rather than left to `find`: an input already
 * holding `value` is the state this is trying to reach, so a StrictMode rerun
 * has nothing to do.
 */
export function fillWhenPresent(
	find: () => HTMLInputElement | null | undefined,
	value: string
): () => void {
	return actWhenPresent(
		() => {
			const target = find();
			return target && target.value === value ? null : target;
		},
		target => {
			// eslint-disable-next-line @typescript-eslint/unbound-method -- Reason: the prototype's own value setter is taken deliberately and applied to the target below
			const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set;
			setter?.call(target, value);
			target.dispatchEvent(new Event('input', { bubbles: true }));
		}
	);
}
