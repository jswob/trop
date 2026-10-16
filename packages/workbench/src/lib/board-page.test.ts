import { afterEach, describe, expect, it, vi } from 'vitest';

import { fitScale, frameBox, startBoard } from './board-page';

function page(markup: string): Document {
	const doc = document.implementation.createHTMLDocument('board');
	doc.body.innerHTML = markup;
	return doc;
}

describe('fitScale', () => {
	it('shrinks a frame to the room it is given and never enlarges it', () => {
		expect(fitScale(720, 1440)).toBe(0.5);
		expect(fitScale(2000, 1440)).toBe(1);
	});

	it('holds a frame at full size while its box has no width yet', () => {
		expect(fitScale(0, 390)).toBe(1);
	});
});

describe('frameBox', () => {
	it('reads the frame and its viewport off the element', () => {
		const doc = page('<div data-frame="people:person" data-width="1440" data-height="900"></div>');
		const element = doc.querySelector<HTMLElement>('[data-frame]');

		expect(element && frameBox(element)).toEqual({
			spec: 'people:person',
			width: 1440,
			height: 900,
		});
	});

	it('is a phone when no size is given', () => {
		const doc = page('<div data-frame="people:person"></div>');
		const element = doc.querySelector<HTMLElement>('[data-frame]');

		expect(element && frameBox(element)).toEqual({
			spec: 'people:person',
			width: 390,
			height: 844,
		});
	});
});

describe('startBoard', () => {
	it('puts a real frame at true size in every marked box', () => {
		const doc = page(
			'<div data-frame="people:person" data-width="1440" data-height="900" data-title="A person, wide"></div>' +
				'<div data-frame="people:grid"></div>'
		);

		startBoard(doc);
		const frames = [...doc.querySelectorAll('iframe')];

		expect(frames).toHaveLength(2);
		expect(frames[0]?.getAttribute('name')).toBe('people:person');
		expect(frames[0]?.getAttribute('src')).toBe('frame.html');
		expect(frames[0]?.getAttribute('title')).toBe('A person, wide');
		expect(frames[0]?.style.width).toBe('1440px');
		expect(frames[0]?.style.height).toBe('900px');
		expect(frames[1]?.style.width).toBe('390px');
		expect(frames[1]?.getAttribute('title')).toBe('people:grid');
	});

	it('gives the box the frame’s proportions, capped at its true width', () => {
		const doc = page('<div data-frame="people:person" data-width="1440" data-height="900"></div>');

		startBoard(doc);
		const box = doc.querySelector<HTMLElement>('[data-frame]');

		expect(box?.style.aspectRatio).toBe('1440 / 900');
		expect(box?.style.maxWidth).toBe('1440px');
	});

	it('names the appearance on a frame it creates after the board turned light', () => {
		const doc = page('<div data-frame="people:person?rate=0"></div>');

		const board = startBoard(doc);
		board.theme('light');
		const later = doc.createElement('div');
		later.dataset['frame'] = 'people:grid';
		doc.body.append(later);
		board.mount(later);

		expect(later.querySelector('iframe')?.getAttribute('name')).toBe('people:grid?theme=light');
	});
});

describe('a board turning over', () => {
	afterEach(() => {
		document.body.innerHTML = '';
	});

	it('tells every frame it mounted, in a message named for the board', () => {
		document.body.innerHTML = '<div data-frame="people:person"></div>';
		const board = startBoard(document);
		const frame = document.querySelector('iframe');
		const post = vi.fn();
		if (frame?.contentWindow) frame.contentWindow.postMessage = post;

		board.theme('light');

		expect(post).toHaveBeenCalledWith({ boardTheme: 'light' }, '*');
	});
});
