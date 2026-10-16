import { describe, expect, it } from 'vitest';

import { addKitExport } from './promote';

const INDEX = `/** The kit. */
export { BoardShell } from './BoardShell';
export { Button, IconButton } from './Button';
export { Field } from './Field';
export { Waiting } from './Waiting';
`;

describe('addKitExport', () => {
	it('inserts in the sorted position the file already keeps', () => {
		const next = addKitExport(INDEX, 'Disclose', ['Disclose'], []);
		const lines = next.trim().split('\n');

		expect(lines[3]).toBe(`export { Disclose } from './Disclose';`);
	});

	it('sorts case-insensitively, the way the existing list does', () => {
		const next = addKitExport(INDEX, 'control', ['CONTROL_H'], []);
		const lines = next.trim().split('\n');

		expect(lines.indexOf(`export { CONTROL_H } from './control';`)).toBe(3);
	});

	it('writes the type export on its own line beneath the value export', () => {
		const next = addKitExport(INDEX, 'Sheet', ['Sheet'], ['SheetDay']);

		expect(next).toContain(
			`export { Sheet } from './Sheet';\nexport type { SheetDay } from './Sheet';`
		);
	});

	it('goes first when it sorts before everything', () => {
		const next = addKitExport(INDEX, 'Banner', ['Banner'], []);
		const lines = next.trim().split('\n');

		expect(lines[1]).toBe(`export { Banner } from './Banner';`);
	});

	it('goes last when it sorts after everything', () => {
		expect(addKitExport(INDEX, 'Zone', ['Zone'], []).trim().split('\n').at(-1)).toBe(
			`export { Zone } from './Zone';`
		);
	});

	/**
	 * A promotion is usually run twice: the first run stops on rule 8's caller count and the
	 * second one goes through. A second export line for one module is a compile error, so the
	 * re-run has to be a no-op rather than an append.
	 */
	it('is idempotent', () => {
		const once = addKitExport(INDEX, 'Disclose', ['Disclose'], []);

		expect(addKitExport(once, 'Disclose', ['Disclose'], [])).toBe(once);
	});

	it('writes nothing for a module that exports nothing', () => {
		expect(addKitExport(INDEX, 'Empty', [], [])).toBe(INDEX);
	});
});
