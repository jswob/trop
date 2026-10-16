import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import {
	APPEARANCE_KEY,
	readAppearance,
	THEME_COLOR,
	useAppearance,
} from '@app/ui/views/appearance';
import { act, renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const INDEX_HTML = resolve(dirname(fileURLToPath(import.meta.url)), '../index.html');
const root = document.documentElement;
let meta: HTMLMetaElement;

beforeEach(() => {
	localStorage.clear();
	delete root.dataset.theme;
	meta = document.createElement('meta');
	meta.name = 'theme-color';
	meta.content = THEME_COLOR.dark;
	document.head.append(meta);
});

afterEach(() => {
	vi.restoreAllMocks();
	delete root.dataset.theme;
	meta.remove();
});

/** `index.html`'s boot: its one inline script, run here as the browser runs it. */
function runBoot(): void {
	const html = readFileSync(INDEX_HTML, 'utf8');
	const code = /<script>([\s\S]*?)<\/script>/.exec(html)?.[1];
	if (!code) throw new Error('index.html has no inline boot script');
	// eslint-disable-next-line @typescript-eslint/no-implied-eval -- Reason: runs index.html's own inline script, the code the browser runs before the first paint
	const boot = new Function(code) as () => void;
	boot();
}

describe("index.html's boot", () => {
	it('names the theme-color meta with the dark ground', () => {
		const html = readFileSync(INDEX_HTML, 'utf8');
		expect(html).toContain(`<meta name="theme-color" content="${THEME_COLOR.dark}" />`);
	});

	it('puts a stored light choice on the root and the bar, before any module runs', () => {
		localStorage.setItem(APPEARANCE_KEY, 'light');
		runBoot();
		expect(readAppearance()).toBe('light');
		expect(meta.content).toBe(THEME_COLOR.light);
	});

	it('leaves the document dark when nothing was chosen, or a value it does not know', () => {
		runBoot();
		expect(readAppearance()).toBe('dark');
		localStorage.setItem(APPEARANCE_KEY, 'sepia');
		runBoot();
		expect(readAppearance()).toBe('dark');
		expect(meta.content).toBe(THEME_COLOR.dark);
	});

	it('leaves the document dark when storage cannot be read', () => {
		vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
			throw new Error('blocked');
		});
		expect(runBoot).not.toThrow();
		expect(readAppearance()).toBe('dark');
	});

	it('reads the key the switch writes', () => {
		const { result } = renderHook(() => useAppearance());
		act(() => {
			result.current.choose('light');
		});
		delete root.dataset.theme;
		meta.content = THEME_COLOR.dark;

		runBoot();
		expect(readAppearance()).toBe('light');
	});
});
