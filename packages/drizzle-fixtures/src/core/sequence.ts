/**
 * Sequence counter utilities for generating incrementing numbers per builder instance.
 *
 * Used to provide unique sequence numbers to field resolvers (e.g., for generating
 * unique emails like "user-1@example.com", "user-2@example.com").
 */

/**
 * Creates a function that returns incrementing sequence numbers.
 * Each call returns the next number in sequence.
 *
 * @param startAt - Starting number (first call returns this value). Defaults to 1.
 * @returns Function that returns the next sequence number
 *
 * @example
 * ```ts
 * const getSeq = createSequenceCounter(1);
 * getSeq(); // 1
 * getSeq(); // 2
 * getSeq(); // 3
 * ```
 */
export function createSequenceCounter(startAt: number = 1): () => number {
	let current = startAt - 1;
	return () => ++current;
}

/**
 * Class-based sequence counter with additional control methods.
 *
 * @example
 * ```ts
 * const seq = new SequenceCounter(1);
 * seq.next(); // 1
 * seq.next(); // 2
 * seq.current(); // 2 (peek at current value)
 * seq.reset(10);
 * seq.next(); // 10
 * ```
 */
export class SequenceCounter {
	private _current: number;
	private _startAt: number;

	constructor(startAt: number = 1) {
		this._startAt = startAt;
		this._current = startAt - 1;
	}

	/**
	 * Returns the next sequence number and increments the counter.
	 */
	next(): number {
		return ++this._current;
	}

	/**
	 * Returns the current sequence number without incrementing.
	 * Returns the initial value minus 1 if next() hasn't been called yet.
	 */
	current(): number {
		return this._current;
	}

	/**
	 * Resets the counter to start fresh from the given number.
	 *
	 * @param startAt - New starting number (next call to next() returns this). Defaults to 1.
	 */
	reset(startAt: number = 1): void {
		this._startAt = startAt;
		this._current = startAt - 1;
	}

	/**
	 * Returns the configured starting value.
	 */
	startAt(): number {
		return this._startAt;
	}
}
