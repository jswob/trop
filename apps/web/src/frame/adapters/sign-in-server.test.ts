import { describe, expect, it, vi } from 'vitest';

import { apiSignIn } from './sign-in-server';

describe('apiSignIn', () => {
	it('signs in on the API’s login and says so', async () => {
		const login = vi.fn(() => Promise.resolve());

		expect(await apiSignIn(login)('hunter2')).toBe('ok');
		expect(login).toHaveBeenCalledWith('hunter2');
	});

	it('reads the API’s refusal as a wrong password', async () => {
		const server = apiSignIn(() => Promise.reject(new Error('Invalid password')));

		expect(await server('nope')).toBe('wrong');
	});

	it('reads a server with no password set up as not set up', async () => {
		const server = apiSignIn(() => Promise.reject(new Error('Authentication not configured')));

		expect(await server('any')).toBe('not-set-up');
	});

	it('passes any other failure on, rather than calling the password wrong', async () => {
		const server = apiSignIn(() => Promise.reject(new Error('Failed to fetch')));

		await expect(server('any')).rejects.toThrow('Failed to fetch');
	});
});
