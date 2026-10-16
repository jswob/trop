import { useNavigate } from '@tanstack/react-router';
import { useState } from 'react';

import { asSearch } from './address';
import { useChrome } from './chrome';
import { jumpEntries, matchJump, type JumpEntry } from './nav';
import { benchNav } from './nav-input';

const ENTRIES = jumpEntries(benchNav());

/**
 * The jump list: any place or kit component, by a part of its name.
 *
 * It sits beside the sidebar and never instead of it. Going somewhere from here takes the options
 * of the bar along, exactly as a link in the sidebar does.
 */
export function Palette() {
	const { carried, setPalette, setDrawer } = useChrome();
	const navigate = useNavigate();
	const [query, setQuery] = useState('');
	const [at, setAt] = useState(0);

	const found = matchJump(ENTRIES, query);
	const chosen = Math.min(at, Math.max(0, found.length - 1));

	const go = (entry: JumpEntry | undefined) => {
		if (entry === undefined) return;

		setPalette(false);
		setDrawer(false);
		void navigate({ to: entry.to, search: asSearch(carried) });
	};

	return (
		<>
			<button
				type="button"
				className="wb-veil"
				aria-label="Close the jump list"
				onClick={() => setPalette(false)}
			/>
			<div className="wb-pal" role="dialog" aria-label="Go to a place">
				<input
					id="wb-jump"
					role="combobox"
					aria-label="Go to a place"
					aria-expanded="true"
					aria-controls="wb-jump-list"
					aria-activedescendant={found[chosen] ? `wb-jump-${found[chosen].id}` : undefined}
					autoComplete="off"
					spellCheck={false}
					autoFocus
					value={query}
					onChange={event => {
						setQuery(event.target.value);
						setAt(0);
					}}
					onKeyDown={event => {
						if (event.key === 'ArrowDown') {
							event.preventDefault();
							setAt(Math.min(chosen + 1, found.length - 1));
						} else if (event.key === 'ArrowUp') {
							event.preventDefault();
							setAt(Math.max(chosen - 1, 0));
						} else if (event.key === 'Enter') {
							event.preventDefault();
							go(found[chosen]);
						} else if (event.key === 'Tab') {
							// The list is the one thing to type into while it is open; the focus stays in it.
							event.preventDefault();
						}
					}}
				/>
				<ul id="wb-jump-list" role="listbox" aria-label="Places">
					{found.map((entry, index) => (
						<li
							key={entry.to}
							id={`wb-jump-${entry.id}`}
							role="option"
							aria-selected={index === chosen}
							onClick={() => go(entry)}
							onMouseEnter={() => setAt(index)}
						>
							{entry.label}
							<span>{entry.hint}</span>
						</li>
					))}
				</ul>
				{found.length === 0 && <p className="wb-none">No place by that name.</p>}
			</div>
		</>
	);
}
