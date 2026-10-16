import { useCallback, useState } from 'react';

/** The two ways the app can be drawn. Dark is every screen that has not chosen. */
export type Appearance = 'dark' | 'light';

/** Where the choice is kept: on the device, since there is no user record to keep it on. */
export const APPEARANCE_KEY = 'app-appearance';

/** Account's two words, in the order they are drawn. */
export const APPEARANCE_OPTIONS = [
	{ value: 'dark', word: 'Dark' },
	{ value: 'light', word: 'Light' },
] as const satisfies readonly { value: Appearance; word: string }[];

/**
 * The browser's bar in each mode, painted the mode's ground. `theme.css` holds the same two values
 * as `--color-ui-bg`, and the app's `index.html` names both: dark in its meta, light in its boot.
 */
export const THEME_COLOR = {
	dark: '#101419',
	light: '#f5f7f9',
} as const satisfies Record<Appearance, string>;

/** The device's choice. Dark when nothing was chosen, the value is unknown, or storage is shut. */
export function storedAppearance(): Appearance {
	try {
		return localStorage.getItem(APPEARANCE_KEY) === 'light' ? 'light' : 'dark';
	} catch {
		return 'dark';
	}
}

/**
 * Draws the document in an appearance. Light is `data-theme="light"` on the root, where
 * `theme.css` turns the tokens over; dark is the attribute gone, because the tokens with nothing
 * set already are the dark mode. The browser's bar follows, where the document names one (the
 * workbench's does not).
 */
export function applyAppearance(appearance: Appearance, root = document.documentElement): void {
	if (appearance === 'light') root.dataset.theme = 'light';
	else delete root.dataset.theme;
	root.ownerDocument
		.querySelector('meta[name="theme-color"]')
		?.setAttribute('content', THEME_COLOR[appearance]);
}

/** What the document is drawn in now — the root is the source of truth, not storage. */
export function readAppearance(root = document.documentElement): Appearance {
	return root.dataset.theme === 'light' ? 'light' : 'dark';
}

/**
 * The appearance, and the press that changes it. It starts from the root rather than from
 * storage, so whatever drew the document first — the app's `index.html` boot, or the workbench's
 * `theme` param — is what the switch shows. A choice redraws at once and is kept on the device; a
 * device that refuses the write still redraws, and forgets on reload.
 */
export function useAppearance(): { value: Appearance; choose: (next: Appearance) => void } {
	const [value, setValue] = useState<Appearance>(readAppearance);

	const choose = useCallback((next: Appearance) => {
		applyAppearance(next);
		try {
			localStorage.setItem(APPEARANCE_KEY, next);
		} catch {
			/* Storage is shut (a private window, blocked site data); the redraw still stands. */
		}
		setValue(next);
	}, []);

	return { value, choose };
}
