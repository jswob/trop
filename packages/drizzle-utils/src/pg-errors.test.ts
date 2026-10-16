import { describe, expect, test } from 'vitest';
import { isCheckViolation, isExclusionViolation, isPgError, isUniqueViolation } from './pg-errors';

describe('isPgError', () => {
	test('reads the code on the error postgres.js throws', () => {
		expect(isPgError({ code: '23505' }, '23505')).toBe(true);
	});

	test('reads the code on the cause Drizzle wraps it in', () => {
		expect(isPgError(new Error('Failed query', { cause: { code: '23505' } }), '23505')).toBe(true);
	});

	test('another code, or no code, is not a match', () => {
		expect(isPgError({ code: '23514' }, '23505')).toBe(false);
		expect(isPgError(new Error('plain'), '23505')).toBe(false);
		expect(isPgError(null, '23505')).toBe(false);
		expect(isPgError('23505', '23505')).toBe(false);
	});
});

test('each named violation is its own code', () => {
	expect(isUniqueViolation({ code: '23505' })).toBe(true);
	expect(isCheckViolation({ code: '23514' })).toBe(true);
	expect(isExclusionViolation({ cause: { code: '23P01' } })).toBe(true);
	expect(isUniqueViolation({ code: '23P01' })).toBe(false);
});
