import { render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import type { Subject } from '../lib/subject';
import { FrameDocument } from './FrameDocument';

const entry: Subject = {
	id: 'sample',
	title: 'Sample',
	description: 'Two states of nothing in particular.',
	chrome: 'bare',
	variants: [
		{ id: 'resting', label: 'Resting', render: () => <p>resting drawn</p> },
		{ id: 'busy', label: 'Busy', render: () => <p>busy drawn</p> },
	],
};

const find = (id: string): Subject | undefined => (id === entry.id ? entry : undefined);
const missing = (what: 'entry' | 'variant', id: string, of?: string) => (
	<p>{`no ${what} ${id}${of === undefined ? '' : ` of ${of}`}`}</p>
);

afterEach(() => {
	delete document.documentElement.dataset['theme'];
});

describe('FrameDocument', () => {
	it('draws the named state of the named entry', () => {
		render(<FrameDocument id="sample" variant="busy" find={find} missing={missing} />);

		expect(screen.getByText('busy drawn')).toBeTruthy();
	});

	it('draws the first state when none is named', () => {
		render(<FrameDocument id="sample" find={find} missing={missing} />);

		expect(screen.getByText('resting drawn')).toBeTruthy();
	});

	it('writes the appearance onto the document it draws in', () => {
		render(<FrameDocument id="sample" theme="light" find={find} missing={missing} />);

		expect(document.documentElement.dataset['theme']).toBe('light');
	});

	it('says which name found nothing, an entry or one of its states', () => {
		const { rerender } = render(<FrameDocument id="gone" find={find} missing={missing} />);
		expect(screen.getByText('no entry gone')).toBeTruthy();

		rerender(<FrameDocument id="sample" variant="lost" find={find} missing={missing} />);
		expect(screen.getByText('no variant lost of sample')).toBeTruthy();
	});
});
