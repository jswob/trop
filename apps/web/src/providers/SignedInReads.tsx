import { IsRestoringProvider } from '@tanstack/react-query';
import { type ReactNode, useEffect } from 'react';

import { wakeServer } from '@/lib/api';

import { useAuthState } from './AuthProvider';

/**
 * **While nobody is signed in, the app's reads are held and the server is woken.**
 *
 * The frame stays mounted behind sign-in, so without this its
 * reads would go out without a token and each come back refused. Held, they stay `pending` and
 * are sent once, with the token, when sign-in succeeds. `IsRestoringProvider` is TanStack Query's
 * own switch for "do not fetch yet" — made for a cache being restored from storage — and it holds
 * every `useQuery` under it, so a read added to the frame later is held without asking for it.
 *
 * Those refused reads did one useful thing: they started a server that had gone to sleep, while the
 * password was being typed. `wakeServer` now does that on purpose. A browser already signed in
 * needs no wake: its reads go out at once and are the wake.
 */
export function SignedInReads({ children }: { children: ReactNode }) {
	const { isAuthenticated } = useAuthState();

	useEffect(() => {
		if (!isAuthenticated) wakeServer();
	}, [isAuthenticated]);

	return <IsRestoringProvider value={!isAuthenticated}>{children}</IsRestoringProvider>;
}
