export interface QueryCounter {
	/** Pass as the postgres.js client's `debug` option; it is called once per statement sent. */
	debug: () => void;
	/**
	 * Count the statements `run` sends, so a test can pin how many queries a read or a save makes —
	 * and that the number stays the same when there is more data:
	 *
	 * ```ts
	 * const queries = await countQueries(() => listItemsWithTags(tx));
	 * expect(queries).toBe(2);
	 * ```
	 *
	 * Counts everything sent on the client while `run` is pending (savepoints included), so keep
	 * fixture creation outside of it.
	 * @param run - The code under test
	 */
	countQueries: (run: () => Promise<unknown>) => Promise<number>;
}

/** A counter of the statements one postgres.js client sends. */
export function createQueryCounter(): QueryCounter {
	let sent = 0;
	return {
		debug: () => {
			sent += 1;
		},
		countQueries: async run => {
			const before = sent;
			await run();
			return sent - before;
		},
	};
}
