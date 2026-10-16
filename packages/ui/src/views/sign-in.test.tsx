import { act, cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { READ_FAILED } from '@app/ui/kit';
import { SIGN_IN_WORDS, SignIn, type SignInAnswer, type SignInServer } from './sign-in';

/**
 * Sign-in's presses, and what it does with each answer.
 *
 * A wrong password has no listed entry because a press reaches it, so what that press does is
 * asserted here rather than photographed. The server is a stand-in per test: the screen's job is
 * what it draws for an answer, not who gives it.
 */

afterEach(cleanup);

function field(): HTMLInputElement {
	const input = screen.getByLabelText(SIGN_IN_WORDS.field);
	if (!(input instanceof HTMLInputElement)) throw new Error('the field is not an input');
	return input;
}

function plate(name: string = SIGN_IN_WORDS.press): HTMLButtonElement {
	const button = screen.getByRole('button', { name });
	if (!(button instanceof HTMLButtonElement)) throw new Error('the plate is not a button');
	return button;
}

/** A server that gives one answer to every password, and says what it was sent. */
function answering(answer: SignInAnswer) {
	return vi.fn<SignInServer>(() => Promise.resolve(answer));
}

/** Let the server's answer land. */
async function answer() {
	await act(async () => {
		await Promise.resolve();
	});
}

describe('sign-in, empty', () => {
	it('keeps the plate off until something is typed, and hides what is typed', () => {
		render(<SignIn server={answering('ok')} onSignedIn={() => undefined} />);

		expect(field().type).toBe('password');
		expect(plate().disabled).toBe(true);
		fireEvent.change(field(), { target: { value: 'x' } });
		expect(plate().disabled).toBe(false);
	});

	it('refuses a wrong password on the field, hands it the keyboard, and lets go once somebody types', async () => {
		vi.spyOn(window, 'requestAnimationFrame').mockImplementation(step => {
			step(0);
			return 0;
		});
		const signedIn = vi.fn();
		render(<SignIn server={answering('wrong')} onSignedIn={signedIn} />);
		fireEvent.change(field(), { target: { value: 'wrong' } });
		fireEvent.click(plate());

		expect(plate(SIGN_IN_WORDS.running).disabled).toBe(true);
		await answer();
		expect(screen.getByText(SIGN_IN_WORDS.wrong)).toBeTruthy();
		expect(field().getAttribute('aria-invalid')).toBe('true');
		expect(signedIn).not.toHaveBeenCalled();
		expect(document.activeElement).toBe(field());

		fireEvent.change(field(), { target: { value: 'wrong again' } });
		expect(screen.queryByText(SIGN_IN_WORDS.wrong)).toBeNull();
		vi.restoreAllMocks();
	});

	it('sends what was typed and signs in on the answer, from the plate or from Enter', async () => {
		const server = answering('ok');
		const signedIn = vi.fn();
		render(<SignIn server={server} onSignedIn={signedIn} />);
		fireEvent.change(field(), { target: { value: 'hunter2' } });
		fireEvent.submit(field());
		await answer();

		expect(server).toHaveBeenCalledWith('hunter2');
		expect(signedIn).toHaveBeenCalledTimes(1);
	});

	it('takes the keyboard when a press brought it here, and not otherwise', () => {
		render(<SignIn focusOnOpen server={answering('ok')} onSignedIn={() => undefined} />);
		expect(document.activeElement).toBe(field());
		cleanup();

		render(<SignIn server={answering('ok')} onSignedIn={() => undefined} />);
		expect(document.activeElement).not.toBe(field());
	});

	/** An unreachable server is said under the plate, never as a wrong password. */
	it('gives the plate back and says the server was not reached, when it fails some other way', async () => {
		const signedIn = vi.fn();
		render(<SignIn server={() => Promise.reject(new Error('offline'))} onSignedIn={signedIn} />);
		fireEvent.change(field(), { target: { value: 'hunter2' } });
		fireEvent.click(plate());
		await answer();

		expect(plate().disabled).toBe(false);
		expect(screen.queryByText(SIGN_IN_WORDS.wrong)).toBeNull();
		expect(screen.getByRole('alert').textContent).toContain(READ_FAILED);
		expect(signedIn).not.toHaveBeenCalled();
	});
});

describe('sign-in, the two conditions no press reaches', () => {
	it('waits on a server that never answers', async () => {
		const signedIn = vi.fn();
		render(
			<SignIn
				opens="signing-in"
				server={() => new Promise(() => undefined)}
				onSignedIn={signedIn}
			/>
		);
		await answer();

		expect(plate(SIGN_IN_WORDS.running).disabled).toBe(true);
		expect(signedIn).not.toHaveBeenCalled();
	});

	it('names the fix on a server with no password, and on every press after', async () => {
		const signedIn = vi.fn();
		render(<SignIn opens="not-set-up" server={answering('not-set-up')} onSignedIn={signedIn} />);
		expect(screen.getByRole('alert').textContent).toContain(SIGN_IN_WORDS.notSetUp);

		fireEvent.change(field(), { target: { value: 'hunter2' } });
		fireEvent.click(plate());
		await answer();
		expect(screen.getByRole('alert').textContent).toContain(SIGN_IN_WORDS.notSetUp);
		expect(signedIn).not.toHaveBeenCalled();
	});
});
