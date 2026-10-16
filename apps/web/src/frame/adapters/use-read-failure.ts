import type { FrameReadFailed } from '@app/ui/views/frame';
import { useQueryClient, type Query, type QueryKey } from '@tanstack/react-query';
import { useCallback, useMemo, useSyncExternalStore } from 'react';

/**
 * Whether one query is a read that failed: something on screen is reading it, and
 * it ended in a server error or a network failure, after its retries. A 4xx is never one — a 401
 * already signs out, and a 404 is the page's own case, as an unknown record is.
 */
function isFailedRead(query: Query): boolean {
	if (query.state.status !== 'error' || query.getObserversCount() === 0) return false;
	const status = (query.state.error as { status?: unknown } | null)?.status;
	return typeof status !== 'number' || status >= 500;
}

/** Whether a key starts with one of the prefixes, element by element. */
function startsWithAny(key: QueryKey, prefixes: readonly QueryKey[]): boolean {
	return prefixes.some(prefix => prefix.every((part, index) => key[index] === part));
}

/**
 * **A read that failed, as the frame draws it**, or `null` while none has.
 *
 * Read off TanStack Query's cache rather than reported by each adapter, so every page — and every
 * page built later — is covered by doing nothing. `ground` when one of the failed reads is the
 * sheet's own (the `ground` key prefixes), so the frame gives way to the centred screen; otherwise
 * the waits on screen say it. `retrying` while the failed reads are asked again, and `retry` asks
 * again for them alone.
 * @param ground - The key prefixes of the ground's own reads (`GROUND_READS`)
 */
export function useReadFailure(ground: readonly QueryKey[]): FrameReadFailed | null {
	const client = useQueryClient();
	const cache = client.getQueryCache();

	const subscribe = useCallback((change: () => void) => cache.subscribe(change), [cache]);
	/** A string, so an unchanged cache reads as the same snapshot. */
	const read = useCallback(() => {
		const failed = cache.findAll({ predicate: isFailedRead });
		if (failed.length === 0) return '';
		const onGround = failed.some(query => startsWithAny(query.queryKey, ground));
		const retrying = failed.some(query => query.state.fetchStatus === 'fetching');
		return `${onGround ? 'ground' : 'page'}:${retrying ? 'retrying' : 'still'}`;
	}, [cache, ground]);
	const state = useSyncExternalStore(subscribe, read, read);

	const retry = useCallback(() => {
		void client.refetchQueries({ predicate: isFailedRead });
	}, [client]);

	return useMemo(() => {
		if (state === '') return null;
		const [whose, doing] = state.split(':');
		return { ground: whose === 'ground', retrying: doing === 'retrying', retry };
	}, [state, retry]);
}
