import { act } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { boardSubjects, startFrame } from './board-frame';
import { readPosition, writePosition } from './frame-position';
import type { Subject } from './subject';

const tally: Subject = {
	id: 'person-tally',
	title: 'A · Tally',
	description: 'The count beside the seat.',
	chrome: 'bare',
	variants: [{ id: 'resting', label: 'Resting', render: () => <p>tally drawn</p> }],
};

describe('boardSubjects', () => {
	it('takes every subject a board’s files export, whatever it is called', () => {
		expect(
			boardSubjects([{ personTally: tally }, { default: { ...tally, id: 'person-bar' } }])
		).toEqual([tally, { ...tally, id: 'person-bar' }]);
	});

	it('leaves a file’s other exports alone', () => {
		expect(
			boardSubjects([
				{ personTally: tally, ROWS: [1, 2], helper: () => 1, shape: { id: 7 }, none: null },
			])
		).toEqual([tally]);
	});
});

describe('startFrame', () => {
	const post = vi.spyOn(window.parent, 'postMessage');

	afterEach(() => {
		post.mockClear();
		window.name = '';
		document.body.innerHTML = '';
		delete document.documentElement.dataset['theme'];
		writePosition(null);
	});

	async function open(name: string): Promise<void> {
		document.body.innerHTML = '<div id="root"></div>';
		window.name = name;
		post.mockImplementation(() => undefined);
		await act(async () => {
			startFrame([{ personTally: tally }]);
			await Promise.resolve();
		});
	}

	it('draws the board’s own drawing its name asks for', async () => {
		await open('person-tally:resting');

		expect(document.body.textContent).toContain('tally drawn');
	});

	it('tells its parent what it measured, in a message named for the board', async () => {
		await open('person-tally:resting');

		expect(post).toHaveBeenCalledWith(
			{ boardFrame: 'person-tally:resting', width: window.innerWidth, height: window.innerHeight },
			'*'
		);
	});

	it('turns over when the board says so', async () => {
		await open('person-tally:resting');

		await act(async () => {
			window.dispatchEvent(new MessageEvent('message', { data: { boardTheme: 'light' } }));
			await Promise.resolve();
		});

		expect(document.documentElement.dataset['theme']).toBe('light');
	});

	it('opens the frame where its name says it has walked to', async () => {
		await open('person-tally:resting?where=roles');

		expect(readPosition()).toBe('roles');
		expect(document.body.textContent).toContain('tally drawn');
	});

	it('leaves a frame that names no position at rest', async () => {
		await open('person-tally:resting');

		expect(readPosition()).toBeNull();
	});

	it('says in plain words that a name found nothing', async () => {
		await open('nowhere:resting');

		expect(document.body.textContent).toContain('No frame called nowhere.');
	});
});
