import type { App } from '@app/api';
import { treaty, type Treaty } from '@elysiajs/eden';
import { getApiBaseUrl } from '@/env';
import { tokenStorage } from '@/lib/token-storage';

let onUnauthorized: (() => void) | null = null;
export function setOnUnauthorized(fn: () => void) {
	onUnauthorized = fn;
}

const baseUrl = getApiBaseUrl();

export const client: Treaty.Create<App> = treaty<App>(baseUrl, {
	headers: {
		get Authorization() {
			const token = tokenStorage.getToken();
			return token ? `Bearer ${token}` : '';
		},
	},
	onResponse(response) {
		if (response.url.includes('/api/auth/login')) return;
		if (response.status === 401) {
			onUnauthorized?.();
		}
	},
});

/**
 * Ask the server to wake: the request starts a machine that was put to sleep, and the server wakes
 * its database. Nothing is drawn from the answer and a failure is left alone — the reads that
 * follow say what is wrong.
 */
export function wakeServer(): void {
	void client.api.wake.get().catch(() => undefined);
}
