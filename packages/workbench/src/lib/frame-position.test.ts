import { act, renderHook } from '@testing-library/react';
import { useEffect } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import {
	POSITION_PARAM,
	positionIn,
	readPosition,
	useFramePosition,
	writePosition,
} from './frame-position';

const FRAME = '/frame/view/order-modify?variant=edit&chrome=bare';

/** A write that lands also forgets a position an earlier test could only keep in memory. */
beforeEach(() => {
	writePosition(null);
	window.history.replaceState(null, '', FRAME);
});

describe('a frame’s position in its address', () => {
	it('is not the param the bench’s stage stands on', () => {
		expect(POSITION_PARAM).not.toBe('at');
	});

	it('is absent on an address that names none', () => {
		expect(readPosition()).toBeNull();
	});

	it('is read off the address', () => {
		window.history.replaceState(null, '', `${FRAME}&where=billing`);

		expect(readPosition()).toBe('billing');
	});

	it('is written beside the params the frame was opened with', () => {
		writePosition('billing');

		const search = new URLSearchParams(window.location.search);
		expect(search.get('where')).toBe('billing');
		expect(search.get('variant')).toBe('edit');
		expect(search.get('chrome')).toBe('bare');
		expect(window.location.pathname).toBe('/frame/view/order-modify');
	});

	it('replaces the entry rather than adding one, so back still leaves the frame', () => {
		const before = window.history.length;

		writePosition('billing');
		writePosition('people');

		expect(window.history.length).toBe(before);
	});

	it('keeps whatever state the entry already carried', () => {
		window.history.replaceState({ key: 'router' }, '', FRAME);

		writePosition('billing');

		expect(window.history.state).toEqual({ key: 'router' });
	});

	it('drops the fragment the page that was left had put on the address', () => {
		window.history.replaceState(null, '', `${FRAME}#people/person/p-ada`);

		writePosition('roles');

		expect(window.location.hash).toBe('');
		expect(readPosition()).toBe('roles');
	});

	it('leaves the address as it was opened once the position is cleared', () => {
		writePosition('billing');
		writePosition(null);

		expect(window.location.pathname + window.location.search).toBe(FRAME);
	});
});

describe('a position read off another document’s address', () => {
	it('is the one its search names', () => {
		expect(positionIn('?variant=edit&chrome=bare&where=billing')).toBe('billing');
	});

	it('is none where the search names none', () => {
		expect(positionIn('?variant=edit&chrome=bare')).toBeNull();
		expect(positionIn('')).toBeNull();
	});
});

describe('useFramePosition', () => {
	it('hands a frame its position and follows every write, without a remount', () => {
		let mounts = 0;
		const { result } = renderHook(() => {
			useEffect(() => {
				mounts += 1;
			}, []);
			return useFramePosition();
		});

		expect(result.current[0]).toBeNull();

		act(() => {
			result.current[1]('billing');
		});

		expect(result.current[0]).toBe('billing');
		expect(readPosition()).toBe('billing');
		expect(mounts).toBe(1);
	});

	it('tells every reader on the document, not only the one that wrote', () => {
		const writer = renderHook(() => useFramePosition());
		const reader = renderHook(() => useFramePosition());

		act(() => {
			writer.result.current[1]('people');
		});

		expect(reader.result.current[0]).toBe('people');
	});

	it('follows the browser’s own back and forward', () => {
		const { result } = renderHook(() => useFramePosition());

		act(() => {
			window.history.replaceState(null, '', `${FRAME}&where=roles`);
			window.dispatchEvent(new PopStateEvent('popstate'));
		});

		expect(result.current[0]).toBe('roles');
	});

	it('keeps the position in memory where the document’s address cannot be written', () => {
		const refused = vi.spyOn(window.history, 'replaceState').mockImplementation(() => {
			throw new DOMException('refused', 'SecurityError');
		});
		const { result } = renderHook(() => useFramePosition());

		act(() => {
			result.current[1]('billing');
		});
		refused.mockRestore();

		expect(result.current[0]).toBe('billing');
		expect(new URLSearchParams(window.location.search).get('where')).toBeNull();
	});
});
