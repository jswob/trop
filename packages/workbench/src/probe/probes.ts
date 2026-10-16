import type { ProbeResult, ProbeScope } from './report';

/**
 * The measured checks, as functions that run **inside the frame document**.
 *
 * Every one of these was a manual paste into a devtools console before it lived here, and
 * every expensive miss in this loop's history was a measurement that nobody ran. They are
 * self-contained on purpose: Playwright serialises them across to the page, so none of them
 * may close over anything in this module.
 *
 * `PROBES.md` in the skill says which checks are here and which are still hand-run, and why.
 */

/** Which probes fail the command, and which only print. */
export const PROBE_SCOPES: Record<string, ProbeScope> = {
	'h-scroll': 'floor',
	'v-overflow': 'floor',
	'hit-test': 'floor',
	'focus-visible': 'floor',
	'not-empty': 'floor',
	contrast: 'settle',
};

export function scopeOf(probe: string): ProbeScope {
	return PROBE_SCOPES[probe] ?? 'floor';
}

/**
 * **1. The document does not scroll sideways.** One probe, and it finds what the eye does not.
 *
 * A non-zero result is almost always a flex item without `min-width: 0`, or an absolutely
 * positioned child escaping a scroller's clip. The findings name the widest offenders so the
 * next step is a file rather than a hunt.
 */
export function hScroll(): ProbeResult {
	const root = document.documentElement;
	const over = root.scrollWidth - root.clientWidth;
	const edge = root.clientWidth;

	const culprits =
		over === 0
			? []
			: [...document.querySelectorAll<HTMLElement>('*')]
					.map(el => ({ el, right: el.getBoundingClientRect().right }))
					.filter(({ right }) => right > edge + 0.5)
					.sort((a, b) => b.right - a.right)
					.slice(0, 5)
					.map(({ el, right }) => {
						const name = el.tagName.toLowerCase();
						const cls =
							el.className && typeof el.className === 'string'
								? `.${el.className.split(/\s+/).slice(0, 3).join('.')}`
								: '';
						return `${name}${cls} right=${Math.round(right)} (edge ${edge})`;
					});

	return {
		probe: 'h-scroll',
		ok: over === 0,
		note: `scrollWidth ${root.scrollWidth} − clientWidth ${root.clientWidth} = ${over}`,
		findings: culprits,
	};
}

/**
 * **2. Nothing overflows its own scroller vertically that should not.** Content taller than the
 * glass inside a box that is not a scroller is a page running past its frame.
 *
 * Two filters keep this from crying wolf, and a probe that cries wolf is one nobody runs. An
 * inline text leaf reports `scrollHeight` a few pixels over `clientHeight` from its own line box
 * — measured at 15 against 9 on a `<span>` that was drawing perfectly — so only boxes with
 * element children count, and only past a line's worth of overflow.
 */
export function vOverflow(): ProbeResult {
	const offenders = [...document.querySelectorAll<HTMLElement>('body *')]
		.filter(el => {
			const style = getComputedStyle(el);
			if (style.overflowY !== 'visible' && style.overflowY !== 'clip') return false;
			if (style.display === 'inline' || style.display === 'contents') return false;
			if (el.childElementCount === 0) return false;
			return el.scrollHeight - el.clientHeight > 12 && el.clientHeight > 0;
		})
		.slice(0, 5)
		.map(el => {
			const name = el.tagName.toLowerCase();
			return `${name} scrollH=${el.scrollHeight} clientH=${el.clientHeight} overflowY=${getComputedStyle(el).overflowY}`;
		});

	return {
		probe: 'v-overflow',
		ok: offenders.length === 0,
		note: `${offenders.length} non-scroller${offenders.length === 1 ? '' : 's'} taller than its glass`,
		findings: offenders,
	};
}

/**
 * **4. Every interactive target is reachable and hit-tests to itself.** A scrim, an overlay or a
 * transformed ancestor can cover a control without changing how it looks.
 *
 * Filtered by a non-zero rect first: a responsive frame that renders both the wide and the narrow
 * chrome with one hidden matches both in a selector, and the hidden copies hit-test to whatever is
 * on top — which reads exactly like the control being covered.
 *
 * And filtered by the viewport second. `elementFromPoint` is viewport-relative and returns `null`
 * for any coordinate outside it, so every control below the fold reads as covered — measured at
 * five identical false positives on every variant of a scrolling sheet. Below the fold is not
 * covered; it is below the fold, and it is counted separately.
 *
 * **A modal is the one case where being covered is the design.** With a dialog open, the page
 * behind it is supposed to be unreachable, and checking it reports every control on the page as a
 * defect. Two shapes of that: a real `[aria-modal]`, and a bare scrim — an overlay filling the
 * viewport with the dialog painted over it. Anything the scrim covers is reported as behind it
 * rather than as a defect; what is left is what the person can actually reach, which is the only
 * set worth checking.
 */
