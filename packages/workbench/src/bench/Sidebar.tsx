import { Link } from '@tanstack/react-router';
import { useEffect, useRef, useState } from 'react';

import { asSearch, useAddress } from './address';
import { useChrome } from './chrome';
import { HOST } from './host';
import { navGroups, type NavRow } from './nav';
import { benchNav } from './nav-input';

const GROUPS = navGroups(benchNav());

/**
 * Where you are: the four spaces, and under Views the app's own destinations.
 *
 * One line a row. A destination with several places folds, and the one you are in is open. Every
 * link takes the options of the bar with it, so a width or an appearance chosen on one place is
 * still chosen on the next.
 */
export function Sidebar() {
	const { carried, appearance, setAppearance, setPalette, drawer, setDrawer } = useChrome();
	const { pathname } = useAddress();
	/** A destination a hand opened or shut. Until then it is open while you are in it. */
	const [turned, setTurned] = useState<Readonly<Record<string, boolean>>>({});
	const jump = useRef<HTMLButtonElement>(null);

	/** The drawer opens over the page, so the focus goes into it rather than staying behind it. */
	useEffect(() => {
		if (drawer) jump.current?.focus();
	}, [drawer]);

	const holds = (row: NavRow): boolean =>
		row.children?.some(child => child.to === pathname) ?? false;

	/** A row that is a page and has pages under it shows them while you are in any of them. */
	const within = (row: NavRow): boolean =>
		row.to === pathname || (row.children?.some(child => child.to === pathname) ?? false);

	const link = (row: NavRow, child = false) => (
		<Link
			key={row.label}
			to={row.to}
			search={asSearch(carried)}
			activeOptions={{ exact: true }}
			className={child ? 'wb-row wb-row-child' : 'wb-row'}
			aria-current={row.to === pathname ? 'page' : undefined}
			onClick={() => setDrawer(false)}
		>
			<span>{row.label}</span>
			{row.count !== undefined && <span className="wb-count">{row.count}</span>}
		</Link>
	);

	return (
		<aside className="wb-side" aria-label="Places">
			<div className="wb-brand">
				Workbench<span>{HOST.name}</span>
			</div>

			<button ref={jump} type="button" className="wb-jump" onClick={() => setPalette(true)}>
				Go to a place
				<kbd className="wb-key" aria-hidden="true">
					⌘K
				</kbd>
			</button>

			<nav className="wb-nav">
				{GROUPS.map(group => (
					<div key={group.title}>
						<h2 className="wb-group">{group.title}</h2>
						{group.rows.map(row => {
							if (row.children === undefined) return link(row);
							if (row.to !== undefined) {
								return (
									<div key={row.label}>
										{link(row)}
										{within(row) && row.children.map(sub => link(sub, true))}
									</div>
								);
							}

							const open = turned[row.label] ?? holds(row);

							return (
								<div key={row.label}>
									<button
										type="button"
										className="wb-row"
										aria-expanded={open}
										onClick={() => setTurned({ ...turned, [row.label]: !open })}
									>
										<i className="wb-chevron" aria-hidden="true" />
										<span>{row.label}</span>
									</button>
									{open && row.children.map(child => link(child, true))}
								</div>
							);
						})}
						{group.empty !== undefined && <p className="wb-none">{group.empty}</p>}
					</div>
				))}
			</nav>

			<div className="wb-foot" role="group" aria-label="Workbench appearance">
				<span aria-hidden="true">Workbench</span>
				<div className="wb-sw">
					<button
						type="button"
						aria-pressed={appearance === 'light'}
						onClick={() => setAppearance('light')}
					>
						Light
					</button>
					<button
						type="button"
						aria-pressed={appearance === 'dark'}
						onClick={() => setAppearance('dark')}
					>
						Dark
					</button>
				</div>
			</div>
		</aside>
	);
}
