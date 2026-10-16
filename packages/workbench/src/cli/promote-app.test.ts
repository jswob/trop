import { describe, expect, it } from 'vitest';

import { barrelNames, importsOf, judgeImports, planAppMove, rewriteCaller } from './promote-app';

const NOTHING_MOVED = { alias: '@/ui', inApp: () => false, appKitExports: new Set<string>() };

describe('importsOf', () => {
	it('reads named, default, side-effect and re-export forms, code only', () => {
		const source = [
			`/** import { Ghost } from './Ghost'; */`,
			`import { useState } from 'react';`,
			`import type { SheetDay } from './sheet-shape';`,
			`import Thing from '../fixtures/thing';`,
			`import './side.css';`,
			`export { Mark } from './Mark';`,
			`import {`,
			`\tcountSeats,`,
			`\ttype Connection,`,
			`} from '../kit';`,
		].join('\n');

		expect(importsOf(source)).toEqual([
			{ specifier: 'react', names: ['useState'] },
			{ specifier: './sheet-shape', names: ['SheetDay'] },
			{ specifier: '../fixtures/thing', names: [] },
			{ specifier: './side.css', names: [] },
			{ specifier: './Mark', names: ['Mark'] },
			{ specifier: '../kit', names: ['countSeats', 'Connection'] },
		]);
	});
});

describe('barrelNames', () => {
	it('follows each barrel re-exported whole, as deep as it goes and once', () => {
		const siblings: Record<string, string> = {
			core: `export { Button } from './Button';\nexport * from './inner';`,
			inner: `export type { Tone } from './tone';\nexport * from './core';`,
			extra: `export { Meter, type MeterProps } from './Meter';`,
		};
		const index = [
			`/** export * from './ghost'; */`,
			`export * from './core';`,
			`export * from './extra';`,
			`export * from './missing';`,
			`export { Mark } from './Mark';`,
		].join('\n');

		expect([...barrelNames(index, name => siblings[name])].sort()).toEqual([
			'Button',
			'Mark',
			'Meter',
			'MeterProps',
			'Tone',
		]);
	});
});

describe('judgeImports', () => {
	it('keeps packages and the app alias as they are', () => {
		const verdicts = judgeImports(
			[
				{ specifier: 'react', names: [] },
				{ specifier: '@/lib/utils', names: [] },
			],
			'kit',
			NOTHING_MOVED
		);

		expect(verdicts).toEqual([
			{ specifier: 'react', to: 'react' },
			{ specifier: '@/lib/utils', to: '@/lib/utils' },
		]);
	});

	it('keeps a sibling that has already moved, and blocks one that has not', () => {
		const verdicts = judgeImports(
			[
				{ specifier: './SeatMark', names: [] },
				{ specifier: './Sheet', names: [] },
			],
			'kit',
			{ ...NOTHING_MOVED, inApp: (space, name) => space === 'kit' && name === 'SeatMark' }
		);

		expect(verdicts[0]).toEqual({ specifier: './SeatMark', to: './SeatMark' });
		expect(verdicts[1]).toMatchObject({ specifier: './Sheet' });
		expect(verdicts[1]).toHaveProperty('blocked');
	});

	it('points a view at the moved kit through the app alias', () => {
		const verdicts = judgeImports(
			[
				{ specifier: '../kit/Mark', names: ['Mark'] },
				{ specifier: '../kit', names: ['Mark', 'formatRange'] },
			],
			'views',
			{
				...NOTHING_MOVED,
				inApp: (space, name) => space === 'kit' && name === 'Mark',
				appKitExports: new Set(['Mark', 'formatRange']),
			}
		);

		expect(verdicts).toEqual([
			{ specifier: '../kit/Mark', to: '@/ui/kit/Mark' },
			{ specifier: '../kit', to: '@/ui/kit' },
		]);
	});

	it('names the kit symbols a view still needs from the workbench', () => {
		const [verdict] = judgeImports([{ specifier: '../kit', names: ['Mark', 'Sheet'] }], 'views', {
			...NOTHING_MOVED,
			appKitExports: new Set(['Mark']),
		});

		expect(verdict).toEqual({ specifier: '../kit', blocked: 'kit has not moved: Sheet' });
	});

	/** Fixtures, places, kit-docs and the workbench's own lib never follow a module out. */
	it('blocks everything else relative — that is the hand split', () => {
		const verdicts = judgeImports(
			[
				{ specifier: '../fixtures', names: [] },
				{ specifier: './timeline-places', names: [] },
				{ specifier: '../lib/plural', names: [] },
			],
			'views',
			NOTHING_MOVED
		);

		expect(verdicts.every(verdict => 'blocked' in verdict)).toBe(true);
	});
});

