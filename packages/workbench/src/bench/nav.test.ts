import { describe, expect, it } from 'vitest';

import { jumpEntries, matchJump, navGroups, type NavInput } from './nav';

const INPUT: NavInput = {
	ground: { id: 'calendar-board', title: 'Calendar — the board', states: 5, label: 'Calendar' },
	destinations: [
		{
			id: 'order',
			label: 'Order',
			places: [
				{ id: 'order-create', title: 'Create order', states: 3 },
				{ id: 'order-modify', title: 'Modify order', states: 5 },
			],
		},
		{
			id: 'billing',
			label: 'Billing',
			places: [{ id: 'billing', title: 'Billing — configure', states: 11 }],
		},
		{ id: 'stock', label: 'Stock', places: [] },
	],
	kit: [
		{ id: 'table', name: 'Table' },
		{ id: 'plinth', name: 'Plinth' },
	],
	protos: [],
};

describe('navGroups', () => {
	const groups = navGroups(INPUT);

	it('has the four spaces, in order', () => {
		expect(groups.map(group => group.title)).toEqual(['Tokens', 'Kit', 'Views', 'Proto']);
	});

	it('counts the kit’s components on the row that lists them', () => {
		expect(groups[1]?.rows[0]).toMatchObject({ label: 'Components', count: 2, to: '/kit' });
	});

	it('lists the components under that row, each on its own page', () => {
		expect(groups[1]?.rows[0]?.children).toEqual([
			{ label: 'Table', to: '/kit/table' },
			{ label: 'Plinth', to: '/kit/plinth' },
		]);
	});

	it('opens the tokens on the bench', () => {
		expect(groups[0]?.rows[0]).toMatchObject({ to: '/tokens' });
	});

	it('opens the group with every place on one page, counted', () => {
		expect(groups[2]?.rows[0]).toEqual({ label: 'Places', to: '/views', count: 4 });
	});

	it('opens the ground as Calendar, with its states counted', () => {
		expect(groups[2]?.rows[1]).toMatchObject({
			label: 'Calendar',
			count: 5,
			to: '/views/calendar-board',
		});
	});

	it('folds a destination with several places, and lists them under it', () => {
		const order = groups[2]?.rows[2];

		expect(order).toMatchObject({ label: 'Order', to: undefined });
		expect(order?.children?.map(child => child.label)).toEqual(['Create order', 'Modify order']);
		expect(order?.children?.[1]).toMatchObject({ count: 5, to: '/views/order-modify' });
	});

	it('makes a destination with one place a row of its own, under the destination’s name', () => {
		expect(groups[2]?.rows[3]).toMatchObject({
			label: 'Billing',
			count: 11,
			to: '/views/billing',
		});
	});

	it('leaves out a destination nobody has drawn', () => {
		expect(groups[2]?.rows.map(row => row.label)).toEqual([
			'Places',
			'Calendar',
			'Order',
			'Billing',
		]);
	});

	it('says in one line that nothing is unsettled', () => {
		expect(groups[3]).toMatchObject({ rows: [], empty: 'Nothing unsettled' });
	});
});

describe('jumpEntries and matchJump', () => {
	const entries = jumpEntries(INPUT);

	it('joins a component’s several names with commas, never a middle dot', () => {
		const dotted = jumpEntries({ ...INPUT, kit: [{ id: 'split', name: 'SplitShell · useSplit' }] });

		expect(dotted.at(-1)?.label).toBe('SplitShell, useSplit');
	});

	it('lists every place and every kit component', () => {
		expect(entries.map(entry => entry.label)).toEqual([
			'Calendar — the board',
			'Create order',
			'Modify order',
			'Billing — configure',
			'Table',
			'Plinth',
		]);
	});

	it('finds a place by a part of its name, whatever the case', () => {
		expect(matchJump(entries, 'MOD').map(entry => entry.label)).toEqual(['Modify order']);
	});

	it('finds a place by its id', () => {
		expect(matchJump(entries, 'calendar-bo').map(entry => entry.label)).toEqual([
			'Calendar — the board',
		]);
	});

	it('lists everything for an empty query', () => {
		expect(matchJump(entries, '  ')).toHaveLength(entries.length);
	});
});
