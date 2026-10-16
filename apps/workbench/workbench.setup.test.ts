import { DESCRIPTION_CAP, NOTE_CAP, settledSubjectsOf, subjectsOf } from '@jswob/design-workbench';
import { describe, expect, it } from 'vitest';

import setup from './workbench.setup';

const STAGED = subjectsOf(setup);

describe('what the setup stages', () => {
	it('gives every entry an id unique across the spaces', () => {
		const ids = STAGED.map(entry => entry.id);
		expect(new Set(ids).size).toBe(ids.length);
	});

	it('stages a destination’s place, the ground and the kit’s stress sheet', () => {
		const ids = new Set(STAGED.map(entry => entry.id));

		expect(ids.has('notes')).toBe(true);
		expect(ids.has('home')).toBe(true);
		expect(ids.has('kit-stress')).toBe(true);
		expect(ids.has('not-here')).toBe(false);
	});

	it('gives every entry at least one variant', () => {
		for (const entry of STAGED) expect(entry.variants.length).toBeGreaterThan(0);
	});
});

/**
 * Proto is left out: a candidate's description is its bet, as long as the bet is, until it settles.
 */
const SETTLED = settledSubjectsOf(setup);

describe('the words a settled subject is drawn with', () => {
	it.each(SETTLED.map(subject => [subject.id, subject] as const))(
		'%s says what it is in at most 100 characters',
		(_id, subject) => {
			expect(subject.description.length).toBeGreaterThan(0);
			expect(subject.description.length).toBeLessThanOrEqual(DESCRIPTION_CAP);
		}
	);

	it.each(
		SETTLED.flatMap(subject =>
			subject.variants.map(variant => [`${subject.id}:${variant.id}`, variant.note] as const)
		)
	)('%s says why it is listed in at most 60 characters', (_name, note) => {
		expect(note, 'no note written').toBeDefined();
		expect(note?.length).toBeGreaterThan(0);
		expect(note?.length).toBeLessThanOrEqual(NOTE_CAP);
	});

	it('holds the caps the brief agreed', () => {
		expect(DESCRIPTION_CAP).toBe(100);
		expect(NOTE_CAP).toBe(60);
	});
});
