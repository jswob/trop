import { NestProvider, NestSwitch, useNest } from '@app/ui/views/nest';
import type { NestControl } from '@app/ui/views/nest';
import { ReadFailed } from '@app/ui/views/read-failed';
import { SignIn } from '@app/ui/views/sign-in';
import type { SignInServer } from '@app/ui/views/sign-in';
import {
	createContext,
	useContext,
	useLayoutEffect,
	useRef,
	useState,
	useSyncExternalStore,
} from 'react';
import type { ReactNode } from 'react';

import {
	AWAY_ID,
	Button,
	GROUND_ID,
	HERE_ID,
	Modal,
	PlaceholderRows,
	SplitShell,
	Waiting,
	WaitingNote,
	WaitFailed,
	useHeld,
} from '@app/ui/kit';
import type {
	DetourStatus,
	ErrandCell,
	HeldStatus,
	NestedPage,
	PageHeading,
	RailReading,
	SplitControl,
	SplitDest,
} from '@app/ui/kit';
import { headOf, hrefId } from '@app/ui/views/destinations';
import type {
	DetourRead,
	Hold,
	HoldSource,
	Live,
	PageErrand,
	SignedOut,
	ViewDraw,
} from '@app/ui/views/frame-types';

/**
 * The chrome, supplied once, to every place.
 *
 * **This is the only file under `views/` allowed to import `SplitShell`.** A place declares what it
 * puts on the page and in the ledger; everything around that — the split, the rail, the two ruled
 * scrolls, the page margins, the slug and the floor the slug needs below `lg` — is decided here and
 * is the same wherever you are. The workbench's `views/frame.test` asserts no place file ever
 * imports the shell, because a view that framed itself could disagree with every other view about
 * where you are, and that disagreement would not show up until both were on screen at once.
 *
 * **The cells are live.** `useHeld` is the frame's own settled behaviour — the logo collapses and
 * restores the ground, a destination's cell opens and closes it, the screen always keeps one half,
 * and a page holding something you would lose guards every way out of itself — so a place is a seed
 * rather than a still, and a state can be reached by using the thing as well as by picking it off
 * the variant bar. Which matters more here than it looks: a splitter that can only be looked at is a
 * splitter nobody has tried.
 *
 * **A hold is declared and never drawn.** A place says what it is holding (`ViewDraw.held`) and the
 * frame does the rest: the slug given over to the hold's row, the guard on the rail above `lg`, the
 * confirm, and the ledger drawn to be read rather than worked. A place able to draw its own status
 * row could disagree with the next one about what being mid-edit looks like.
 *
 * **A destination that is not this place is drawn undrawn.** A place draws only its own page, so
 * another cell pressed from it opens that destination's head over a waiting page, never a stand-in
 * page of its own. It has to open here, because what is being judged is the frame, so it gets its
 * own head from the rail (which is the whole point of a page head: it is there before the contents
 * are) and a waiting page under it. That makes an undrawn page and a loading page the same drawing,
 * and they are the same thing.
 */

/**
 * What a destination with no place draws: its name from the rail, and rows at the real pitch.
 *
 * It is drawn as a page that is *waiting* rather than as one that is blank, because those are the
 * same drawing and the honest one of the two is the loader — a destination with no place behind it
 * is the exact picture a real page presents while its contents are still out. The glaze and the note
 * are what say the app is working rather than stuck.
 */
function UndrawnPage() {
	return (
		<Waiting>
			<div className="flex items-center justify-end pb-1">
				<WaitingNote />
			</div>
			<PlaceholderRows rows={7} />
		</Waiting>
	);
}

/**
 * **The confirm a guarded press raises: two outcomes and no third.**
 *
 * It is `kit/Modal` drawn the way a destructive confirmation is drawn — the errand is this modal's
 * own, so it takes the ramp's `primary` for the thing a hand is most likely to want, and **red warns
 * in type** on the line that says what is lost: red is a warning in type, never a plate.
 *
 * *Stay* is the primary because it is the outcome that loses nothing. There is no third: no *save
 * and go*, and nothing that leaves the hold behind to be found later. The words are the hold's own
 * (`Hold.words`) and there is no default here, so the button that discards in place says what the
 * foot's own button says.
 *
 * It is drawn here rather than in `kit/RailShell` because it is a dialog about the caller's own
 * record, and the shell already takes `overlay`.
 */
