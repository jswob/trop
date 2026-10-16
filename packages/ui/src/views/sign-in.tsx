import { type FormEvent, type ReactNode, type Ref, useEffect, useRef, useState } from 'react';

import { Button, Mark, READ_FAILED, SCREEN_H, TextField } from '@app/ui/kit';
import { GROUND } from '@app/ui/views/destinations';
import type { SignedOut } from '@app/ui/views/frame-types';

/**
 * **Sign-in, centred.**
 *
 * It stands outside the frame: no rail, no slug, no sheet, because nothing behind it can be read
 * until the password is in. The mark at three times its rail size, the product's name at display
 * size, one line saying why there is no name to type, then the field and the `Sign in` plate at the
 * full width of one 20rem column in the middle of the screen. No card, no shadow, no radius.
 *
 * `views/frame` draws it in place of the shell while the app is signed out, so Account's `Sign out`
 * lands here and a right password lands on the sheet.
 *
 * **Who answers is the caller's**: the app hands it the API's login, the workbench a stand-in
 * (`apps/workbench/fixtures/sign-in-server`). A promise that never settles is a server still
 * answering, which is how the workbench draws `signing-in`.
 */

/** What the server said to one password. A server that never answered rejects instead. */
export type SignInAnswer = 'ok' | 'wrong' | 'not-set-up';

/** Sends one password and says what came back. */
export type SignInServer = (password: string) => Promise<SignInAnswer>;

/** Ten characters in the box for the conditions that open with a press already made. */
const TYPED = 'correcthor';

export const SIGN_IN_WORDS = {
	name: GROUND,
	lede: 'Everyone here uses the same password.',
	field: 'Password',
	press: 'Sign in',
	running: 'Signing in',
	wrong: `That isn’t the password. Ask whoever set up ${GROUND} for the current one.`,
	notSetUp:
		'Signing in isn’t set up on this server. Set AUTH_PASSWORD and JWT_SECRET for the API, then restart it.',
} as const;

type Status = 'idle' | 'busy' | 'wrong' | 'not-set-up' | 'unreachable';

const OPENS: Record<SignedOut, Status> = {
	empty: 'idle',
	'signing-in': 'busy',
	'not-set-up': 'not-set-up',
	unreachable: 'unreachable',
};

/**
 * **The centred screen**: the mark at three times its rail size and the product's name at display
 * size over one 20rem column in the middle of the ground. No card, no shadow, no radius. Sign-in
 * stands on it, and so does a read that failed under the whole frame, since neither has a frame it
 * can trust.
 */
export function CentredScreen({
	lede,
	children,
	column,
}: {
	/** The line under the name — sign-in's why, or a failed read's what. */
	lede: ReactNode;
	children: ReactNode;
	column?: Ref<HTMLElement>;
}) {
	return (
		<div
			className={`flex ${SCREEN_H} items-center justify-center overflow-y-auto bg-ui-bg px-4 font-ui-sans text-ui-ink [color-scheme:var(--ui-scheme)]`}
		>
			<main ref={column} className="w-[20rem] max-w-full py-10">
				<div className="flex flex-col items-center text-center">
					<Mark size="display" />
					<h1 className="mt-5 text-ui-display leading-8 text-ui-ink">{SIGN_IN_WORDS.name}</h1>
					{lede}
				</div>
				{children}
			</main>
		</div>
	);
}

/** A line of the app replying, in red type and never on a red plate, under the plate it is about. */
function Refusal({ children }: { children: ReactNode }) {
	return (
		<p role="alert" className="flex items-start gap-1.5 text-ui-body text-ui-red">
			<span aria-hidden>!</span>
			{children}
		</p>
	);
}

/**
 * `focusOnOpen` is true when a press brought the app here — `Sign out` — so the keyboard lands on
 * the field instead of the page's body. A listed condition opens without taking it.
 */
export function SignIn({
	opens = 'empty',
	focusOnOpen = false,
	server,
	onSignedIn,
}: {
	opens?: SignedOut;
	focusOnOpen?: boolean;
	server: SignInServer;
	onSignedIn: () => void;
}) {
	const [password, setPassword] = useState(opens === 'empty' ? '' : TYPED);
	const [status, setStatus] = useState<Status>(OPENS[opens]);
	/** Cleared on unmount, so an answer that lands after the screen has gone changes nothing. */
	const mounted = useRef(true);
	const column = useRef<HTMLElement>(null);

	/**
	 * The plate turns busy under the keyboard, so an answer that refuses hands it back to the field,
	 * where the fix is typed — and the field's reason is read out with it.
	 */
	const toField = () => {
		column.current?.querySelector('input')?.focus();
	};

	useEffect(() => {
		mounted.current = true;
		if (focusOnOpen) column.current?.querySelector('input')?.focus();
		return () => {
			mounted.current = false;
		};
	}, [focusOnOpen]);

	const submit = () => {
		if (password === '' || status === 'busy') return;
		setStatus('busy');
		server(password).then(
			answer => {
				if (!mounted.current) return;
				if (answer === 'ok') {
					onSignedIn();
					return;
				}
				setStatus(answer);
				requestAnimationFrame(toField);
			},
			/**
			 * A server that never answered, or erred: the plate comes back so the press can be made
			 * again, and the line under it says why. Nothing claims the password was wrong.
			 */
			() => {
				if (!mounted.current) return;
				setStatus('unreachable');
				requestAnimationFrame(toField);
			}
		);
	};

	return (
		<CentredScreen
			column={column}
			lede={
				<p className="mt-1.5 text-ui-body leading-5 text-balance text-ui-ink-2">
					{SIGN_IN_WORDS.lede}
				</p>
			}
		>
			<form
				noValidate
				className="mt-6 grid gap-4"
				onSubmit={(event: FormEvent) => {
					event.preventDefault();
					submit();
				}}
			>
				<TextField
					secret
					label={SIGN_IN_WORDS.field}
					value={password}
					error={status === 'wrong' ? SIGN_IN_WORDS.wrong : null}
					onChange={next => {
						setPassword(next);
						if (status === 'wrong') setStatus('idle');
					}}
				/>
				{/* A grid item stretches, so the plate runs the column's width without a class on it. */}
				<div className="grid gap-2">
					<Button
						weight="primary"
						disabled={password === ''}
						running={status === 'busy' ? SIGN_IN_WORDS.running : null}
						onClick={submit}
					>
						{SIGN_IN_WORDS.press}
					</Button>
					{status === 'not-set-up' ? <Refusal>{SIGN_IN_WORDS.notSetUp}</Refusal> : null}
					{status === 'unreachable' ? <Refusal>{READ_FAILED}</Refusal> : null}
				</div>
			</form>
		</CentredScreen>
	);
}
