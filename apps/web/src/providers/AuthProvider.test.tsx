import { QueryClient, QueryClientProvider, useQuery } from '@tanstack/react-query';
import { act, cleanup, render, waitFor } from '@testing-library/react';
import { useEffect } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { client, setOnUnauthorized } from '@/lib/api';
import { ApiError } from '@/lib/api-utils';
import { tokenStorage } from '@/lib/token-storage';

import { AuthProvider, useAuthActions, useAuthState } from './AuthProvider';

vi.mock('@/lib/api', () => ({
	client: { api: { auth: { login: { post: vi.fn() } } } },
	setOnUnauthorized: vi.fn(),
}));

let actions: ReturnType<typeof useAuthActions> | null = null;
let signedIn: boolean | null = null;
let read: { data: string | undefined; pending: boolean } | null = null;

function Probe() {
	const nextActions = useAuthActions();
	const { isAuthenticated } = useAuthState();
	useEffect(() => {
		actions = nextActions;
		signedIn = isAuthenticated;
	});
	return null;
}

/** A read the frame keeps on screen whoever is signed in, as it keeps the ground's. */
function Reader({ ask }: { ask: () => Promise<string> }) {
	const query = useQuery({ queryKey: ['notes'], queryFn: ask });
	useEffect(() => {
		read = { data: query.data, pending: query.isPending };
	});
	return null;
}

function mount(ask?: () => Promise<string>) {
	const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
	render(
		<QueryClientProvider client={queryClient}>
			<AuthProvider>
				<Probe />
				{ask === undefined ? null : <Reader ask={ask} />}
			</AuthProvider>
		</QueryClientProvider>
	);
	if (actions === null) throw new Error('the probe did not render');
	return actions;
}

beforeEach(() => {
	tokenStorage.clearToken();
	vi.mocked(client.api.auth.login.post).mockResolvedValue({
		data: { token: 'jwt' },
		error: null,
	} as never);
});

afterEach(() => {
	cleanup();
	vi.clearAllMocks();
});

/** Neither move navigates: the frame draws sign-in in place, and lands on its ground. */
describe('AuthProvider', () => {
	it('signs in and keeps the token', async () => {
		const { login } = mount();
		await act(() => login('hunter2'));

		expect(signedIn).toBe(true);
		expect(tokenStorage.getToken()).toBe('jwt');
	});

	it('asks every read again once signed in, so one refused before it is not left waiting', async () => {
		const ask = vi
			.fn<() => Promise<string>>()
			.mockRejectedValueOnce(new ApiError('Unauthorized', 401))
			.mockResolvedValue('the notes');
		const { login } = mount(ask);
		await waitFor(() => expect(read?.pending).toBe(false));
		expect(read?.data).toBeUndefined();

		await act(() => login('hunter2'));

		await waitFor(() => expect(read?.data).toBe('the notes'));
		expect(ask).toHaveBeenCalledTimes(2);
	});

	it('refuses a wrong password with the API’s words, and stays signed out', async () => {
		vi.mocked(client.api.auth.login.post).mockResolvedValue({
			data: null,
			error: { value: { message: 'Invalid password' } },
		} as never);
		const { login } = mount();

		await expect(login('nope')).rejects.toThrow('Invalid password');
		expect(signedIn).toBe(false);
	});

	it('signs out in place, and on a 401', () => {
		tokenStorage.setToken('jwt');
		const { logout } = mount();
		act(() => logout());

		expect(signedIn).toBe(false);
		expect(tokenStorage.getToken()).toBeNull();
		expect(setOnUnauthorized).toHaveBeenCalledWith(logout);
	});
});
