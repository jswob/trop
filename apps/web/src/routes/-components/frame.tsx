import type { SplitControl, SplitWhere } from '@app/ui/kit';
import { GROUND, RESTING, destPath, hrefNest, nestPath } from '@app/ui/views/destinations';
import { Frame, createFrameDoor } from '@app/ui/views/frame';
import type { NestControl } from '@app/ui/views/nest';
import { Outlet, useNavigate, useRouter, useRouterState } from '@tanstack/react-router';
import { useEffect, useMemo, useState } from 'react';

import { apiSignIn } from '@/frame/adapters/sign-in-server';
import { useReadFailure } from '@/frame/adapters/use-read-failure';
import { DeclareProvider, createDeclareStore, useDeclared } from '@/frame/declare';
import { APP_DESTINATIONS, destOfPath, pageDeclaration } from '@/frame/pages';
import { useAuthActions, useAuthState } from '@/providers/AuthProvider';

import { GROUND_READS, useGround } from './ground';

/**
 * **The frame, on the app's records**, drawn by `__root` around every route. The app draws its own
 * sign-in in place of the frame, so nothing is sent to a login page. It stays mounted while the
 * pages under it change, so the split's moves run between pages as they do in the workbench.
 *
 * Where the frame is lives in two places, split by what survives a reload: the open destination is
 * the address (`/<dest>`), so a link, a reload and the browser's back agree with the rail;
 * whether the ground is collapsed and the register raised are this layout's state. The session is
 * the app's token — `Sign out` and a 401 both land on sign-in, drawn in place.
 *
 * A page declares what it draws beside its body — its nest, hold, errand and facts — through
 * `useDeclare`, and it is merged into `draw` here for that page's destination only. A record the
 * page opens inside itself is the address too: `/<dest>/<kind>/<id>`.
 *
 * The ground — its page, the cells' facts, the slug's reading — comes from `useGround`
 * (`./ground`); everything else here is the frame's wiring.
 */
export function AppFrame() {
	const pathname = useRouterState({ select: state => state.location.pathname });
	const navigate = useNavigate();
	const { isAuthenticated } = useAuthState();
	const { login, logout } = useAuthActions();
	const [ledger, setLedger] = useState(true);
	const [menu, setMenu] = useState(false);
	const [store] = useState(createDeclareStore);
	const readFailed = useReadFailure(GROUND_READS);
	usePreloadedDestinations();

	const dest = destOfPath(pathname);
	const declared = useDeclared(store, dest);
	const [door] = useState(createFrameDoor);
	const ground = useGround();
	const where: SplitWhere = { ledger, dest, menu };
	const control: SplitControl = {
		where,
		onWhere: next => {
			setLedger(next.ledger);
			setMenu(next.menu);
			if (next.dest === dest) return;
			void navigate({ to: destPath(next.dest) });
		},
	};

	const server = useMemo(() => {
		const answer = apiSignIn(login);
		return () => answer;
	}, [login]);

	const page = pageDeclaration(dest, ground.facts, ground.reading);

	const nest = declared.nest;
	const addressed = hrefNest(pathname);
	const nestControl: NestControl | undefined =
		nest === undefined || dest === null
			? undefined
			: {
					open: addressed?.dest === dest && addressed.kind === nest.kind ? addressed.id : null,
					onOpen: id => {
						void navigate({ to: id === null ? destPath(dest) : nestPath(dest, nest.kind, id) });
					},
				};

	return (
		<DeclareProvider store={store}>
			<Frame
				place={{ id: 'app', dest }}
				destinations={APP_DESTINATIONS}
				home={GROUND}
				resting={RESTING}
				draw={{
					page: <Outlet />,
					head: declared.head,
					reading: declared.reading ?? page.reading,
					errand: declared.errand ?? page.errand,
					facts: declared.facts,
					nest,
					held: declared.held,
					overlay: declared.overlay,
					centred: declared.centred,
					detour: declared.detour,
				}}
				ledger={ground.ledger}
				facts={ground.facts}
				groundReading={ground.reading}
				server={server}
				control={control}
				nestControl={nestControl}
				session={{ signedIn: isAuthenticated, signOut: logout }}
				readFailed={readFailed}
				door={door}
			/>
		</DeclareProvider>
	);
}

/**
 * **Every destination's code is fetched before it is pressed.** Routes are split per file, and a rail
 * cell is a button, not a link, so the router's own preload on intent never runs for one: the first
 * press on each destination would fetch and evaluate its chunk between the press and the move
 * (measured at 380ms for a large page in dev). Fetched once the frame is idle, the press finds it
 * there.
 */
function usePreloadedDestinations() {
	const router = useRouter();

	useEffect(() => {
		const preload = () => {
			for (const dest of APP_DESTINATIONS)
				void router.preloadRoute({ to: destPath(dest.id) }).catch(() => undefined);
		};
		const timer = setTimeout(preload, 1000);
		return () => {
			clearTimeout(timer);
		};
	}, [router]);
}