export function hitTest(): ProbeResult {
	const dialogs = [
		...document.querySelectorAll<HTMLElement>('[aria-modal="true"], dialog[open]'),
	].filter(el => el.getBoundingClientRect().width > 0);
	const scope: ParentNode = dialogs[dialogs.length - 1] ?? document;

	const targets = [
		...scope.querySelectorAll<HTMLElement>(
			'a[href], button, input, select, textarea, [tabindex]:not([tabindex="-1"])'
		),
	].filter(el => {
		const rect = el.getBoundingClientRect();
		return rect.width > 0 && rect.height > 0;
	});

	/*
	 * The point a hit test reads is the target's centre, and a target is only reachable there if
	 * that point is inside **every** box that clips it — the viewport, and any scrolling ancestor.
	 * A row half-cut by the top edge of a column has its centre on the clip line: the viewport
	 * filter admits it, the scroller paints nothing there, and `elementFromPoint` falls through to
	 * whatever wrapper is behind — which reads exactly like a covered control and is not one.
	 * Measured at 12 of 45 frames, on a column parked with a row straddling a
	 * scroll port that starts 8px below the viewport's own top.
	 */
	const clipOf = (el: HTMLElement): DOMRect => {
		let box = new DOMRect(0, 0, window.innerWidth, window.innerHeight);
		for (let node = el.parentElement; node; node = node.parentElement) {
			const overflow = getComputedStyle(node).overflowY;
			if (overflow !== 'auto' && overflow !== 'scroll' && overflow !== 'hidden') continue;
			const rect = node.getBoundingClientRect();
			if (rect.width === 0 && rect.height === 0) continue;
			box = new DOMRect(
				Math.max(box.x, rect.x),
				Math.max(box.y, rect.y),
				Math.min(box.right, rect.right) - Math.max(box.x, rect.x),
				Math.min(box.bottom, rect.bottom) - Math.max(box.y, rect.y)
			);
		}
		return box;
	};

	const inViewport = targets.filter(el => {
		const rect = el.getBoundingClientRect();
		const x = rect.x + rect.width / 2;
		const y = rect.y + rect.height / 2;
		const clip = clipOf(el);
		return x >= clip.x && y >= clip.y && x <= clip.right && y <= clip.bottom;
	});

	const scrims = [...document.querySelectorAll<HTMLElement>('*')].filter(el => {
		const rect = el.getBoundingClientRect();
		const position = getComputedStyle(el).position;
		if (position !== 'fixed' && position !== 'absolute') return false;
		return rect.width >= window.innerWidth * 0.95 && rect.height >= window.innerHeight * 0.95;
	});

	const hitOf = (el: HTMLElement): Element | null => {
		const rect = el.getBoundingClientRect();
		return document.elementFromPoint(rect.x + rect.width / 2, rect.y + rect.height / 2);
	};

	/*
	 * **What covers a control is the floating object, not the leaf under the pointer.**
	 *
	 * `elementFromPoint` returns the deepest node, so one slug at the foot of a column reported
	 * itself as `svg`, `span.truncate.text-body`, `span.ml-auto.shrink-0`, `button.flex.h-touch`
	 * and `div.w-full.max-w-[20rem]` depending on which pixel was tested — five names for one
	 * object, which reads as five unrelated defects until somebody walks the parent chain. That
	 * walk was done by hand once and it is mechanical, so it is done here.
	 *
	 * The nearest positioned ancestor is the right answer because a floating object is floating by
	 * virtue of its own `position`; its children are carried. Naming it also makes a finding
	 * **stable**, which matters more than the tidier output: the baseline keys findings by these
	 * words, and a key built on a leaf class churns on any markup edit inside the object and comes
	 * back as a new finding.
	 */
	const coveringObject = (hit: Element): Element => {
		for (let node: Element | null = hit; node; node = node.parentElement) {
			if (node === document.body || node === document.documentElement) break;
			const position = getComputedStyle(node).position;
			if (position === 'fixed' || position === 'absolute' || position === 'sticky') return node;
		}
		return hit;
	};

	const behindScrim = inViewport.filter(el => {
		const hit = hitOf(el);
		return hit !== null && !el.contains(hit) && scrims.some(scrim => scrim.contains(hit));
	});

	/*
	 * **Counted before it is cut.** This list used to be `.slice(0, 5)`-ed and then have its
	 * `.length` printed, so the note said "5 covered" on a frame with thirty — `held-register` at
	 * 390 measured 30. Harmless while every finding is expected, and not harmless at all
	 * underneath a baseline: the one number `PROBES.md` says to watch on a floating object is
	 * whether it **grew**, and a count pinned at 5 cannot grow.
	 */
	const covered = inViewport
		.filter(el => !behindScrim.includes(el))
		.filter(el => {
			const hit = hitOf(el);
			return !(hit && el.contains(hit));
		});

	/*
	 * One finding per covering object, carrying its count — because that is what a baseline should
	 * key on. A finding per target keys on a label, and then a covered control gaining a word comes
	 * back as a new finding while thirty more appearing behind the same slug does not.
	 */
	const byObject = new Map<string, string[]>();

	for (const el of covered) {
		const hit = hitOf(el);
		const object = hit === null ? null : coveringObject(hit);
		const by = object
			? `${object.tagName.toLowerCase()}${object.className && typeof object.className === 'string' ? `.${object.className.split(/\s+/).slice(0, 2).join('.')}` : ''}`
			: 'nothing (outside the viewport)';
		const label = el.getAttribute('aria-label') ?? el.textContent?.trim().slice(0, 30) ?? '';
		byObject.set(by, [...(byObject.get(by) ?? []), `${el.tagName.toLowerCase()} "${label}"`]);
	}

	const findings = [...byObject.entries()].map(
		([by, labels]) => `${labels.length} target${labels.length === 1 ? '' : 's'} covered by ${by}`
	);

	/* The names stay in the note, which nothing keys on, so they can churn without crying wolf. */
	const examples = [...byObject.values()].flat().slice(0, 4).join(' · ');

	return {
		probe: 'hit-test',
		ok: covered.length === 0,
		note: `${inViewport.length - behindScrim.length} reachable target${inViewport.length - behindScrim.length === 1 ? '' : 's'}, ${covered.length} covered · ${targets.length - inViewport.length} below the fold, ${behindScrim.length} behind a scrim${dialogs.length > 0 ? ', scoped to the open dialog' : ''}${examples === '' ? '' : ` — ${examples}`}`,
		findings,
	};
}

