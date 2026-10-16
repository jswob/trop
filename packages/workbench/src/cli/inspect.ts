/** One interactive target, as the frame itself reports it. */
export type Target = {
	role: string;
	/** The accessible name — `aria-labelledby`, then `aria-label`, then the visible text. */
	name: string;
	tag: string;
	disabled: boolean;
	/** Whether it has a box at all. A responsive frame renders both chromes and hides one. */
	visible: boolean;
	x: number;
	y: number;
	w: number;
	h: number;
	/**
	 * The same box in **content** coordinates — viewport position plus every scroll offset above it.
	 *
	 * A click has to bring its target into view before it can press it, so a press near the foot of
	 * a column scrolls that column, and every viewport rect in it changes. Read that way, an inert
	 * button reports 67 targets "moved" and the run concludes the press reached a state. Adding the
	 * scroll back takes the scrolling out of the measurement: content coordinates move when the
	 * layout moves and stay put when the reader does.
	 */
	cx: number;
	cy: number;
	/**
	 * Whether a press on it is a fair question: `press` for a control that should do something,
	 * `chosen` for one already standing on its answer (a pressed toggle, the current tab), and `no`
	 * for what is typed in rather than pressed, what leaves for another window, and what an open
	 * modal or an `inert` ancestor has put out of reach.
	 */
	press: 'press' | 'chosen' | 'no';
	/** What it is one of: tag, role and classes. Sixty rows of one list share it. */
	like: string;
};

/** Everything one read of a frame yields: what is on it, and how much of it there is. */
export type FrameRead = {
	chars: number;
	/** A sum over the text itself, so words swapped for others of the same length still show. */
	words: number;
	boxes: number;
	scrollY: number;
	/** What the open dialog calls itself, or `null`. A modal is a state, not a defect. */
	dialog: string | null;
	/**
	 * The largest `column-count` actually applied anywhere on the page.
	 *
	 * A round deleted a `collapsed` variant on the evidence *"1 → 2 columns"*, and counting
	 * characters cannot produce that number: collapsing a side column clips a 596px column away and
	 * reflows the page onto the whole glass **without changing a word of text**. A diff made only
	 * of content reads that as `text —, boxes −6` and all but says nothing happened.
	 */
	columns: number;
	/**
	 * The appearance the root is drawn in. A press on an appearance switch repaints every
	 * token and moves no text and no box, so without this line it reads as an inert control.
	 */
	theme: 'dark' | 'light';
	/** The union of every drawn target's box — what the frame is actually using, in px. */
	extent: { x: number; right: number; y: number; bottom: number };
	targets: Target[];
};

/** Where `readFrame` parks the live elements, so a step can be handed the one it matched. */
export const HANDLE_KEY = '__DESIGN_TARGETS__';

/**
 * Reads a frame, **inside the frame document**.
 *
 * Self-contained on purpose, exactly as `probe/probes.ts` is: Playwright serialises this across
 * to the page by `toString()`, so it may not close over anything in this module. That is also why
 * it is one function rather than a `listTargets` and a `snapshot` — two of them would mean two
 * copies of the accessible-name rules, which is `TRAPS.md`'s *one enum, two drawings* in a place
 * nobody would think to census.
 *
 * It parks the matched elements on `window` as it goes. A driver that re-queried by name to press
 * something would be resolving the name twice against two different DOMs — once to report it and
 * once to click it — and the whole point of reporting the name is that the press used that one.
 */
