import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { act, renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import {
	APPEARANCE_KEY,
	applyAppearance,
	readAppearance,
	storedAppearance,
	THEME_COLOR,
	useAppearance,
} from './appearance';

const HERE = dirname(fileURLToPath(import.meta.url));
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

describe('storedAppearance', () => {
	it('is dark when nothing was chosen', () => {
		expect(storedAppearance()).toBe('dark');
	});

	it('reads light back', () => {
		localStorage.setItem(APPEARANCE_KEY, 'light');
		expect(storedAppearance()).toBe('light');
	});

	it('is dark for a value it does not know', () => {
		localStorage.setItem(APPEARANCE_KEY, 'sepia');
		expect(storedAppearance()).toBe('dark');
	});

	it('is dark when storage cannot be read', () => {
		vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
			throw new Error('blocked');
		});
		expect(storedAppearance()).toBe('dark');
	});
});

describe('applyAppearance', () => {
	it('marks the root light', () => {
		applyAppearance('light');
		expect(root.dataset.theme).toBe('light');
	});

	it('clears the mark for dark, which is the tokens with nothing set', () => {
		root.dataset.theme = 'light';
		applyAppearance('dark');
		expect(root.dataset.theme).toBeUndefined();
	});

	it("paints the browser's bar the mode's ground", () => {
		applyAppearance('light');
		expect(meta.content).toBe(THEME_COLOR.light);
		applyAppearance('dark');
		expect(meta.content).toBe(THEME_COLOR.dark);
	});

	it('draws a document with no theme-color meta, as the workbench is', () => {
		meta.remove();
		applyAppearance('light');
		expect(root.dataset.theme).toBe('light');
	});
});

describe('THEME_COLOR', () => {
	it("is each mode's ground in theme.css", () => {
		const css = readFileSync(resolve(HERE, '../theme.css'), 'utf8');
		const light = css.slice(css.indexOf(":root[data-theme='light']"));

		expect(css.slice(0, css.indexOf(":root[data-theme='light']"))).toContain(
			`--color-ui-bg: ${THEME_COLOR.dark};`
		);
		expect(light).toContain(`--color-ui-bg: ${THEME_COLOR.light};`);
	});
});

describe('readAppearance', () => {
	it('reads what the root is drawn in', () => {
		expect(readAppearance()).toBe('dark');
		root.dataset.theme = 'light';
		expect(readAppearance()).toBe('light');
	});
});

describe('useAppearance', () => {
	it('starts from what the root is drawn in', () => {
		root.dataset.theme = 'light';
		const { result } = renderHook(() => useAppearance());
		expect(result.current.value).toBe('light');
	});

	it('choosing redraws the root and keeps the choice on the device', () => {
		const { result } = renderHook(() => useAppearance());

		act(() => {
			result.current.choose('light');
		});
		expect(result.current.value).toBe('light');
		expect(root.dataset.theme).toBe('light');
		expect(localStorage.getItem(APPEARANCE_KEY)).toBe('light');

		act(() => {
			result.current.choose('dark');
		});
		expect(root.dataset.theme).toBeUndefined();
		expect(localStorage.getItem(APPEARANCE_KEY)).toBe('dark');
	});

	it('still redraws when storage refuses the write', () => {
		vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
			throw new Error('full');
		});
		const { result } = renderHook(() => useAppearance());

		act(() => {
			result.current.choose('light');
		});
		expect(root.dataset.theme).toBe('light');
	});
});