/**
 * **7. Every state draws at every width.** A variant is a state of the app, never a screen size.
 *
 * This is the check that catches a frame whose halves are `hidden lg:flex` / `lg:hidden`, where
 * half the stage is blank at any width you pick.
 */
export function notEmpty(): ProbeResult {
	const root = document.getElementById('root') ?? document.body;
	const text = (root.textContent ?? '').trim().length;
	const painted = [...root.querySelectorAll<HTMLElement>('*')].filter(el => {
		const rect = el.getBoundingClientRect();
		return rect.width > 2 && rect.height > 2;
	}).length;

	return {
		probe: 'not-empty',
		ok: text > 0 || painted > 3,
		note: `${text} chars of text, ${painted} painted boxes`,
		findings: text === 0 && painted <= 3 ? ['the frame renders an empty box at this width'] : [],
	};
}

/**
 * **10. Contrast, measured.** Every ink on every ground it is actually drawn on, against AA.
 *
 * A token's documented ratio is the ratio on the ground it was specified against, not on the one
 * it ended up on — which is the whole reason this is measured per element rather than read off the
 * palette. An element whose ground is a gradient or an image is reported as unresolved rather than
 * guessed at; those are the ones to look at by hand.
 */
export function contrast(): ProbeResult {
	function parse(value: string): [number, number, number, number] | null {
		const nums = value.match(/[\d.]+/g);
		if (!nums || nums.length < 3) return null;
		const [r, g, b, a] = nums.map(Number) as [number, number, number, number?];
		return [r, g, b, a ?? 1];
	}

	function luminance([r, g, b]: [number, number, number, number]): number {
		const channel = (raw: number) => {
			const c = raw / 255;
			return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
		};
		return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
	}

	function ratio(ink: [number, number, number, number], ground: [number, number, number, number]) {
		const a = luminance(ink);
		const b = luminance(ground);
		return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
	}

	const unresolved: string[] = [];
	const failing: string[] = [];
	let checked = 0;

	const texts = [...document.querySelectorAll<HTMLElement>('body *')].filter(el =>
		[...el.childNodes].some(
			node => node.nodeType === 3 && (node.textContent ?? '').trim().length > 0
		)
	);

	for (const el of texts) {
		const style = getComputedStyle(el);
		const rect = el.getBoundingClientRect();
		if (rect.width === 0 || rect.height === 0) continue;

		const ink = parse(style.color);
		if (!ink || ink[3] === 0) continue;

		let ground: [number, number, number, number] | null = null;
		let imaged = false;
		for (let node: HTMLElement | null = el; node; node = node.parentElement) {
			const nodeStyle = getComputedStyle(node);
			if (nodeStyle.backgroundImage !== 'none') {
				imaged = true;
				break;
			}
			const candidate = parse(nodeStyle.backgroundColor);
			if (candidate && candidate[3] === 1) {
				ground = candidate;
				break;
			}
		}

		const label = `${el.tagName.toLowerCase()} "${(el.textContent ?? '').trim().slice(0, 24)}"`;

		if (imaged) {
			if (unresolved.length < 5)
				unresolved.push(`${label} sits on a gradient or image — measure by hand`);
			continue;
		}
		if (!ground) continue;

		const size = parseFloat(style.fontSize);
		const bold = Number(style.fontWeight) >= 700;
		const large = size >= 24 || (size >= 18.66 && bold);
		const required = large ? 3 : 4.5;
		const measured = ratio(ink, ground);
		checked += 1;

		if (measured < required && failing.length < 8) {
			failing.push(
				`${label} ${measured.toFixed(2)}:1 against ${style.color} on ${ground.slice(0, 3).join(',')} — needs ${required}`
			);
		}
	}

	return {
		probe: 'contrast',
		ok: failing.length === 0,
		note: `${checked} ink/ground pairs measured, ${failing.length} under AA, ${unresolved.length} unresolved`,
		findings: [...failing, ...unresolved],
	};
}

