import { describe, expect, it } from 'vitest';

import { splitFrames, splitVariants } from './stage-search';

describe('splitVariants', () => {
	it('returns undefined when the URL names none', () => {
		expect(splitVariants(undefined)).toBeUndefined();
	});

	it('splits a comma-separated list', () => {
		expect(splitVariants('a,b')).toEqual(['a', 'b']);
	});

	it('drops the empties a trailing comma leaves', () => {
		expect(splitVariants('a,,b,')).toEqual(['a', 'b']);
		expect(splitVariants(',')).toBeUndefined();
	});

	it('reads a repeated search param, which arrives as an array', () => {
		expect(splitVariants(['a', 'b'])).toEqual(['a', 'b']);
		expect(splitVariants(['a,b', 'c'])).toEqual(['a', 'b', 'c']);
		expect(splitVariants([])).toBeUndefined();
	});

	it('ignores a value of any other shape rather than throwing', () => {
		expect(splitVariants(7)).toBeUndefined();
		expect(splitVariants(null)).toBeUndefined();
	});
});

describe('splitFrames', () => {
	it('reads subject:variant pairs', () => {
		expect(splitFrames('timeline:populated,timeline-v2-call-sheet:populated')).toEqual([
			{ subject: 'timeline', variant: 'populated' },
			{ subject: 'timeline-v2-call-sheet', variant: 'populated' },
		]);
	});

	it('leaves the variant off when none is named', () => {
		expect(splitFrames('timeline')).toEqual([{ subject: 'timeline' }]);
	});

	it('drops empties and entries with no subject', () => {
		expect(splitFrames('timeline:populated,,:orphan,')).toEqual([
			{ subject: 'timeline', variant: 'populated' },
		]);
	});

	it('reads a repeated param, and returns undefined for nothing usable', () => {
		expect(splitFrames(['timeline', 'login:default'])).toEqual([
			{ subject: 'timeline' },
			{ subject: 'login', variant: 'default' },
		]);
		expect(splitFrames(undefined)).toBeUndefined();
		expect(splitFrames('')).toBeUndefined();
	});
});
