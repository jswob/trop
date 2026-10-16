import { describe, expect, it } from 'vitest';

import { formatTried, parseTried, readTokens, triedFor, usedBy } from './tokens';

const CSS = `
/* A comment that says @theme { and must not be read. */
@theme {
	--color-bg: #101419;
	--color-ink-2: #9aa3b2;
	--color-row: rgba(255, 255, 255, 0.04);
	--font-sans: 'Archivo', system-ui, sans-serif;
	--text-body: 13px;
	--spacing-row: 28px;
	--container-sheet: 28rem;
}

:root {
	--scheme: dark;
}

:root[data-theme='light'] {
	--scheme: light;
	--color-bg: #f5f7f9;
	--color-ink-2: #465362;
}
`;

describe('readTokens', () => {
	const tokens = readTokens(CSS);

	it('reads every token the theme declares, in the order it declares them', () => {
		expect(tokens.map(token => token.name)).toEqual([
			'--color-bg',
			'--color-ink-2',
			'--color-row',
			'--font-sans',
			'--text-body',
			'--spacing-row',
			'--container-sheet',
		]);
	});

	it('sorts each into a kind by what it is declared as', () => {
		expect(tokens.map(token => token.kind)).toEqual([
			'colour',
			'colour',
			'colour',
			'face',
			'type',
			'space',
			'space',
		]);
	});

	it('pairs a token with its value in the light appearance, where it has one', () => {
		expect(tokens[1]).toMatchObject({ key: 'ink-2', dark: '#9aa3b2', light: '#465362' });
		expect(tokens[2]).toMatchObject({ key: 'row', dark: 'rgba(255, 255, 255, 0.04)' });
		expect(tokens[2]?.light).toBeUndefined();
	});
});

describe('usedBy', () => {
	const tokens = readTokens(CSS);
	const sources = {
		'kit/A.tsx': 'const a = "text-ink-2 bg-row h-row";',
		'kit/B.tsx': '/* text-ink-2 in prose is not a use */ const b = "border-ink-2/40";',
		'kit/C.tsx': 'const c = "text-ink-20 var(--color-bg)";',
		'kit/D.css': '.d { height: var(--spacing-row); font-size: var(--text-body); }',
		'kit/E.tsx': 'const e = "@container/sheet @sheet:grid";',
		'kit/F.tsx': 'const f = "@max-sheet:hidden @sheets:flex";',
	};

	it('counts the files whose code uses a token, by its utility or its variable', () => {
		const used = usedBy(tokens, sources);

		expect(used['--color-ink-2']).toEqual(['kit/A.tsx', 'kit/B.tsx']);
		expect(used['--color-bg']).toEqual(['kit/C.tsx']);
		expect(used['--text-body']).toEqual(['kit/D.css']);
	});

	it('tells a colour from a length that share a name', () => {
		const used = usedBy(tokens, sources);

		expect(used['--color-row']).toEqual(['kit/A.tsx']);
		expect(used['--spacing-row']).toEqual(['kit/A.tsx', 'kit/D.css']);
	});

	it('counts a container width spent as a container variant', () => {
		expect(usedBy(tokens, sources)['--container-sheet']).toEqual(['kit/E.tsx', 'kit/F.tsx']);
	});
});

describe('tried values in the address', () => {
	it('reads a value tried for one appearance of one token', () => {
		expect(parseTried('color-accent.dark:%23b8860b,spacing-row:32px')).toEqual({
			'--color-accent.dark': '#b8860b',
			'--spacing-row': '32px',
		});
	});

	it('writes them back the way it reads them', () => {
		const tried = { '--color-accent.light': 'rgb(1 2 3 / 0.5)', '--text-body': '14px' };

		expect(parseTried(formatTried(tried))).toEqual(tried);
	});

	it('writes nothing when nothing is tried', () => {
		expect(formatTried({})).toBeUndefined();
		expect(parseTried(undefined)).toEqual({});
	});

	it('picks what applies to a frame drawn in one appearance', () => {
		const tried = {
			'--color-accent.dark': '#111111',
			'--color-accent.light': '#eeeeee',
			'--spacing-row': '32px',
		};

		expect(triedFor(tried, 'light')).toEqual({
			'--color-accent': '#eeeeee',
			'--spacing-row': '32px',
		});
		expect(triedFor(tried, 'dark')).toEqual({
			'--color-accent': '#111111',
			'--spacing-row': '32px',
		});
	});
});
