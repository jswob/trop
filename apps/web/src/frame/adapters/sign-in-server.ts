import type { SignInAnswer, SignInServer } from '@app/ui/views/sign-in';

/** The API's own words for the two refusals sign-in draws (`apps/api/src/modules/auth`). */
const REFUSALS = new Map<string, SignInAnswer>([
	['Invalid password', 'wrong'],
	['Authentication not configured', 'not-set-up'],
]);

/**
 * **Sign-in's server, answered by the API** — `useAuthActions().login`, which stores the token
 * when it succeeds. A 401 is a wrong password and the unset server's 500 is `not-set-up`; any
 * other failure is passed on, and sign-in says the server was not reached — a network
 * error is not a wrong password.
 */
export function apiSignIn(login: (password: string) => Promise<void>): SignInServer {
	return async password => {
		try {
			await login(password);
			return 'ok';
		} catch (error) {
			const answer = error instanceof Error ? REFUSALS.get(error.message) : undefined;
			if (answer === undefined) throw error;
			return answer;
		}
	};
}
