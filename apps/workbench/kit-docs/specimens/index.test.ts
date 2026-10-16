import { describe, expect, it } from 'vitest';

import { KIT_ENTRIES } from '../index';

import { SPECIMENS } from './index';

/** A component enters the kit with its page: the catalogue and the pages list the same things. */
describe('the kit’s pages', () => {
	it('are one a component, in the catalogue’s order', () => {
		expect(SPECIMENS.map(spec => spec.id)).toEqual(KIT_ENTRIES.map(entry => entry.id));
	});

	it.each(SPECIMENS.map(spec => [spec.id, spec] as const))(
		'%s draws a state and says what it holds and refuses',
		(_id, spec) => {
			expect(spec.states.length).toBeGreaterThan(0);
			expect(spec.holds.length).toBeGreaterThan(0);
			expect(spec.refuses.length).toBeGreaterThan(0);
		}
	);

	it.each(SPECIMENS.map(spec => [spec.id, spec] as const))(
		'%s says what every prop is',
		(_id, spec) => {
			for (const prop of spec.props) expect(prop.is.length, prop.name).toBeGreaterThan(0);
		}
	);
});
