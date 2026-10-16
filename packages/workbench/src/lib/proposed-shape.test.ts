import { describe, expect, it } from 'vitest';

import { parseProposedShape } from './proposed-shape';

describe('parseProposedShape', () => {
	it('returns null when the file declares no shape', () => {
		expect(parseProposedShape('export const x = 1;\n')).toBeNull();
	});

	it('reads an object type alias including its nested braces', () => {
		const source = `const before = 1;

export type PROPOSED_SHAPE = {
	id: string;
	slots: { start: string; end: string }[];
};

const after = 2;
`;

		expect(parseProposedShape(source)).toBe(
			`export type PROPOSED_SHAPE = {
	id: string;
	slots: { start: string; end: string }[];
};`
		);
	});

	it('reads a union type alias that spans several lines', () => {
		const source = `export type PROPOSED_SHAPE =
	| { kind: 'draft' }
	| { kind: 'published'; at: string };
`;

		expect(parseProposedShape(source)).toBe(
			`export type PROPOSED_SHAPE =
	| { kind: 'draft' }
	| { kind: 'published'; at: string };`
		);
	});

	it('reads an interface declaration that has no trailing semicolon', () => {
		const source = `export interface PROPOSED_SHAPE {
	id: string;
}

export function Screen() {
	return null;
}
`;

		expect(parseProposedShape(source)).toBe(`export interface PROPOSED_SHAPE {
	id: string;
}`);
	});

	it('ignores a semicolon inside a string literal type', () => {
		const source = `export type PROPOSED_SHAPE = { separator: ';'; id: string };\n`;

		expect(parseProposedShape(source)).toBe(
			`export type PROPOSED_SHAPE = { separator: ';'; id: string };`
		);
	});

	it('ignores a declaration that only mentions the name', () => {
		expect(parseProposedShape('// PROPOSED_SHAPE lives in the sibling file\n')).toBeNull();
	});
});
