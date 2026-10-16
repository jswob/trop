import { createRootRouteWithContext } from '@tanstack/react-router';

import { AuthProvider } from '@/providers/AuthProvider';
import { SignedInReads } from '@/providers/SignedInReads';
import type { RouterContext } from '@/router';

import { AppFrame } from './-components/frame';

/**
 * **The app, around every route.** The frame is mounted once here and stays mounted while the pages
 * under it change (`-components/frame`), inside `AuthProvider`. The app draws its own sign-in in
 * place of its frame, so nothing is sent to a login page; `SignedInReads` holds the frame's reads
 * until someone is signed in.
 */
export const Route = createRootRouteWithContext<RouterContext>()({
	component: RootLayout,
});

function RootLayout() {
	return (
		<AuthProvider>
			<SignedInReads>
				<AppFrame />
			</SignedInReads>
		</AuthProvider>
	);
}
