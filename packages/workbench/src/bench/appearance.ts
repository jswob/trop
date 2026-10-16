import { useEffect, useState } from 'react';

export type WorkbenchAppearance = 'light' | 'dark';

const APPEARANCE_KEY = 'workbench-appearance';
const SIDEBAR_KEY = 'workbench-sidebar';

/** The workbench's own appearance: what was chosen on this device, or what the device prefers. */
export function resolveAppearance(
	stored: string | null,
	prefersDark: boolean
): WorkbenchAppearance {
	if (stored === 'light' || stored === 'dark') return stored;
	return prefersDark ? 'dark' : 'light';
}

/** Whether the sidebar was left folded on this device. */
export function resolveFolded(stored: string | null): boolean {
	return stored === 'folded';
}

function read(key: string): string | null {
	try {
		return window.localStorage.getItem(key);
	} catch {
		return null;
	}
}

function write(key: string, value: string): void {
	try {
		window.localStorage.setItem(key, value);
	} catch {
		// A device that refuses storage keeps the choice for this visit only.
	}
}

const prefersDark = (): boolean =>
	typeof window.matchMedia === 'function' &&
	window.matchMedia('(prefers-color-scheme: dark)').matches;

/**
 * The workbench's own light and dark.
 *
 * It is not the appearance the frames are drawn in, which is an option of the stage and lives in
 * the address. This one is a preference of the person looking, so it follows their device until
 * they choose, and the choice stays on the device.
 */
export function useWorkbenchAppearance(): [
	WorkbenchAppearance,
	(next: WorkbenchAppearance) => void,
] {
	const [appearance, setAppearance] = useState<WorkbenchAppearance>(() =>
		resolveAppearance(read(APPEARANCE_KEY), prefersDark())
	);

	useEffect(() => {
		if (typeof window.matchMedia !== 'function') return;

		const media = window.matchMedia('(prefers-color-scheme: dark)');
		const follow = () => setAppearance(resolveAppearance(read(APPEARANCE_KEY), media.matches));

		media.addEventListener('change', follow);
		return () => media.removeEventListener('change', follow);
	}, []);

	return [
		appearance,
		next => {
			write(APPEARANCE_KEY, next);
			setAppearance(next);
		},
	];
}

/** Whether the sidebar is folded away, remembered on the device. */
export function useFolded(): [boolean, () => void] {
	const [folded, setFolded] = useState(() => resolveFolded(read(SIDEBAR_KEY)));

	return [
		folded,
		() => {
			write(SIDEBAR_KEY, folded ? 'open' : 'folded');
			setFolded(!folded);
		},
	];
}
