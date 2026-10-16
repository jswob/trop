import { act, cleanup, renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Nest } from './frame-types';
import { useNest, type NestControl } from './nest';

/**
 * Which record a page has open inside itself, in both hands: the workbench's, where the nest keeps
 * it and writes the browser's history, and the app's, where the address keeps it.
 */

afterEach(cleanup);

const NAMES: Record<string, string> = { 'n-groceries': 'Groceries', 'n-errands': 'Errands' };

const NEST: Nest = {
	kind: 'note',
	name: id => NAMES[id] ?? null,
	lede: '',
	page: () => null,
};

describe('uncontrolled, as the workbench draws it', () => {
	beforeEach(() => {
		window.history.replaceState(null, '', '/');
	});

	it('opens a nested link on the history, and closes back through it', () => {
		const { result } = renderHook(() => useNest(NEST, 'notes', 'notes'));

		act(() => {
			expect(result.current.follow('#notes/note/n-groceries')).toBe(true);
		});

		expect(result.current.open).toEqual({ id: 'n-groceries', name: 'Groceries' });
		expect(window.location.hash).toBe('#notes/note/n-groceries');
	});
});

describe('controlled, as the app holds it', () => {
	it('reads the open record from its hand and never writes the history', () => {
		const push = vi.spyOn(window.history, 'pushState');
		const onOpen = vi.fn<NestControl['onOpen']>();
		const closed: NestControl = { open: null, onOpen };
		const { result, rerender } = renderHook(
			({ control }: { control: NestControl }) =>
				useNest(NEST, 'notes', 'notes', undefined, control),
			{ initialProps: { control: closed } }
		);

		expect(result.current.open).toBeNull();
		act(() => {
			expect(result.current.follow('#notes/note/n-errands')).toBe(true);
		});
		expect(onOpen).toHaveBeenLastCalledWith('n-errands');
		expect(result.current.open).toBeNull();

		rerender({ control: { open: 'n-errands', onOpen } });
		expect(result.current.open).toEqual({ id: 'n-errands', name: 'Errands' });

		act(() => {
			result.current.close();
		});
		expect(onOpen).toHaveBeenLastCalledWith(null);
		expect(push).not.toHaveBeenCalled();
	});

	it('follows the app’s own address form as it does the workbench’s', () => {
		const onOpen = vi.fn<NestControl['onOpen']>();
		const { result } = renderHook(() =>
			useNest(NEST, 'notes', 'notes', undefined, { open: null, onOpen })
		);

		act(() => {
			expect(result.current.follow('/notes/note/n-groceries')).toBe(true);
		});
		expect(onOpen).toHaveBeenLastCalledWith('n-groceries');
	});

	it('opens nothing it cannot name, and nothing while another page is open', () => {
		const onOpen = vi.fn<NestControl['onOpen']>();
		const { result: unknown } = renderHook(() =>
			useNest(NEST, 'notes', 'notes', undefined, { open: 'n-gone', onOpen })
		);
		const { result: elsewhere } = renderHook(() =>
			useNest(NEST, 'notes', 'account', undefined, { open: 'n-groceries', onOpen })
		);

		expect(unknown.current.open).toBeNull();
		expect(elsewhere.current.open).toBeNull();
		expect(elsewhere.current.follow('#notes/note/n-groceries')).toBe(false);
	});
});
