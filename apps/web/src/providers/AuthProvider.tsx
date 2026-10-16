import { useQueryClient } from '@tanstack/react-query';
import {
	createContext,
	type ReactNode,
	useCallback,
	useContext,
	useEffect,
	useMemo,
	useState,
} from 'react';

import { client, setOnUnauthorized } from '@/lib/api';
import { extractErrorMessage } from '@/lib/api-utils';
import { tokenStorage } from '@/lib/token-storage';

const AuthStateContext = createContext<{ isAuthenticated: boolean } | undefined>(undefined);

const AuthActionsContext = createContext<
	| {
			login: (password: string) => Promise<void>;
			logout: () => void;
	  }
	| undefined
>(undefined);

/**
 * Signs in and out, and says whether this browser is signed in.
 *
 * Neither move navigates: the frame draws its own sign-in in place of itself while signed out,
 * and lands on its own ground when signed in.
 *
 * Signing in asks every read again. The frame stays mounted behind sign-in, and a read of its that
 * was refused with a 401 — sent with a token that had run out — is never retried and never drawn as
 * a failed read; left alone it stays refused, and the sheet waits until something else happens to
 * ask. It also drops what the last session read. Reads are not sent at all while nobody is signed
 * in (`SignedInReads`).
 */
export function AuthProvider({ children }: { children: ReactNode }) {
	const queryClient = useQueryClient();
	const [isAuthenticated, setIsAuthenticated] = useState(tokenStorage.hasToken);

	const login = useCallback(
		async (password: string) => {
			const { data, error } = await client.api.auth.login.post({ password });
			if (error) throw new Error(extractErrorMessage(error.value));
			tokenStorage.setToken(data.token);
			setIsAuthenticated(true);
			void queryClient.resetQueries();
		},
		[queryClient]
	);

	const logout = useCallback(() => {
		tokenStorage.clearToken();
		setIsAuthenticated(false);
	}, []);

	useEffect(() => {
		setOnUnauthorized(logout);
	}, [logout]);

	const stateValue = useMemo(() => ({ isAuthenticated }), [isAuthenticated]);
	const actionsValue = useMemo(() => ({ login, logout }), [login, logout]);

	return (
		<AuthStateContext value={stateValue}>
			<AuthActionsContext value={actionsValue}>{children}</AuthActionsContext>
		</AuthStateContext>
	);
}

export function useAuthState() {
	const context = useContext(AuthStateContext);
	if (context === undefined) {
		throw new Error('useAuthState must be used within an AuthProvider');
	}
	return context;
}

export function useAuthActions() {
	const context = useContext(AuthActionsContext);
	if (context === undefined) {
		throw new Error('useAuthActions must be used within an AuthProvider');
	}
	return context;
}
