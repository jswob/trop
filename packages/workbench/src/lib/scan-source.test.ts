import { describe, expect, it } from 'vitest';

import { matchBracket, objectEntries } from './scan-source';

describe('matchBracket', () => {
	it('matches a simple brace pair', () => {
		const source = '{ a }';

		expect(matchBracket(source, 0)).toBe(source.length);
	});

	it('matches through nesting', () => {
		const source = 'x = { a: { b: 1 }, c: 2 };';

		expect(source.slice(4, matchBracket(source, 4))).toBe('{ a: { b: 1 }, c: 2 }');
	});

	it('ignores braces inside string literals', () => {
		const source = "{ a: 'not } a brace' }";

		expect(matchBracket(source, 0)).toBe(source.length);
	});

	it('ignores braces inside comments', () => {
		const source = '{ /* } */ a: 1 }';

		expect(matchBracket(source, 0)).toBe(source.length);
	});

	it('returns -1 when the bracket never closes', () => {
		expect(matchBracket('{ a: 1', 0)).toBe(-1);
	});

	it('returns -1 when the index is not an opening bracket', () => {
		expect(matchBracket('abc', 1)).toBe(-1);
	});
});

describe('objectEntries', () => {
	it('splits top-level keys and values', () => {
		expect(objectEntries("{ variant: 'primary', size: 'md' }")).toEqual([
			['variant', "'primary'"],
			['size', "'md'"],
		]);
	});

	it('keeps nested objects intact in the value', () => {
		expect(objectEntries('{ variants: { size: { sm: 1, md: 2 } } }')).toEqual([
			['variants', '{ size: { sm: 1, md: 2 } }'],
		]);
	});

	it('unquotes quoted keys', () => {
		expect(objectEntries("{ '2xl': 'max-w-2xl' }")).toEqual([['2xl', "'max-w-2xl'"]]);
	});

	it('handles boolean-shaped keys', () => {
		expect(objectEntries("{ true: 'on', false: 'off' }")).toEqual([
			['true', "'on'"],
			['false', "'off'"],
		]);
	});

	it('tolerates trailing commas and comments', () => {
		expect(objectEntries('{\n\t// a note\n\tsize: 1,\n}')).toEqual([['size', '1']]);
	});

	it('ignores commas inside strings and arrays', () => {
		expect(objectEntries("{ a: 'x, y', b: [1, 2] }")).toEqual([
			['a', "'x, y'"],
			['b', '[1, 2]'],
		]);
	});

	it('returns nothing for an empty object', () => {
		expect(objectEntries('{}')).toEqual([]);
	});
});