/**
 * **6. Focus is visible on every control.**
 *
 * Two evaluates rather than one, because `:focus-visible` is the thing being checked and a
 * programmatic `.focus()` does not set it — only a real Tab does. The runner installs the
 * baselines, presses Tab, and reads back what changed.
 */
export function installFocusBaselines(): number {
	/* A page that opens with the keyboard on a control would measure that control's ring as its
	   "before" and report it as showing nothing (60j), so the baselines are taken unfocused. */
	if (document.activeElement instanceof HTMLElement) document.activeElement.blur();

	const targets = [
		...document.querySelectorAll<HTMLElement>(
			'a[href], button, input, select, textarea, [tabindex]:not([tabindex="-1"])'
		),
	].filter(el => {
		/* A disabled or inert control is correctly unreachable; counting it reports nine
		   phantom a11y defects on a frame whose keyboard order is perfect. */
		if (
			el.matches(':disabled') ||
			el.closest('[inert]') ||
			el.getAttribute('aria-hidden') === 'true'
		) {
			return false;
		}
		const rect = el.getBoundingClientRect();
		return rect.width > 0 && rect.height > 0;
	});

	const baselines: Record<string, string> = {};
	targets.forEach((el, index) => {
		el.setAttribute('data-probe-idx', String(index));
		const style = getComputedStyle(el);
		baselines[String(index)] = [
			style.outline,
			style.boxShadow,
			style.borderColor,
			style.backgroundColor,
		].join('|');
	});

	(window as unknown as Record<string, unknown>).__PROBE_FOCUS__ = baselines;
	return targets.length;
}

/** What the currently focused element looks like, against the baseline taken before it had focus. */
export function readFocusState(): { index: string | null; label: string; changed: boolean } | null {
	const el = document.activeElement as HTMLElement | null;
	if (!el || el === document.body) return null;

	const index = el.getAttribute('data-probe-idx');
	const baselines = (window as unknown as Record<string, unknown>).__PROBE_FOCUS__ as
		Record<string, string> | undefined;
	const style = getComputedStyle(el);
	const now = [style.outline, style.boxShadow, style.borderColor, style.backgroundColor].join('|');
	const before = index !== null && baselines ? baselines[index] : undefined;

	return {
		index,
		label: `${el.tagName.toLowerCase()} "${(el.getAttribute('aria-label') ?? el.textContent ?? '').trim().slice(0, 30)}"`,
		changed: before === undefined ? true : now !== before,
	};
}
