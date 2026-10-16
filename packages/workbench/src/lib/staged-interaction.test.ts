import { describe, expect, it, vi } from 'vitest';

import { clickWhenPresent, fillWhenPresent } from './staged-interaction';

/** Waits for the MutationObserver callback, which is queued as a microtask. */
const flush = () => new Promise(resolve => setTimeout(resolve, 0));

describe('clickWhenPresent', () => {
	it('clicks an element that is already there', () => {
		const button = document.createElement('button');
		const onClick = vi.fn();
		button.addEventListener('click', onClick);
		document.body.append(button);

		clickWhenPresent(() => button);

		expect(onClick).toHaveBeenCalledTimes(1);
		button.remove();
	});

	it('clicks an element that only appears on a later commit', async () => {
		const onClick = vi.fn();
		const stop = clickWhenPresent(() => document.querySelector<HTMLElement>('[data-late]'));

		expect(onClick).not.toHaveBeenCalled();

		const button = document.createElement('button');
		button.dataset.late = '';
		button.addEventListener('click', onClick);
		document.body.append(button);
		await flush();

		expect(onClick).toHaveBeenCalledTimes(1);
		stop();
		button.remove();
	});

	it('clicks once, however many times the effect is re-run', async () => {
		const onClick = vi.fn();
		const button = document.createElement('button');
		button.dataset.toggle = 'false';
		button.addEventListener('click', () => {
			button.dataset.toggle = 'true';
			onClick();
		});
		document.body.append(button);

		const guarded = () =>
			button.dataset.toggle === 'true' ? null : (button satisfies HTMLElement);

		const first = clickWhenPresent(guarded);
		first();
		const second = clickWhenPresent(guarded);
		await flush();

		expect(onClick).toHaveBeenCalledTimes(1);
		second();
		button.remove();
	});

	it('stops watching once torn down', async () => {
		const onClick = vi.fn();
		const stop = clickWhenPresent(() => document.querySelector<HTMLElement>('[data-never]'));
		stop();

		const button = document.createElement('button');
		button.dataset.never = '';
		button.addEventListener('click', onClick);
		document.body.append(button);
		await flush();

		expect(onClick).not.toHaveBeenCalled();
		button.remove();
	});
});

describe('clickWhenPresent — attribute changes', () => {
	it('clicks a button that is only enabled on a later commit', async () => {
		const onClick = vi.fn();
		const button = document.createElement('button');
		button.disabled = true;
		button.addEventListener('click', onClick);
		document.body.append(button);

		const stop = clickWhenPresent(() => (button.disabled ? null : button));
		expect(onClick).not.toHaveBeenCalled();

		button.disabled = false;
		await flush();

		expect(onClick).toHaveBeenCalledTimes(1);
		stop();
		button.remove();
	});
});

describe('fillWhenPresent', () => {
	it('sets the value and fires the input event React listens for', () => {
		const input = document.createElement('input');
		const onInput = vi.fn();
		input.addEventListener('input', onInput);
		document.body.append(input);

		fillWhenPresent(() => input, 'hunter2');

		expect(input.value).toBe('hunter2');
		expect(onInput).toHaveBeenCalledTimes(1);
		expect(onInput.mock.calls[0]![0]).toMatchObject({ bubbles: true });
		input.remove();
	});

	it('fills an input that only appears on a later commit', async () => {
		const stop = fillWhenPresent(
			() => document.querySelector<HTMLInputElement>('[data-late-input]'),
			'hunter2'
		);

		const input = document.createElement('input');
		input.dataset.lateInput = '';
		document.body.append(input);
		await flush();

		expect(input.value).toBe('hunter2');
		stop();
		input.remove();
	});

	it('fills once, however many times the effect is re-run', async () => {
		const input = document.createElement('input');
		const onInput = vi.fn();
		input.addEventListener('input', onInput);
		document.body.append(input);

		const first = fillWhenPresent(() => input, 'hunter2');
		first();
		const second = fillWhenPresent(() => input, 'hunter2');
		await flush();

		expect(onInput).toHaveBeenCalledTimes(1);
		second();
		input.remove();
	});
});
