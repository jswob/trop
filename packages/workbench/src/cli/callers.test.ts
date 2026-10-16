import { describe, expect, it } from 'vitest';

import { codeOf, exportedSymbols, usesOf } from './callers';

describe('codeOf', () => {
	/**
	 * **Prose is not a caller.** Naming a component in a doc comment is how these files explain
	 * themselves, and counting it claimed two callers for a component that had one — a lie in the
	 * one direction that matters, because rule 8's whole job is to stop a promotion on a single use.
	 */
	it('strips block and line comments before anything is counted', () => {
		const source = `/** Draws a Sheet. */\n// also a Sheet\nexport const x = 1;\n`;

		expect(codeOf(source)).not.toContain('Sheet');
		expect(codeOf(source)).toContain('export const x = 1;');
	});
});

describe('usesOf', () => {
	/**
	 * **A specimen is not a caller, and neither is the path to one.** A fixture that takes a
	 * component's scenes names the component in the path it imports them from (`kit/Sheet.specimen`)
	 * and draws nothing of it.
	 */
	it('blanks a path to a specimen, and keeps the names imported from it', () => {
		const source = `import { SHOOT } from './Sheet.specimen';\nconst page = await import('./Sheet.specimen');\n`;

		expect(usesOf(source)).not.toContain('Sheet');
		expect(usesOf(source)).toContain('import { SHOOT }');
	});

	it('leaves every other import as it is', () => {
		const source = `import { Sheet } from './Sheet';\n`;

		expect(usesOf(source)).toBe(source);
	});

	it('leaves the prose out too', () => {
		expect(usesOf(`/** Draws a Sheet. */\nexport const x = 1;\n`)).not.toContain('Sheet');
	});
});

describe('exportedSymbols', () => {
	it('separates values from types', () => {
		const source = `export function Sheet() {}\nexport const SHEET_W = 34;\nexport type SheetDay = { id: string };\n`;

		expect(exportedSymbols(source)).toEqual({
			values: ['SHEET_W', 'Sheet'],
			types: ['SheetDay'],
		});
	});

	it('leaves PROPOSED_SHAPE behind — it is the handoff note, not part of the contract', () => {
		const source = `export type PROPOSED_SHAPE = { a: 1 };\nexport function Thing() {}\n`;

		expect(exportedSymbols(source).types).toEqual([]);
		expect(exportedSymbols(source).values).toEqual(['Thing']);
	});

	it('does not count a symbol only a comment names', () => {
		expect(
			exportedSymbols(`/* export function Ghost() {} */\nexport const real = 1;\n`).values
		).toEqual(['real']);
	});

	it('counts a re-declared export once', () => {
		expect(exportedSymbols(`export const a = 1;\nexport const a = 2;\n`).values).toEqual(['a']);
	});
});
