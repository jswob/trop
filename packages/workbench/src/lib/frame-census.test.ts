import { describe, expect, it } from 'vitest';

import type { Subject } from '../lib/subject';
import { frameCensus } from './frame-census';
import type { FrameKind } from './frame-url';

const subject = (id: string, chrome: Subject['chrome'], variants: string[]): Subject => ({
	id,
	title: `${id} title`,
	description: '',
	chrome,
	variants: variants.map(variant => ({
		id: variant,
		label: `${variant} label`,
		render: () => null,
	})),
});

const kindOf = (id: string): FrameKind => (id === 'a-proto' ? 'proto' : 'view');

describe('frameCensus', () => {
	it('flattens every variant to its own address', () => {
		const census = frameCensus([subject('a-proto', 'bare', ['one', 'two'])], kindOf);

		expect(census).toHaveLength(2);
		expect(census.map(entry => entry.variant)).toEqual(['one', 'two']);
	});

	it('carries the space and the chrome a frame URL needs', () => {
		const census = frameCensus([subject('a-proto', 'bare', ['one'])], kindOf);

		expect(census[0]).toMatchObject({ id: 'a-proto', kind: 'proto', chrome: 'bare' });
	});

	it('crosses spaces in one list, so entries from two sit side by side', () => {
		const census = frameCensus(
			[subject('a-proto', 'bare', ['one']), subject('a-view', 'bare', ['one'])],
			kindOf
		);

		expect(census.map(entry => entry.kind)).toEqual(['proto', 'view']);
	});

	it('is empty when nothing is registered', () => {
		expect(frameCensus([], kindOf)).toEqual([]);
	});
});
