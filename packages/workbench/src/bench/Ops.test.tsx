import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { Ops } from './Ops';
import type { BenchSearch } from './search';

const IDS = ['edit', 'holding', 'blocked', 'saving', 'deleting'];

function setup(search: BenchSearch = {}) {
	const onChange = vi.fn<(next: BenchSearch) => void>();
	render(<Ops ids={IDS} search={search} onChange={onChange} />);
	return { onChange, user: userEvent.setup() };
}

const group = (name: string) => screen.getByRole('group', { name });
const pressed = (name: string) =>
	within(group(name))
		.getAllByRole('button')
		.filter(button => button.getAttribute('aria-pressed') === 'true')
		.map(button => button.textContent);

describe('Ops', () => {
	it('draws the six switches, at rest', () => {
		setup();

		expect(pressed('Show')).toEqual(['States']);
		expect(pressed('States')).toEqual(['All 5A']);
		expect(pressed('Width')).toEqual(['Desktop3']);
		expect(pressed('Frames')).toEqual(['Dark']);
		expect(pressed('Across')).toEqual(['Fit']);
		expect(pressed('Motion')).toEqual(['Real time']);
	});

	it('labels motion and size in words, never in codes', () => {
		setup();

		expect(within(group('Motion')).getByRole('button', { name: 'A fifth' })).toBeTruthy();
		expect(within(group('Across')).getByRole('button', { name: 'Two' })).toBeTruthy();
		expect(screen.queryByText('5×')).toBeNull();
		expect(screen.queryByText('50%')).toBeNull();
	});

	it('writes the frames’ appearance into the address', async () => {
		const { onChange, user } = setup({ width: 'mobile' });

		await user.click(within(group('Frames')).getByRole('button', { name: 'Light' }));

		expect(onChange).toHaveBeenCalledWith({ width: 'mobile', theme: 'light' });
	});

	it('leaves a default off the address', async () => {
		const { onChange, user } = setup({ width: 'mobile', theme: 'light' });

		await user.click(within(group('Width')).getByRole('button', { name: /Desktop/ }));

		expect(onChange).toHaveBeenCalledWith({ theme: 'light' });
	});

	it('turns motion off as a rate of zero, and clears a pin', async () => {
		const { onChange, user } = setup({ t: 120 });

		await user.click(within(group('Motion')).getByRole('button', { name: 'Off' }));

		expect(onChange).toHaveBeenCalledWith({ rate: 0 });
	});

	it('steps to the next state', async () => {
		const { onChange, user } = setup({ variants: 'edit', at: 'edit' });

		await user.click(screen.getByRole('button', { name: 'Next state' }));

		expect(onChange).toHaveBeenCalledWith({ variants: 'holding', at: 'holding' });
	});

	it('says which state is showing once one is alone', () => {
		setup({ variants: 'holding', at: 'holding' });

		expect(within(group('States')).getByText('2 of 5')).toBeTruthy();
		expect(pressed('States')).toEqual([]);
	});

	it('says how many are showing once some are hidden', () => {
		setup({ variants: 'edit,holding,saving' });

		expect(within(group('States')).getByText('3 of 5')).toBeTruthy();
	});

	it('drops the width and size switches in Widths, where every width is showing', () => {
		setup({ show: 'widths', at: 'blocked' });

		expect(pressed('Show')).toEqual(['Widths']);
		expect(screen.queryByRole('group', { name: 'Width' })).toBeNull();
		expect(screen.queryByRole('group', { name: 'Across' })).toBeNull();
		expect(within(group('States')).getByText('3 of 5')).toBeTruthy();
	});

	it('offers the parts switch only where the page has parts, and writes it to the address', async () => {
		const onChange = vi.fn<(next: BenchSearch) => void>();
		const { unmount } = render(<Ops ids={IDS} search={{}} onChange={onChange} />);
		expect(screen.queryByRole('group', { name: 'Parts' })).toBeNull();
		unmount();

		render(<Ops ids={IDS} search={{}} onChange={onChange} parts />);
		await userEvent.setup().click(within(group('Parts')).getByRole('button', { name: 'On' }));

		expect(onChange).toHaveBeenCalledWith({ parts: 'on' });
	});

	it('offers the notes switch only where the page has notes, and writes it to the address', async () => {
		const onChange = vi.fn<(next: BenchSearch) => void>();
		const { unmount } = render(<Ops ids={IDS} search={{}} onChange={onChange} />);
		expect(screen.queryByRole('group', { name: 'Notes' })).toBeNull();
		unmount();

		render(<Ops ids={IDS} search={{}} onChange={onChange} notes />);
		await userEvent.setup().click(within(group('Notes')).getByRole('button', { name: 'On' }));

		expect(onChange).toHaveBeenCalledWith({ notes: 'on' });
	});

	it('drops the mode and row-size switches for a component drawn at its own size', () => {
		render(<Ops ids={IDS} search={{}} onChange={vi.fn()} modes={false} parts />);

		expect(screen.queryByRole('group', { name: 'Show' })).toBeNull();
		expect(screen.queryByRole('group', { name: 'Across' })).toBeNull();
		expect(pressed('Width')).toEqual(['Desktop3']);
	});

	it('goes to Widths and back', async () => {
		const { onChange, user } = setup({ theme: 'light' });

		await user.click(within(group('Show')).getByRole('button', { name: 'Widths' }));

		expect(onChange).toHaveBeenCalledWith({ theme: 'light', show: 'widths' });
	});
});
