import { Outlet, useNavigate } from '@tanstack/react-router';
import { useEffect, useRef, useState } from 'react';

import './bench.css';
import { Palette } from './Palette';
import { Sidebar } from './Sidebar';
import { asSearch, useAddress } from './address';
import { useFolded, useWorkbenchAppearance } from './appearance';
import { ChromeContext, type Chrome } from './chrome';
import { carry } from './search';
import { formatTried, parseTried, type Tried } from './tokens';

/**
 * The bench: the workbench's own chrome.
 *
 * A sidebar that folds away, the page, and the jump list over both. It owns what is about the
 * person looking rather than about the stage: the workbench's own light and dark and whether the
 * sidebar is folded, both kept on the device. What is about the stage is in the address, and the
 * part of it a link to another place takes along is worked out here, once, for every link.
 *
 * The stylesheet is imported here and nowhere a frame loads, so none of it reaches the app.
 */
export function BenchShell() {
	const { pathname, search } = useAddress();
	const navigate = useNavigate();
	const [appearance, setAppearance] = useWorkbenchAppearance();
	const [folded, toggleFolded] = useFolded();
	const [drawer, setDrawer] = useState(false);
	const [palette, setPaletteOpen] = useState(false);
	const opener = useRef<HTMLElement | null>(null);

	/** The jump list gives the focus back to whatever opened it. */
	const setPalette = (open: boolean) => {
		if (open) {
			opener.current =
				document.activeElement instanceof HTMLElement ? document.activeElement : null;
		}
		setPaletteOpen(open);
	};

	/** After the list has gone: while it is open everything behind it is inert and takes no focus. */
	useEffect(() => {
		if (!palette) opener.current?.focus();
	}, [palette]);

	useEffect(() => {
		const onKey = (event: KeyboardEvent) => {
			if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
				event.preventDefault();
				setPalette(!palette);
				return;
			}

			if (event.key === 'Escape') {
				setPalette(false);
				setDrawer(false);
				return;
			}

			const target = event.target;
			const typing =
				target instanceof HTMLElement &&
				(target.isContentEditable || ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName));

			if (event.key === '\\' && !typing && !event.metaKey && !event.ctrlKey && !event.altKey) {
				event.preventDefault();
				toggleFolded();
			}
		};

		window.addEventListener('keydown', onKey);
		return () => window.removeEventListener('keydown', onKey);
	});

	/** A drawer left open is a phone's; it does not follow the window out to a width with a sidebar. */
	useEffect(() => {
		if (typeof window.matchMedia !== 'function') return;

		const wide = window.matchMedia('(min-width: 1024px)');
		const shut = () => {
			if (wide.matches) setDrawer(false);
		};

		wide.addEventListener('change', shut);
		return () => wide.removeEventListener('change', shut);
	}, []);

	const chrome: Chrome = {
		folded,
		toggleFolded,
		drawer,
		setDrawer,
		palette,
		setPalette,
		appearance,
		setAppearance,
		carried: carry(search),
		tried: parseTried(search['try']),
		setTried: (next: Tried) => {
			const rest = Object.fromEntries(Object.entries(search).filter(([key]) => key !== 'try'));
			const written = formatTried(next);

			void navigate({
				to: pathname,
				search: asSearch(written === undefined ? rest : { ...rest, try: written }),
				replace: true,
			});
		},
	};

	return (
		<ChromeContext value={chrome}>
			<div
				className="wb"
				data-wb-theme={appearance}
				data-wb-folded={folded ? '' : undefined}
				data-wb-drawer={drawer ? '' : undefined}
			>
				<div className="wb-side-hold" inert={palette}>
					<Sidebar />
				</div>
				{drawer && (
					<button
						type="button"
						className="wb-veil wb-veil-touch"
						aria-label="Close the places"
						onClick={() => setDrawer(false)}
					/>
				)}
				<main className="wb-main" inert={palette || drawer}>
					<Outlet />
				</main>
				{palette && <Palette />}
			</div>
		</ChromeContext>
	);
}
