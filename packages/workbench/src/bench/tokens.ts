import { matchBracket } from '../lib/scan-source';

export type TokenKind = 'colour' | 'type' | 'space' | 'face' | 'other';

export type Token = {
	/** The custom property, with its leading `--`. */
	name: string;
	/** What a class calls it: `ink-2`. */
	key: string;
	kind: TokenKind;
	/** The value the theme declares, which is the dark appearance's. */
	dark: string;
	/** The value the light appearance turns it to, where it does. */
	light?: string;
};

const KINDS: [TokenKind, RegExp][] = [
	['colour', /^--color-/],
	['type', /^--text-/],
	['space', /^--(spacing|container)-/],
	['face', /^--font-/],
];

const codeOf = (source: string): string =>
	source.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^[ \t]*\/\/.*$/gm, '');

function declarations(block: string): [string, string][] {
	return [...block.matchAll(/^\s*(--[\w-]+)\s*:\s*([^;]+);/gm)].map(match => [
		match[1] ?? '',
		(match[2] ?? '').trim(),
	]);
}

function blockAfter(source: string, opener: RegExp): string {
	const found = opener.exec(source);
	if (found === null) return '';

	const open = found.index + found[0].length - 1;
	const close = matchBracket(source, open);
	return close === -1 ? '' : source.slice(open + 1, close - 1);
}

/**
 * Every token a stylesheet's theme declares, each with its value in the light appearance.
 *
 * Read from the source, so the tokens page is never written: a token added to the theme is on the
 * page, and one taken out is gone from it.
 */
export function readTokens(css: string): Token[] {
	const source = css.replace(/\/\*[\s\S]*?\*\//g, '');
	const light = new Map(declarations(blockAfter(source, /\[data-theme=['"]light['"]\][^{]*\{/)));

	return declarations(blockAfter(source, /@theme[^\S\n]*\{/)).map(([name, value]) => {
		const kind = KINDS.find(([, pattern]) => pattern.test(name))?.[0] ?? 'other';
		const turned = light.get(name);

		return {
			name,
			key: name.replace(/^--[a-z]+-/, ''),
			kind,
			dark: value,
			...(turned === undefined ? {} : { light: turned }),
		};
	});
}

/** The utilities a token of each kind is spent through. A colour and a length may share a key. */
const UTILITIES: Record<TokenKind, string> = {
	colour:
		'text|bg|border(?:-[trblxy])?|outline|fill|stroke|ring|decoration|from|to|via|accent|caret|divide|placeholder',
	space:
		'[hw]|min-[hw]|max-[hw]|size|-?p[xytrbl]?|-?m[xytrbl]?|gap(?:-[xy])?|top|left|right|bottom|inset(?:-[xy])?|basis|leading|space-[xy]|translate-[xy]|scroll-[mp][xytrbl]?',
	type: 'text',
	face: 'font',
	other: '',
};

const escaped = (text: string): string => text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

/**
 * Which files spend each token, read off their code: by a utility of its kind, by its variable, or,
 * for a container width, by its container variant (`@sheet:grid`, `@max-sheet:hidden`).
 * Prose is not a use, so comments are stripped first, as they are when a component's callers are
 * counted.
 */
export function usedBy(
	tokens: readonly Token[],
	sources: Record<string, string>
): Record<string, string[]> {
	const files = Object.entries(sources).map(([path, source]) => [path, codeOf(source)] as const);

	return Object.fromEntries(
		tokens.map(token => {
			const utility = UTILITIES[token.kind];
			const pattern = new RegExp(
				[
					`var\\(\\s*${escaped(token.name)}(?![\\w-])`,
					...(utility === '' ? [] : [`(?<![\\w-])(?:${utility})-${escaped(token.key)}(?![\\w-])`]),
					...(token.name.startsWith('--container-')
						? [`(?<![\\w-])@(?:max-|min-)?${escaped(token.key)}(?![\\w-])`]
						: []),
				].join('|')
			);

			return [
				token.name,
				files
					.filter(([, code]) => pattern.test(code))
					.map(([path]) => path)
					.sort(),
			];
		})
	);
}

/** A value being tried, by token and appearance: `--color-accent.dark`. A length has no appearance. */
export type Tried = Record<string, string>;

/** The tried values the address carries: `color-accent.dark:%23b8860b,spacing-row:32px`. */
export function parseTried(value: unknown): Tried {
	if (typeof value !== 'string') return {};

	const tried: Tried = {};
	for (const pair of value.split(',')) {
		const at = pair.indexOf(':');
		if (at <= 0) continue;

		tried[`--${pair.slice(0, at)}`] = decodeURIComponent(pair.slice(at + 1));
	}

	return tried;
}

export function formatTried(tried: Tried): string | undefined {
	const pairs = Object.entries(tried).map(
		([key, value]) => `${key.replace(/^--/, '')}:${encodeURIComponent(value)}`
	);

	return pairs.length > 0 ? pairs.join(',') : undefined;
}

/** The tried values a frame drawn in one appearance takes, by custom property. */
export function triedFor(tried: Tried, appearance: 'dark' | 'light'): Record<string, string> {
	const applied: Record<string, string> = {};

	for (const [key, value] of Object.entries(tried)) {
		const [, name = key, of] = /^(.*)\.(dark|light)$/.exec(key) ?? [];
		if (of === undefined || of === appearance) applied[name] = value;
	}

	return applied;
}
