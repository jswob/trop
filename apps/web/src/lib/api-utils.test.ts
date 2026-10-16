import { describe, expect, it } from 'vitest';

import { ApiError, extractErrorMessage } from './api-utils';

describe('ApiError', () => {
	it('sets message, status, and details', () => {
		const details = { rules: { count: 2, ruleIds: ['r1', 'r2'] } };
		const err = new ApiError('Conflict', 409, details);

		expect(err.message).toBe('Conflict');
		expect(err.status).toBe(409);
		expect(err.details).toEqual(details);
		expect(err.name).toBe('ApiError');
	});

	it('is an instance of Error', () => {
		const err = new ApiError('fail');

		expect(err).toBeInstanceOf(Error);
	});

	it('status and details are optional', () => {
		const err = new ApiError('fail');

		expect(err.status).toBeUndefined();
		expect(err.details).toBeUndefined();
	});
});

describe('extractErrorMessage', () => {
	it('extracts message from Error instance', () => {
		expect(extractErrorMessage(new Error('Something failed'))).toBe('Something failed');
	});

	it('returns string errors as-is', () => {
		expect(extractErrorMessage('direct error')).toBe('direct error');
	});

	it('extracts message from object with message property', () => {
		expect(extractErrorMessage({ message: 'object error' })).toBe('object error');
	});

	it('extracts error from object with error property (API shape)', () => {
		expect(extractErrorMessage({ error: 'Task not found' })).toBe('Task not found');
	});

	it('prefers message over error when both present', () => {
		expect(extractErrorMessage({ message: 'msg', error: 'err' })).toBe('msg');
	});

	it('returns fallback for unknown types', () => {
		expect(extractErrorMessage(42)).toBe('An unknown error occurred');
		expect(extractErrorMessage(null)).toBe('An unknown error occurred');
		expect(extractErrorMessage(undefined)).toBe('An unknown error occurred');
		expect(extractErrorMessage({})).toBe('An unknown error occurred');
	});
});