function LeaveDialog({
	hold,
	going,
	onStay,
	onDiscard,
}: {
	hold: Hold;
	/** Where the press was heading, in words — `null` for a stop where it stands. */
	going: string | null;
	onStay: () => void;
	onDiscard: () => void;
}) {
	const words = hold.words;

	return (
		<Modal
			title={`${hold.name} — ${hold.fact}`}
			onClose={onStay}
			footer={
				<>
					<Button weight="normal" onClick={onDiscard}>
						{going === null ? words.stop : words.leave}
					</Button>
					<Button weight="primary" onClick={onStay}>
						{words.stay}
					</Button>
				</>
			}
		>
			<p className="flex items-start gap-1.5 text-ui-body leading-5 text-ui-red">
				<span aria-hidden>!</span>
				{words.loss(going)}
			</p>
		</Modal>
	);
}

/**
 * What a guarded press was heading for, in words the confirm can print. `null`: nowhere.
 * @param home - What the ground is called
 * @param find - The destination an id names, among the frame's own
 */
function goingName(
	going: string,
	away: string | null,
	home: string,
	find: (id: string) => SplitDest | null
): string | null {
	if (going === HERE_ID) return null;
	if (going === AWAY_ID) return away;
	if (going === GROUND_ID) return home;
	return find(going)?.label ?? home;
}

const DetourContext = createContext<(() => void) | null>(null);

/**
 * **How a page starts its detour** — the phone to the ground, the page kept open, the slug
 * the detour's row. `null` where the place declares none, so a page cannot offer a way onto the
 * ground that the frame has no row for.
 */
export function useDetour(): (() => void) | null {
	return useContext(DetourContext);
}

const PressContext = createContext<((cell: string) => void) | null>(null);

/**
 * **How a page presses one of its hold's cells** — a draft's `Save changes`, a run's `Stop
 * running`, from the page's own foot. It is the slug's press exactly: the same cell, the same
 * confirm where that cell asks, the same `onEnd` told the same id, so the two surfaces cannot
 * disagree about what the press does. `null` where nothing is held.
 *
 * Asking is a fact about the cell, not a flag on the hold, so a run's stop is a cell like any other.
 */
export function useHoldPress(): ((cell: string) => void) | null {
	return useContext(PressContext);
}

const SignOutContext = createContext<() => void>(() => undefined);

/**
 * An errand as the slug's cell: a link stays a link, and the frame's one press (`sign-out`)
 * becomes the frame's own handler, so the page never holds the power to sign anybody out.
 */
function errandCell(errand: PageErrand, signOut: () => void): ErrandCell {
	return 'href' in errand
		? { label: errand.label, icon: errand.icon, href: errand.href }
		: { label: errand.label, icon: errand.icon, onPress: signOut };
}

const NO_DETOUR = () => () => undefined;
const NO_READ = (): DetourRead | null => null;
const NO_HOLD = (): Hold | null => null;

function isSource(held: Hold | HoldSource | undefined): held is HoldSource {
	return held !== undefined && 'subscribe' in held;
}

function isLive<T extends object>(value: T | Live<T>): value is Live<T> {
	return 'subscribe' in value;
}

const NOT_CENTRED = () => false;

/**
 * **The session, where the caller keeps it** — the app's, whose sign-in is a token the whole app
 * reads. Omitted, the frame keeps its own: a listed `ViewDraw.signedOut` opens it signed out and
 * `Sign out` signs it out, as the workbench draws it.
 */
export type FrameSession = { signedIn: boolean; signOut: () => void };

/**
 * **A read that failed, where the caller learns of it** — the app's, from the query cache.
 * `ground`: the sheet's own reads failed, and the frame gives way to
 * `views/read-failed`. Otherwise a page's read failed, and every wait on screen says so.
 * `retrying` while the failed reads are asked again. Omitted, the frame keeps its own from a listed
 * `ViewDraw.readFailed`, as the workbench draws it.
 */
export type FrameReadFailed = { ground: boolean; retrying: boolean; retry: () => void };

