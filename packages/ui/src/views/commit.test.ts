import { describe, expect, it } from 'vitest';

import {
	NOTHING_CHANGED,
	changesFact,
	holdParts,
	isDirty,
	isSavable,
	leaveReason,
	owedSentence,
	type Commit,
} from './commit';

const COMMITS: Commit[] = ['clean', 'ready', 'blocked', 'saving'];

describe('what a record owes', () => {
	it('joins one, two and three clauses the way English does', () => {
		const due = { section: 'When it’s due', wants: 'the day it’s due' };
		const item = { section: 'Checklist', wants: 'an item on the checklist' };
		const name = { section: 'New project', wants: 'a name' };

		expect(owedSentence([])).toBe('');
		expect(owedSentence([item])).toBe('Needs an item on the checklist.');
		expect(owedSentence([name, item])).toBe('Needs a name and an item on the checklist.');
		expect(owedSentence([name, due, item])).toBe(
			'Needs a name, the day it’s due, and an item on the checklist.'
		);
	});
});

describe('the words', () => {
	it('counts changes as one string, so the box and the page cannot spell it apart', () => {
		expect(changesFact(3)).toBe('3 unsaved');
	});
});

describe('one control, one surface', () => {
	it('offers a commit only where the record could take one', () => {
		expect(COMMITS.filter(commit => holdParts(commit).commit)).toEqual(['ready']);
		expect(COMMITS.filter(commit => isSavable(commit))).toEqual(['ready']);
	});

	it('holds nothing on a clean record, and something in every other condition', () => {
		expect(isDirty('clean')).toBe(false);
		expect(COMMITS.filter(commit => holdParts(commit).discard)).toEqual([
			'ready',
			'blocked',
			'saving',
		]);
	});
});

describe('leaveReason', () => {
	it('turns the leaver off on a record nothing has changed on, whichever word it carries', () => {
		expect(leaveReason('clean')).toBe(NOTHING_CHANGED);
	});

	it('leaves it on whenever something is held', () => {
		expect(leaveReason('ready')).toBeNull();
		expect(leaveReason('blocked')).toBeNull();
		expect(leaveReason('saving')).toBeNull();
	});
});