describe('planAppMove', () => {
	const MARK = `export function Mark() {\n\treturn null;\n}\n`;

	it('moves a leaf with its test when the test needs nothing from the workbench', () => {
		const plan = planAppMove(
			{ space: 'kit', name: 'Mark' },
			MARK,
			`import { Mark } from './Mark';\n`,
			NOTHING_MOVED
		);

		expect(plan.blocked).toEqual([]);
		expect(plan.source).toBe(MARK);
		expect(plan.test).toEqual({ moves: true, source: `import { Mark } from './Mark';\n` });
	});

	it('leaves a fixture-driven test in the workbench and says why', () => {
		const plan = planAppMove(
			{ space: 'kit', name: 'sheet-shape' },
			`export const x = 1;\n`,
			`import { CYCLE } from '../views/timeline-cycle';\nimport { x } from './sheet-shape';\n`,
			NOTHING_MOVED
		);

		expect(plan.blocked).toEqual([]);
		expect(plan.test).toMatchObject({ moves: false });
		expect(plan.test && !plan.test.moves && plan.test.because).toEqual([
			'../views/timeline-cycle: views/timeline-cycle has not moved',
		]);
	});

	it('refuses a module that imports what has not moved, listing each import', () => {
		const plan = planAppMove(
			{ space: 'views', name: 'sign-in' },
			`import { Mark } from '../kit';\nimport { SIGNED } from './sign-in-places';\n`,
			null,
			NOTHING_MOVED
		);

		expect(plan.blocked).toEqual([
			'../kit: kit has not moved: Mark',
			'./sign-in-places: views/sign-in-places has not moved',
		]);
	});

	it('rewrites the moved module’s own imports to where they resolve in the app', () => {
		const plan = planAppMove(
			{ space: 'views', name: 'sign-in' },
			`import { Mark } from '../kit';\n`,
			null,
			{ alias: '@/ui', inApp: () => true, appKitExports: new Set(['Mark']) }
		);

		expect(plan.source).toBe(`import { Mark } from '@/ui/kit';\n`);
	});
});

describe('rewriteCaller', () => {
	const MOVING = {
		space: 'kit',
		name: 'Mark',
		ext: '.tsx',
		alias: '@/ui',
		settled: 'src/ui',
	} as const;

	it('re-points a sibling import in the workbench kit', () => {
		expect(rewriteCaller(`import { Mark } from './Mark';\n`, 'kit/SplitShell.tsx', MOVING)).toBe(
			`import { Mark } from '@/ui/kit/Mark';\n`
		);
	});

	it('re-points the workbench barrel’s export line', () => {
		expect(rewriteCaller(`export { Mark } from './Mark';\n`, 'kit/index.ts', MOVING)).toBe(
			`export { Mark } from '@/ui/kit/Mark';\n`
		);
	});

	it('re-points a deep import from another space', () => {
		expect(rewriteCaller(`import { Mark } from '../kit/Mark';\n`, 'views/x.tsx', MOVING)).toBe(
			`import { Mark } from '@/ui/kit/Mark';\n`
		);
	});

	/** `kit-docs/index.ts` names each entry's files so the caller count can leave them out. */
	it('renames the file where the workbench names it as data', () => {
		expect(rewriteCaller(`files: ['kit/Mark.tsx'],\n`, 'kit-docs/index.ts', MOVING)).toBe(
			`files: ['src/ui/kit/Mark.tsx'],\n`
		);
	});

	it('leaves a namesake alone', () => {
		const source = `import { SeatMark } from './SeatMark';\nimport { Mark } from '../views/Mark';\n`;

		expect(rewriteCaller(source, 'kit/Sheet.tsx', MOVING)).toBe(source);
	});
});