export type FrameProps = {
	/**
	 * What is being framed: `dest` is the destination whose page `draw` is — `null` for the ground
	 * — and `id` keys the page's wipe, so two errands on one destination wipe between each other.
	 */
	place: { id: string; dest: string | null };
	draw: ViewDraw;
	/**
	 * **The rail's cells, in order**: the destinations this caller has a page for, and no other. The
	 * app hands it the ones it routes; the workbench every one, so a destination can stand on the
	 * bench before the app has a page for it without becoming a cell there that leads nowhere.
	 */
	destinations: readonly SplitDest[];
	/**
	 * What the ground is called — the logo's name, and where a guarded press to the ground says it
	 * was going: the app's name (`views/destinations`' `GROUND`).
	 */
	home: string;
	/** Where the frame goes when the ledger is collapsed with nothing else open: a destination's id. */
	resting: string;
	/** The sheet as it stands, wherever the page declares none of its own. */
	ledger: ReactNode;
	/**
	 * Every cell's one fact at rest. A page changes its own through `ViewDraw.facts`; a cell named
	 * in neither prints its own `fact`.
	 */
	facts: Partial<Record<string, string>>;
	/** What the slug prints on the ground: the app's name and its one fact. */
	groundReading: RailReading;
	/** Who answers sign-in, for the condition it opens in. */
	server: (opens: SignedOut) => SignInServer;
	/** Where the frame is, when the caller keeps it — the app's address. See `kit/SplitShell`. */
	control?: SplitControl;
	/** Which record the page has open inside itself, when the caller keeps it. */
	nestControl?: NestControl;
	session?: FrameSession;
	/** A read that failed, or `null` while none has. See {@link FrameReadFailed}. */
	readFailed?: FrameReadFailed | null;
	/**
	 * The door a press from outside the page leaves by — the sheet's, which the caller wires.
	 * Undefined, nothing outside the frame can ask it.
	 */
	door?: FrameDoor;
};

/**
 * **A way out the rail has no cell for, made through the frame**: another record, pressed on the
 * sheet. The caller keeps one door for the frame's life and presses it; the frame
 * behind it asks first while the page holds something you would lose — the same confirm, the
 * same two outcomes, the loss line naming where the press was going — and goes straight through
 * when nothing is held.
 */
export type FrameDoor = {
	go: (name: string, then: () => void) => void;
	/** The frame's own: what `go` asks. */
	open: (away: (name: string, then: () => void) => void) => void;
};

/** A door, shut until a frame opens it — then every `go` goes through that frame's guard. */
export function createFrameDoor(): FrameDoor {
	let away: ((name: string, then: () => void) => void) | null = null;
	return {
		go: (name, then) => {
			if (away === null) then();
			else away(name, then);
		},
		open: next => {
			away = next;
		},
	};
}

/**
 * **The frame, whole**: the split, the rail, the slug, the hold and its confirm, the detour, the
 * nested page and sign-in, around one page's declaration.
 *
 * Everything it needs is passed in rather than read from a fixture world: the resting sheet, the
 * cells' facts, the ground's reading and the sign-in server. The workbench frames a place with it
 * (`views/place-frame`); the app frames a route with it, keeping `where` in the address and the
 * session in its token.
 */
