import type { Hold, HoldSource } from '@app/ui/views/frame-types';

/** A hold the page keeps and the frame follows: the frame reads a plain one only once. */
export function createHoldSource(): HoldSource & { set: (hold: Hold | null) => void } {
	let hold: Hold | null = null;
	const listeners = new Set<() => void>();
	return {
		get: () => hold,
		subscribe: listener => {
			listeners.add(listener);
			return () => {
				listeners.delete(listener);
			};
		},
		set: next => {
			hold = next;
			for (const listener of listeners) listener();
		},
	};
}
