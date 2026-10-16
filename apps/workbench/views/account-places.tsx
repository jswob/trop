import { AccountPage, SIGN_OUT } from '@app/ui/views/account';
import { useAppearance } from '@app/ui/views/appearance';

import type { Place, SignedOut, ViewDraw } from './types';

/**
 * **Account** — the rail's last cell: the device's appearance, then `Sign out`.
 *
 * The app has one shared password and no user record, so there is nobody to name and no expiry
 * to report. The one setting is how this device draws the app, and it lives on the device.
 * `Sign out` is the page's errand: the ink plate under the head from `lg`, one cell on the slug
 * below it, where the page draws its head and the switch.
 *
 * **It never asks.** Every way onto this page already asked while anything was held, so by the
 * time `Sign out` is on screen nothing is, and the press loses nothing.
 *
 * Signed out is a state of the app rather than of a hand, so each condition sign-in opens in is
 * listed, the empty one among them although `Sign out` reaches it.
 */

/** The page with its switch live: a press redraws the frame and keeps the choice. */
function AccountPlace() {
	return <AccountPage appearance={useAppearance()} />;
}

function draw(signedOut?: SignedOut): ViewDraw {
	return {
		page: <AccountPlace />,
		reading: { place: 'Account', fact: '' },
		errand: SIGN_OUT,
		signedOut,
	};
}

export const account: Place = {
	id: 'account',
	dest: 'account',
	title: 'Account, and sign-in',
	description: 'The device’s appearance and Sign out; signed out, the one centred screen.',
	variants: [
		{
			id: 'account',
			label: 'Signed in',
			note: 'The page at rest: head, appearance switch, Sign out.',
			draw: () => draw(),
		},
		{
			id: 'signed-out',
			label: 'Signed out',
			note: 'Sign-in, empty: the app’s other mode.',
			draw: () => draw('empty'),
		},
		{
			id: 'signing-in',
			label: 'Signing in',
			note: 'The password sent and the server not answered yet.',
			draw: () => draw('signing-in'),
		},
		{
			id: 'not-set-up',
			label: 'Not set up',
			note: 'A server with no password configured.',
			draw: () => draw('not-set-up'),
		},
		{
			id: 'unreachable',
			label: 'Unreachable',
			note: 'The password sent and no server answered.',
			draw: () => draw('unreachable'),
		},
	],
};

export const ACCOUNT_PLACES: Place[] = [account];