export function Frame({
	place,
	draw,
	destinations,
	home: homeName,
	resting,
	ledger,
	facts: restingFacts,
	groundReading,
	server,
	control,
	nestControl,
	session,
	readFailed,
	door,
}: FrameProps) {
	const [detouring, setDetouring] = useState(false);
	/** A listed failed read, which `Try again` clears as signing in clears `signedOut`. */
	const [ownFailed, setOwnFailed] = useState(draw.readFailed ?? null);
	const failed: FrameReadFailed | null =
		readFailed === undefined
			? ownFailed === null
				? null
				: {
						ground: ownFailed === 'ground',
						retrying: false,
						retry: () => {
							setOwnFailed(null);
						},
					}
			: readFailed;
	/**
	 * **Signed out**: sign-in drawn whole in place of the shell. A listed condition opens in it;
	 * Account's `Sign out` reaches the empty one, and `byPress` hands that one the keyboard.
	 */
	const [ownSignedOut, setSignedOut] = useState<{ opens: SignedOut; byPress: boolean } | null>(
		draw.signedOut === undefined ? null : { opens: draw.signedOut, byPress: false }
	);
	/** Whether the caller's session ended on this frame's own `Sign out`, which takes the keyboard. */
	const [outByPress, setOutByPress] = useState(false);
	const signedOut =
		session === undefined
			? ownSignedOut
			: session.signedIn
				? null
				: { opens: 'empty' as const, byPress: outByPress };
	/** What opened the detour, so the way back returns the keyboard to it. */
	const opener = useRef<HTMLElement | null>(null);
	const detourRead = useSyncExternalStore<DetourRead | null>(
		draw.detour?.subscribe ?? NO_DETOUR,
		draw.detour?.get ?? NO_READ
	);

	/**
	 * `useHeld` rather than `useSplit`: the same settled cells, plus the guard.
	 *
	 * With nothing held it *is* `useSplit` — every press goes straight through — so a place that
	 * declares no hold behaves exactly as an unguarded split does.
	 */
	/** A declared hold is read once; a live one is followed. */
	const [declared] = useState<Hold | null>(() =>
		draw.held === undefined || isSource(draw.held) ? null : draw.held
	);
	const source = isSource(draw.held) ? draw.held : null;
	const live = useSyncExternalStore<Hold | null>(
		source?.subscribe ?? NO_DETOUR,
		source?.get ?? NO_HOLD
	);

	/** Whether the page is a stage, and what the slug prints of it — both followed while live. */
	const staged = useSyncExternalStore(
		draw.centred?.subscribe ?? NO_DETOUR,
		draw.centred?.get ?? NOT_CENTRED
	);
	const declaredReading = draw.reading;
	const readingSource: Live<RailReading> = isLive(declaredReading)
		? declaredReading
		: { subscribe: NO_DETOUR, get: () => declaredReading };
	const drawnReading = useSyncExternalStore(readingSource.subscribe, readingSource.get);

	const hand = useHeld(
		{
			ledger: true,
			dest: place.dest,
			menu: draw.menu ?? false,
			held: source === null ? declared : live,
		},
		{ resting, control }
	);

	/** The door the caller presses from outside the page goes through this frame's guard. */
	useLayoutEffect(() => {
		door?.open(hand.away);
	});

	/**
	 * Every way a hold ends tells the record, so a run stops when its hold does — and tells it which
	 * cell, so a hold with two capabilities can do two different things. The cell that raised
	 * the confirm is remembered until the confirm is answered.
	 */
	const ending = hand.held?.onEnd;
	const asked = useRef<string | null>(null);
	const discard = () => {
		const cell = asked.current;
		asked.current = null;
		hand.discard();
		ending?.(cell);
	};
	const stay = () => {
		asked.current = null;
		hand.stay();
	};
	const heldNow = hand.held;
	const press =
		heldNow === null
			? null
			: (id: string) => {
					const cell = heldNow.cells.find(each => each.id === id);
					if (cell === undefined) return;
					if (cell.asks === true) {
						asked.current = id;
						hand.stop();
						return;
					}
					hand.release();
					ending?.(id);
				};

	/** The destination an id names, among the ones this frame was handed — and no other. */
	const find = (id: string): SplitDest | null =>
		destinations.find(entry => entry.id === id) ?? null;

	const open = hand.where.dest;
	const dest = open === null ? null : find(open);

	/** The place's own destination — whose glyph the slug wears for *back to this page*. */
	const own = place.dest === null ? null : find(place.dest);

	/** Whether what is open is the place being judged, or a cell somebody pressed to get off it. */
	const here = open !== null && open === place.dest;

	/**
	 * **A nested page**: which record the page has open inside itself. It belongs to the page,
	 * so leaving the page lets it go; coming back to the page finds the page, not it.
	 */
	const nesting = useNest(draw.nest, place.dest, open, draw.nested, nestControl);
	const inner = nesting.open;

	/**
	 * **Back to the page from a nested page that is holding its own record.** Free everywhere
	 * else; here it loses the work, so it raises the confirm like any way out, and discarding
	 * ends the hold before the nested page lets go.
	 */
	const [backing, setBacking] = useState(false);
	const back = () => {
		if (hand.where.menu) hand.slug();
		if (inner !== null && heldNow?.nested === true) {
			setBacking(true);
			return;
		}
		nesting.close();
	};
	const backDiscard = () => {
		setBacking(false);
		hand.release();
		ending?.(null);
		nesting.close();
	};

	/** Every destination press, from the rail, the register or a link on the page, is this one. */
	const goTo = (id: string) => {
		/**
		 * The page's own cell, pressed on a nested page, goes back to the page: it is that page's
		 * glyph, and a page's glyph goes back to that page. A second press closes it as ever.
		 */
		if (inner !== null && id === open) {
			back();
			return;
		}
		setDetouring(false);
		hand.go(id);
	};

	/**
	 * **A link on the page is a way to a destination, and it goes through the rail's door.**
	 *
	 * Left to the browser, a link to another destination's record would be a way out of a hold that
	 * nothing guarded: the browser follows the link and the hold never hears of it. A link to one of
	 * the frame's destinations (`hrefId`) is made the same press a rail
	 * cell makes, so it asks while anything is held and goes straight through when nothing is. A link
	 * to the page you are on is left to the browser — the rail's press would close the page.
	 *
	 * **A link to a record the page opens inside itself is read first**: it is the nest's press,
	 * free and never asked.
	 */
	const follow = (href: string): boolean => {
		if (nesting.follow(href)) return true;
		const id = hrefId(href);
		const to = id === null ? null : (find(id)?.id ?? null);
		if (to === null || to === open) return false;
		goTo(to);
		return true;
	};

	const head: PageHeading | null =
		dest === null
			? null
			: inner !== null && draw.nest !== undefined
				? {
						title: inner.name,
						lede: typeof draw.nest.lede === 'string' ? draw.nest.lede : draw.nest.lede(inner.id),
						cut: true,
					}
				: here
					? (draw.head ?? headOf(dest))
					: headOf(dest);

	/**
	 * The slug prints what the variant says only while the variant is what is on screen. Press
	 * another cell and the frame is somewhere the place never declared, so the rail's own word for
	 * that destination is the honest thing to print.
	 *
	 * **A peek is the third case, and it is the one the slug does not print.** While
	 * anything is held the box is the hold's row and this reading is not drawn at all; it is still
	 * computed, because the hold ends on a press and the bar comes back the same frame.
	 *
	 * **Leaving a page for the ground is the fourth, and it prints the ground.** A place whose page
	 * is a destination declares that page's reading, and the ground is somewhere it never declared:
	 * `←` from a page would leave the slug printing that page's reading over the sheet. Only the
	 * ground's own place prints its declared reading there.
	 */
	/** Every cell's one fact: the caller's at rest, and whatever this place changed. */
	const facts = { ...restingFacts, ...draw.facts };

	const onGround = hand.peeking || (dest === null && place.dest !== null);
	const reading = onGround
		? groundReading
		: here || dest === null
			? drawnReading
			: { place: dest.label, fact: facts[dest.id] ?? dest.fact ?? '' };

	/**
	 * The hold, wired: the record's words are the place's, every press is the frame's.
	 *
	 * It is the **place's** hold and not the frame's position — a page goes on holding its draft
	 * while you stand on another destination above `lg` deciding whether to leave it, which is what
	 * the guard and its confirm are for. Below `lg` that walk is not possible at all: the slug is the
	 * hold's row for the length of the errand.
	 */
	const enterDetour =
		draw.detour === undefined
			? null
			: () => {
					opener.current =
						document.activeElement instanceof HTMLElement ? document.activeElement : null;
					draw.detour?.onEnter?.();
					setDetouring(true);
				};

	const detour: DetourStatus | null =
		detouring && here && dest !== null && detourRead !== null
			? {
					name: dest.label,
					icon: dest.icon,
					fact: detourRead.fact,
					cells: detourRead.cells,
					onBack: () => {
						setDetouring(false);
						requestAnimationFrame(() => {
							opener.current?.focus();
						});
					},
				}
			: null;

	const held: HeldStatus | null =
		hand.held === null || own === null
			? null
			: {
					name: hand.held.name,
					fact: hand.held.fact,
					share: hand.held.share,
					peeking: hand.peeking,
					onPeek: hand.peek,
					busy: hand.held.busy,
					reads: hand.held.reads,
					icon: own.icon,
					cells: hand.held.cells.map(cell => ({
						...cell,
						onPress: () => {
							press?.(cell.id);
						},
					})),
				};

	/**
	 * **The page's own errand on the slug**, below `lg`: only while the place's own page is what
	 * is on screen. It is a link, as the plate is, so it opens what the plate opens through the same
	 * door — the frame's own link press.
	 */
	const signOut = () => {
		if (session === undefined) {
			setSignedOut({ opens: 'empty', byPress: true });
			return;
		}
		setOutByPress(true);
		session.signOut();
	};
	const errand: ErrandCell | null =
		here && draw.errand !== undefined ? errandCell(draw.errand, signOut) : null;

	/** The nested page, as the frame draws it: the way back goes to this page, as it was left. */
	const nested: NestedPage | null =
		inner === null || dest === null
			? null
			: {
					id: inner.id,
					name: inner.name,
					page: dest.label,
					icon: dest.icon,
					onBack: back,
				};

	if (signedOut !== null) {
		return (
			<SignIn
				opens={signedOut.opens}
				focusOnOpen={signedOut.byPress}
				server={server(signedOut.opens)}
				onSignedIn={() => {
					setSignedOut(null);
					setOutByPress(false);
					hand.home();
				}}
			/>
		);
	}

	/**
	 * The ground's reads failing puts the centred screen up and the frame **away, not down**: it
	 * stays mounted under the screen, so a page holding unsaved work still holds it when the reads
	 * come back and the screen goes.
	 */
	const groundFailed = failed?.ground === true;

	return (
		<WaitFailed onRetry={failed?.retry ?? null}>
			{groundFailed ? <ReadFailed retrying={failed.retrying} onRetry={failed.retry} /> : null}
			<div hidden={groundFailed}>
				<NestProvider value={inner}>
					<SplitShell
						destinations={destinations.map(entry => ({
							...entry,
							fact: facts[entry.id] ?? entry.fact,
						}))}
						where={hand.where}
						reading={reading}
						home={homeName}
						head={head}
						centred={here && staged}
						flow={`${place.id}:${open ?? 'ground'}:${here ? 'drawn' : 'undrawn'}${inner === null ? '' : `:${inner.id}`}`}
						ledger={draw.ledger ?? ledger}
						held={held}
						detour={detour}
						nested={nested}
						errand={errand}
						away={groundFailed}
						overlay={
							backing && hand.held !== null && dest !== null ? (
								<LeaveDialog
									hold={hand.held}
									going={dest.label}
									onStay={() => {
										setBacking(false);
									}}
									onDiscard={backDiscard}
								/>
							) : hand.going !== null && hand.held !== null ? (
								<LeaveDialog
									hold={hand.held}
									going={goingName(hand.going, hand.awayName, homeName, find)}
									onStay={stay}
									onDiscard={discard}
								/>
							) : here ? (
								draw.overlay
							) : undefined
						}
						onGo={goTo}
						onLink={follow}
						onLogo={hand.toggleLedger}
						onSlug={hand.slug}
						onHome={hand.home}
					>
						{here ? (
							<DetourContext.Provider value={enterDetour}>
								<PressContext.Provider value={press}>
									<SignOutContext.Provider value={signOut}>
										{draw.nest === undefined ? (
											draw.page
										) : (
											<NestSwitch
												open={inner?.id ?? null}
												parent={draw.page}
												nested={inner === null ? null : draw.nest.page(inner.id)}
											/>
										)}
									</SignOutContext.Provider>
								</PressContext.Provider>
							</DetourContext.Provider>
						) : (
							<UndrawnPage />
						)}
					</SplitShell>
				</NestProvider>
			</div>
		</WaitFailed>
	);
}

/**
 * **A page's own errand, in its flow, from `lg`** — the ink plate under the head. Below `lg` the
 * slug carries the same declaration as a cell, so this draws nothing there: which surface has it is
 * CSS on one declaration, never a width each side reads for itself.
 */
export function ErrandPlate({ errand }: { errand: PageErrand }) {
	const signOut = useContext(SignOutContext);

	return (
		<div className="hidden pb-8 lg:flex">
			{'href' in errand ? (
				<Button weight="primary" href={errand.href}>
					{errand.label}
				</Button>
			) : (
				<Button weight="primary" onClick={signOut}>
					{errand.label}
				</Button>
			)}
		</div>
	);
}
