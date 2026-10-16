import { useEffect, useState, type RefObject } from 'react';

type Size = { width: number; height: number };

/**
 * The live content-box size of an element. Returns zeroes until the first
 * measurement, which callers treat as "not measured yet" rather than "empty".
 */
export function useElementSize(ref: RefObject<HTMLElement | null>): Size {
	const [size, setSize] = useState<Size>({ width: 0, height: 0 });

	useEffect(() => {
		const element = ref.current;
		if (!element) return;

		const observer = new ResizeObserver(entries => {
			const box = entries[0]?.contentRect;
			if (box) setSize({ width: box.width, height: box.height });
		});

		observer.observe(element);
		return () => observer.disconnect();
	}, [ref]);

	return size;
}