export function readFrame(): FrameRead {
	const SELECTOR = [
		'a[href]',
		'button',
		'input',
		'select',
		'textarea',
		'summary',
		'[role="button"]',
		'[role="link"]',
		'[role="tab"]',
		'[role="menuitem"]',
		'[role="option"]',
		'[role="checkbox"]',
		'[role="switch"]',
		'[tabindex]:not([tabindex="-1"])',
	].join(', ');

	const clean = (value: string): string => value.replace(/\s+/g, ' ').trim();

	const nameOf = (element: Element): string => {
		const labelledBy = element.getAttribute('aria-labelledby');
		if (labelledBy !== null) {
			const joined = clean(
				labelledBy
					.split(/\s+/)
					.map(id => document.getElementById(id)?.textContent ?? '')
					.join(' ')
			);
			if (joined !== '') return joined;
		}

		const label = element.getAttribute('aria-label');
		if (label !== null && clean(label) !== '') return clean(label);

		const own = element as HTMLElement;
		const text = clean(own.innerText ?? own.textContent ?? '');
		if (text !== '') return text;

		const placeholder = element.getAttribute('placeholder');
		if (placeholder !== null && clean(placeholder) !== '') return clean(placeholder);

		const title = element.getAttribute('title');
		return title === null ? '' : clean(title);
	};

	/** Every scroll offset between an element and the document, which is what a click can change. */
	const scrolledBy = (element: HTMLElement): { left: number; top: number } => {
		let left = window.scrollX;
		let top = window.scrollY;

		for (let node = element.parentElement; node; node = node.parentElement) {
			const style = getComputedStyle(node);
			const scrolls = (value: string): boolean => value === 'auto' || value === 'scroll';
			if (scrolls(style.overflowY) || scrolls(style.overflowX)) {
				left += node.scrollLeft;
				top += node.scrollTop;
			}
		}

		return { left, top };
	};

	const modal = document.querySelector<HTMLElement>('[aria-modal="true"], dialog[open]');

	const pressOf = (element: HTMLElement): 'press' | 'chosen' | 'no' => {
		const tag = element.tagName.toLowerCase();
		const role = element.getAttribute('role');
		const type = (element.getAttribute('type') ?? '').toLowerCase();

		const typed =
			tag === 'select' ||
			tag === 'textarea' ||
			(tag === 'input' && !['button', 'submit', 'reset', 'checkbox', 'radio'].includes(type));
		const pressed =
			tag === 'button' ||
			tag === 'summary' ||
			tag === 'a' ||
			tag === 'input' ||
			['button', 'link', 'tab', 'menuitem', 'option', 'checkbox', 'switch'].includes(role ?? '');

		if (typed || !pressed) return 'no';
		if (tag === 'a' && element.getAttribute('target') === '_blank') return 'no';
		if (element.closest('[inert]') !== null) return 'no';
		if (modal !== null && !modal.contains(element)) return 'no';

		const standing =
			element.getAttribute('aria-pressed') === 'true' ||
			element.getAttribute('aria-selected') === 'true' ||
			element.getAttribute('aria-checked') === 'true' ||
			(element.hasAttribute('aria-current') && element.getAttribute('aria-current') !== 'false') ||
			(type === 'radio' && (element as HTMLInputElement).checked);

		return standing ? 'chosen' : 'press';
	};

	const elements = [...document.querySelectorAll<HTMLElement>(SELECTOR)];
	(window as unknown as Record<string, unknown>)['__DESIGN_TARGETS__'] = elements;

	const targets = elements.map(element => {
		const rect = element.getBoundingClientRect();
		const style = getComputedStyle(element);
		const input = element as HTMLInputElement;
		const scroll = scrolledBy(element);

		return {
			role: element.getAttribute('role') ?? element.tagName.toLowerCase(),
			name: nameOf(element),
			tag: element.tagName.toLowerCase(),
			disabled: input.disabled === true || element.getAttribute('aria-disabled') === 'true',
			visible:
				rect.width > 0 && rect.height > 0 && style.visibility !== 'hidden' && style.opacity !== '0',
			x: Math.round(rect.x),
			y: Math.round(rect.y),
			w: Math.round(rect.width),
			h: Math.round(rect.height),
			cx: Math.round(rect.x + scroll.left),
			cy: Math.round(rect.y + scroll.top),
			press: pressOf(element),
			like: `${element.tagName.toLowerCase()}|${element.getAttribute('role') ?? ''}|${element.className}`,
		};
	});

	const boxes = [...document.querySelectorAll<HTMLElement>('body *')].filter(element => {
		const rect = element.getBoundingClientRect();
		return rect.width > 0 && rect.height > 0;
	}).length;

	const dialog = document.querySelector<HTMLElement>('[aria-modal="true"], dialog[open]');

	const columns = [...document.querySelectorAll<HTMLElement>('body *')].reduce((most, element) => {
		const count = Number.parseInt(getComputedStyle(element).columnCount, 10);
		return Number.isFinite(count) && count > most ? count : most;
	}, 1);

	const drawn = targets.filter(target => target.visible);
	const extent =
		drawn.length === 0
			? { x: 0, right: 0, y: 0, bottom: 0 }
			: {
					x: Math.min(...drawn.map(target => target.x)),
					right: Math.max(...drawn.map(target => target.x + target.w)),
					y: Math.min(...drawn.map(target => target.y)),
					bottom: Math.max(...drawn.map(target => target.y + target.h)),
				};

	const text = clean(document.body.innerText ?? '');
	let words = 0;
	for (let at = 0; at < text.length; at += 1) words = (words * 31 + text.charCodeAt(at)) | 0;

	return {
		chars: text.length,
		words,
		boxes,
		scrollY: Math.round(window.scrollY),
		dialog: dialog === null ? null : clean(dialog.innerText ?? '').slice(0, 60),
		columns,
		theme: document.documentElement.dataset.theme === 'light' ? 'light' : 'dark',
		extent,
		targets,
	};
}
