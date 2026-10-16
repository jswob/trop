import { describe, expect, it } from 'vitest';

import {
	groundView,
	kitEntryById,
	protoById,
	stageEntries,
	stageEntryById,
	stageKindOf,
	undrawnSections,
	viewById,
	viewsOfSection,
} from './registry';

describe('the registry', () => {
	it('stages the places, the candidates, the stress sheet and the kit’s states, in that order', () => {
		expect(stageEntries().map(entry => entry.id)).toEqual([
			'overview',
			'ledger-list',
			'ledger-entry',
			'entry-split',
			'kit-stress',
			'kit-mark',
		]);
	});

	it('looks an entry up by id across spaces', () => {
		expect(stageEntryById('ledger-entry')?.id).toBe('ledger-entry');
		expect(stageEntryById('entry-split')?.id).toBe('entry-split');
		expect(stageEntryById('kit-stress')?.id).toBe('kit-stress');
		expect(stageEntryById('not-here')).toBeUndefined();
	});

	it('names the space an entry came from, for the address its frame carries', () => {
		expect(stageKindOf('overview')).toBe('view');
		expect(stageKindOf('entry-split')).toBe('proto');
		expect(stageKindOf('kit-stress')).toBe('kit');
		expect(stageKindOf('kit-mark')).toBe('kit');
	});

	it('finds the ground: the one place under no section', () => {
		expect(groundView()?.id).toBe('overview');
	});

	it('lists a section’s places and names the sections with none', () => {
		expect(viewsOfSection('ledger').map(view => view.id)).toEqual(['ledger-list', 'ledger-entry']);
		expect(undrawnSections().map(section => section.id)).toEqual(['people']);
	});

	it('looks a place, a candidate and a kit entry up by id', () => {
		expect(viewById('ledger-list')?.title).toBe('Ledger');
		expect(viewById('entry-split')).toBeUndefined();
		expect(protoById('entry-split')?.title).toBe('Split entry');
		expect(kitEntryById('mark')?.name).toBe('Mark');
	});
});
