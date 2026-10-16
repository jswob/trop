import type { SignedOut } from '@app/ui/views/frame-types';
import type { SignInServer } from '@app/ui/views/sign-in';

/** The password the workbench's stand-in server takes. Anything else is a wrong password. */
export const WORKBENCH_PASSWORD = 'workbench';

/** How long the stand-in takes to answer — long enough to see `Signing in`. */
const ANSWER_MS = 700;

/**
 * **The server the workbench signs in against**, read off the condition the frame opened in: one
 * that answers takes {@link WORKBENCH_PASSWORD} and refuses anything else, one that is still
 * answering never does, and one nobody set a password on refuses every press with the fix.
 *
 * The app signs in against the API instead, so nothing invented here ships.
 */
export function standInServer(opens: SignedOut): SignInServer {
	if (opens === 'signing-in') return () => new Promise(() => undefined);
	return password =>
		new Promise(resolve => {
			window.setTimeout(() => {
				if (opens === 'not-set-up') resolve('not-set-up');
				else resolve(password === WORKBENCH_PASSWORD ? 'ok' : 'wrong');
			}, ANSWER_MS);
		});
}
