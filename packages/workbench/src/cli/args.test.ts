import { describe, expect, it } from 'vitest';

import { bool, flag, list, num, parseArgv } from './args';

describe('parseArgv', () => {
	it('takes the first bare token as the verb and keeps the rest in order', () => {
		const argv = parseArgv(['drive', 'order-modify:edit', 'extra']);

		expect(argv.verb).toBe('drive');
		expect(argv.positional).toEqual(['order-modify:edit', 'extra']);
	});

	it('reads --flag value and --flag=value the same way', () => {
		expect(flag(parseArgv(['x', '--width', 'mobile']), 'width')).toBe('mobile');
		expect(flag(parseArgv(['x', '--width=mobile']), 'width')).toBe('mobile');
	});

	it('treats a trailing flag as bare, and a bare flag as present without a value', () => {
		const argv = parseArgv(['x', '--list']);

		expect(bool(argv, 'list')).toBe(true);
		expect(flag(argv, 'list')).toBeUndefined();
	});

	/**
	 * `--scroll -400` is a scroll up. A parser that read the single minus as the next flag would
	 * drop the value in silence and scroll the default distance the other way.
	 */
	it('keeps a negative number as a value', () => {
		expect(flag(parseArgv(['drive', '--scroll', '-400']), 'scroll')).toBe('-400');
	});

	it('does not swallow the next flag as a value', () => {
		const argv = parseArgv(['x', '--list', '--width', 'mobile']);

		expect(bool(argv, 'list')).toBe(true);
		expect(flag(argv, 'width')).toBe('mobile');
	});

	it('splits a comma list and drops the blanks', () => {
		expect(list(parseArgv(['x', '--space', 'views, kit ,']), 'space')).toEqual(['views', 'kit']);
		expect(list(parseArgv(['x']), 'space')).toEqual([]);
	});

	it('refuses a numeric flag that is not a number', () => {
		expect(num(parseArgv(['x', '--n', '12']), 'n', 1)).toBe(12);
		expect(num(parseArgv(['x']), 'n', 7)).toBe(7);
		expect(() => num(parseArgv(['x', '--n', 'lots']), 'n', 1)).toThrow(/takes a number/);
	});

	it('keeps an equals sign inside a value', () => {
		expect(flag(parseArgv(['x', '--do=fill:Name=Ada']), 'do')).toBe('fill:Name=Ada');
	});
});
