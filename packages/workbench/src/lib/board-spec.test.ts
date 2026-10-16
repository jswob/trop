import { describe, expect, it } from 'vitest';

import { parseFrameSpec } from './board-spec';

describe('parseFrameSpec', () => {
	it('reads an id and a state the way drive writes them', () => {
		expect(parseFrameSpec('people:person')).toEqual({
			id: 'people',
			variant: 'person',
			theme: 'dark',
		});
	});

	it('leaves the state out when only the id is named', () => {
		expect(parseFrameSpec('timeline-sheet')).toEqual({ id: 'timeline-sheet', theme: 'dark' });
	});

	it('takes the three params the workbench frame address takes', () => {
		expect(parseFrameSpec('people:person?theme=light&rate=0&t=120')).toEqual({
			id: 'people',
			variant: 'person',
			theme: 'light',
			rate: 0,
			t: 120,
		});
	});

	it('takes the place a frame has walked to, as the frame address writes it', () => {
		expect(parseFrameSpec('people:grid?where=teams')).toEqual({
			id: 'people',
			variant: 'grid',
			theme: 'dark',
			where: 'teams',
		});
	});

	it('drops a position that could not be a place', () => {
		expect(parseFrameSpec('people?where=Not a place')).toEqual({ id: 'people', theme: 'dark' });
		expect(parseFrameSpec('people?where=')).toEqual({ id: 'people', theme: 'dark' });
	});

	it('reads any appearance but light as dark', () => {
		expect(parseFrameSpec('people?theme=sepia')?.theme).toBe('dark');
	});

	it('drops a speed or a moment outside the workbench bounds', () => {
		expect(parseFrameSpec('people?rate=3&t=-1')).toEqual({ id: 'people', theme: 'dark' });
		expect(parseFrameSpec('people?rate=fast')).toEqual({ id: 'people', theme: 'dark' });
	});

	it('names nothing when the frame carries no name or a malformed one', () => {
		expect(parseFrameSpec('')).toBeNull();
		expect(parseFrameSpec('   ')).toBeNull();
		expect(parseFrameSpec('People Page')).toBeNull();
		expect(parseFrameSpec(':person')).toBeNull();
	});
});
