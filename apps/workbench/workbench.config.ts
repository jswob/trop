import { defineWorkbench } from '@jswob/design-workbench/config';

/**
 * The app, bound to the workbench: the terminal's half of the binding (`workbench.setup.tsx` is the
 * browser's).
 *
 * Every path is relative to this file, and the folder it stands in is the design root: the one
 * place a design round may write, with the settled spaces of the UI package (`packages/ui/src/`)
 * named below.
 */
export default defineWorkbench({
	name: 'App',
	port: 5175,
	app: { root: '.', vite: './vite.config.ts' },
	setup: './workbench.setup.tsx',
	styles: './index.css',
	/*
	 * The UI's faces: Archivo for everything, Geist Mono for figures that align. The web's
	 * `apps/web/index.html` loads the same link; keep the two identical.
	 */
	head: [
		{ tag: 'link', attrs: { rel: 'preconnect', href: 'https://fonts.googleapis.com' } },
		{
			tag: 'link',
			attrs: { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: true },
		},
		{
			tag: 'link',
			attrs: {
				href: 'https://fonts.googleapis.com/css2?family=Archivo:wght@400;500;600&family=Geist+Mono:wght@400;500&display=swap',
				rel: 'stylesheet',
			},
		},
	],
	theme: {
		css: '../../packages/ui/src/theme.css',
		ground: '--color-ui-bg',
		face: '--font-ui-sans',
		spentIn: [
			'../../packages/ui/src/**/*.{ts,tsx,css}',
			'../web/src/{frame,routes}/**/*.{ts,tsx,css}',
			'./{views,kit,kit-docs}/**/*.{ts,tsx}',
		],
	},
	/* Where settled code lives: the UI package the web and the workbench both draw. */
	settled: {
		root: '../../packages/ui/src',
		alias: '@app/ui',
		spaces: ['kit', 'views'],
		styles: ['theme.css', 'scroll.css', 'glaze.css', 'document.css'],
	},
	bans: {
		file: './BANS.md',
		/* The words a round uses for a subject, mapped to words in BANS.md's section titles. */
		subjects: {
			colour: ['hue'],
			color: ['hue'],
			status: ['hue'],
			shadow: ['depth'],
			radius: ['depth'],
			shape: ['depth'],
			font: ['type'],
			hierarchy: ['type'],
			control: ['density'],
			button: ['density'],
			field: ['density'],
			ledger: ['ground'],
			home: ['ground'],
			nav: ['navigation'],
			rail: ['navigation'],
			chrome: ['navigation'],
			frame: ['navigation', 'below'],
			mobile: ['below'],
			phone: ['below'],
			slug: ['below'],
			record: ['records'],
			form: ['records'],
			note: ['records'],
			notes: ['records'],
			page: ['spread'],
			column: ['spread'],
			animation: ['motion'],
			loading: ['waiting'],
			empty: ['waiting'],
			place: ['earns'],
			view: ['earns'],
			variant: ['earns'],
			state: ['earns'],
		},
		/* §11 Settled and §12 Pre-seeded bind every round. */
		always: ['11', '12'],
	},
	census: {
		states: ['./kit/types.ts', '../../packages/ui/src/kit/types.ts'],
		type: 'SlotState',
		hues: '--color-ui-',
	},
	gates: [
		{
			name: 'eslint',
			cwd: '../..',
			run: [
				'bunx',
				'eslint',
				'apps/workbench',
				'packages/ui',
				'--ext',
				'.ts,.tsx',
				'--report-unused-disable-directives',
				'--max-warnings',
				'0',
			],
		},
		{ name: 'types', run: ['bunx', 'tsc', '--noEmit'] },
		{ name: 'ui types', cwd: '../../packages/ui', run: ['bunx', 'tsc', '--noEmit'] },
		{ name: 'tests', run: ['bunx', 'vitest', 'run'] },
		{ name: 'ui tests', cwd: '../../packages/ui', run: ['bunx', 'vitest', 'run'] },
	],
	skill: '../../.claude/skills/design',
});
