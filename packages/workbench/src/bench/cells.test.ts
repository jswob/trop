import { describe, expect, it } from 'vitest';

import type { Subject } from '../lib/subject';

import { listCells, stateCells } from './cells';

const subject = (id: string, states: string[]): Subject => ({
	id,
	title: `Title of ${id}`,
	description: '',
	chrome: 'bare',
	variants: states.map(state => ({
		id: state,
		label: `Label of ${state}`,
		note: `Why ${state}`,
		render: () => null,
	})),
});

const MODIFY = subject('order-modify', ['edit', 'holding']);
const TALLY = subject('tally', ['resting']);

const find = (id: string): Subject | undefined => [MODIFY, TALLY].find(entry => entry.id === id);
const kindOf = (id: string) => (id === 'tally' ? 'proto' : 'view');

describe('stateCells', () => {
	it('is one cell a state, under the state’s own id, label and note', () => {
		expect(stateCells('view', MODIFY)).toEqual([
			{
				id: 'edit',
				label: 'Label of edit',
				note: 'Why edit',
				kind: 'view',
				subject: 'order-modify',
				variant: 'edit',
				chrome: 'bare',
			},
			{
				id: 'holding',
				label: 'Label of holding',
				note: 'Why holding',
				kind: 'view',
				subject: 'order-modify',
				variant: 'holding',
				chrome: 'bare',
			},
		]);
	});
});

describe('listCells', () => {
	it('puts frames of two spaces side by side, each named by its subject and its state', () => {
		const { cells, missing } = listCells(
			[
				{ subject: 'tally', variant: 'resting' },
				{ subject: 'order-modify', variant: 'holding' },
			],
			find,
			kindOf
		);

		expect(missing).toEqual([]);
		expect(cells).toEqual([
			{
				id: 'tally:resting',
				label: 'Title of tally',
				note: 'Label of resting',
				kind: 'proto',
				subject: 'tally',
				variant: 'resting',
				chrome: 'bare',
			},
			{
				id: 'order-modify:holding',
				label: 'Title of order-modify',
				note: 'Label of holding',
				kind: 'view',
				subject: 'order-modify',
				variant: 'holding',
				chrome: 'bare',
			},
		]);
	});

	it('draws a subject’s first state where the list names none, or one it does not have', () => {
		const { cells } = listCells(
			[{ subject: 'order-modify' }, { subject: 'tally', variant: 'gone' }],
			find,
			kindOf
		);

		expect(cells.map(cell => cell.id)).toEqual(['order-modify:edit', 'tally:resting']);
	});

	it('names what nobody registered, and draws the rest', () => {
		const { cells, missing } = listCells(
			[{ subject: 'nowhere', variant: 'x' }, { subject: 'tally' }],
			find,
			kindOf
		);

		expect(missing).toEqual(['nowhere']);
		expect(cells.map(cell => cell.id)).toEqual(['tally:resting']);
	});

	it('draws a frame once, however often the list names it', () => {
		const { cells } = listCells([{ subject: 'tally' }, { subject: 'tally' }], find, kindOf);

		expect(cells).toHaveLength(1);
	});
});
