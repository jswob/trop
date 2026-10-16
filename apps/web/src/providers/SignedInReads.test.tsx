import { QueryClient, QueryClientProvider, useQuery } from '@tanstack/react-query';
import { act, cleanup, render, waitFor } from '@testing-library/react';
import { useEffect } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { client, wakeServer } from '@/lib/api';
import { tokenStorage } from '@/lib/token-storage';

import { AuthProvider, useAuthActions } from './AuthProvider';
import { SignedInReads } from './SignedInReads';

vi.mock('@/lib/api', () => ({
	client: { api: { auth: { login: { post: vi.fn() } } } },
	setOnUnauthorized: vi.fn(),
	wakeServer: vi.fn(),
}));

let actions: ReturnType<typeof useAuthActions> | null = null;
let read: { data: string | undefined; pending: boolean } | null = null;

function Probe() {
	const nextActions = useAuthActions();
	useEffect(() => {
		actions = nextActions;
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

function mount(ask: () => Promise<string>) {
	const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
	render(
		<QueryClientProvider client={queryClient}>
			<AuthProvider>
				<SignedInReads>
					<Probe />
					<Reader ask={ask} />
				</SignedInReads>
			</AuthProvider>
		</QueryClientProvider>
	);
	if (actions === null) throw new Error('the probe did not render');
	return actions;
}

/** Long enough for a read that was going to be sent to have been. */
const settle = () => act(() => new Promise<void>(resolve => setTimeout(resolve, 20)));

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

describe('SignedInReads', () => {
	it('holds a read while nobody is signed in', async () => {
		const ask = vi.fn<() => Promise<string>>().mockResolvedValue('the notes');
		mount(ask);
		await settle();

		expect(ask).not.toHaveBeenCalled();
		expect(read).toEqual({ data: undefined, pending: true });
	});

	it('sends a held read when sign-in succeeds, and sends it once', async () => {
		const ask = vi.fn<() => Promise<string>>().mockResolvedValue('the notes');
		const { login } = mount(ask);
		await settle();

		await act(() => login('hunter2'));

		await waitFor(() => expect(read?.data).toBe('the notes'));
		await settle();
		expect(ask).toHaveBeenCalledTimes(1);
	});

	it('sends a read at once in a browser already signed in', async () => {
		tokenStorage.setToken('jwt');
		const ask = vi.fn<() => Promise<string>>().mockResolvedValue('the notes');
		mount(ask);

		await waitFor(() => expect(read?.data).toBe('the notes'));
		expect(ask).toHaveBeenCalledTimes(1);
	});

	it('asks afresh on the next sign-in, and nothing in between', async () => {
		tokenStorage.setToken('jwt');
		const ask = vi
			.fn<() => Promise<string>>()
			.mockResolvedValueOnce('the notes')
			.mockResolvedValue('the notes, since');
		const { login, logout } = mount(ask);
		await waitFor(() => expect(read?.data).toBe('the notes'));

		act(() => logout());
		await settle();
		expect(ask).toHaveBeenCalledTimes(1);

		await act(() => login('hunter2'));

		await waitFor(() => expect(read?.data).toBe('the notes, since'));
		expect(ask).toHaveBeenCalledTimes(2);
	});

	it('wakes the server while nobody is signed in', async () => {
		mount(() => Promise.resolve('the notes'));
		await settle();

		expect(wakeServer).toHaveBeenCalledTimes(1);
	});

	it('leaves the server be in a browser already signed in, whose reads wake it', async () => {
		tokenStorage.setToken('jwt');
		mount(() => Promise.resolve('the notes'));
		await settle();

		expect(wakeServer).not.toHaveBeenCalled();
	});
});
