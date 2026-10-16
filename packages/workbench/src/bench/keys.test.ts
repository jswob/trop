import { describe, expect, it } from 'vitest';

import { keyAction } from './keys';

const IDS = ['edit', 'holding', 'blocked'];

describe('keyAction', () => {
	it('steps with the bracket keys', () => {
		expect(keyAction(']', IDS, { variants: 'edit', at: 'edit' })).toEqual({
			variants: 'holding',
			at: 'holding',
		});
		expect(keyAction('[', IDS, { variants: 'edit', at: 'edit' })).toEqual({
			variants: 'blocked',
			at: 'blocked',
		});
	});

	it('brings every state back with A, in either case', () => {
		expect(keyAction('a', IDS, { variants: 'edit' })).toEqual({});
		expect(keyAction('A', IDS, { variants: 'edit' })).toEqual({});
	});

	it('goes to Widths and back with W', () => {
		expect(keyAction('w', IDS, {})).toEqual({ show: 'widths' });
		expect(keyAction('w', IDS, { show: 'widths' })).toEqual({});
	});

	it('sets the width with 1, 2 and 3', () => {
		expect(keyAction('1', IDS, {})).toEqual({ width: 'mobile' });
		expect(keyAction('2', IDS, {})).toEqual({ width: 'tablet' });
		expect(keyAction('3', IDS, { width: 'mobile' })).toEqual({});
	});

	it('leaves the width alone in Widths, where every width is showing', () => {
		expect(keyAction('1', IDS, { show: 'widths' })).toBeUndefined();
		expect(keyAction('f', IDS, { show: 'widths' })).toBeUndefined();
	});

	it('turns the frames over with T', () => {
		expect(keyAction('t', IDS, {})).toEqual({ theme: 'light' });
		expect(keyAction('t', IDS, { theme: 'light' })).toEqual({});
	});

	it('walks the row sizes with F', () => {
		expect(keyAction('f', IDS, {})).toEqual({ scale: 0.5 });
		expect(keyAction('f', IDS, { scale: 0.5 })).toEqual({ scale: 1 });
		expect(keyAction('f', IDS, { scale: 1 })).toEqual({});
	});

	it('walks the clock with M, ending on off and coming back to real time', () => {
		expect(keyAction('m', IDS, {})).toEqual({ rate: 0.5 });
		expect(keyAction('m', IDS, { rate: 0.5 })).toEqual({ rate: 0.2 });
		expect(keyAction('m', IDS, { rate: 0.2 })).toEqual({ rate: 0 });
		expect(keyAction('m', IDS, { rate: 0 })).toEqual({});
	});

	it('outlines the parts with P, where the page has parts to outline', () => {
		expect(keyAction('p', IDS, {}, { parts: true })).toEqual({ parts: 'on' });
		expect(keyAction('P', IDS, { parts: 'on' }, { parts: true })).toEqual({});
		expect(keyAction('p', IDS, {})).toBeUndefined();
	});

	it('opens the notes with N, where the page has notes to open', () => {
		expect(keyAction('n', IDS, {}, { notes: true })).toEqual({ notes: 'on' });
		expect(keyAction('N', IDS, { notes: 'on' }, { notes: true })).toEqual({});
		expect(keyAction('n', IDS, {})).toBeUndefined();
	});

	it('leaves the mode and the row size alone on a page that has neither switch', () => {
		expect(keyAction('w', IDS, {}, { modes: false })).toBeUndefined();
		expect(keyAction('f', IDS, {}, { modes: false })).toBeUndefined();
	});

	it('answers nothing for a key it does not own', () => {
		expect(keyAction('x', IDS, {})).toBeUndefined();
		expect(keyAction('Enter', IDS, {})).toBeUndefined();
	});
});
