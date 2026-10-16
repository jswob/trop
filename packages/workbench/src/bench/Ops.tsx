import type { ReactNode } from 'react';

import { VIEWPORT_NAMES, type ViewportName } from '../lib/viewports';

import { all, cursorOf, shownOf, step, withOption, type BenchSearch } from './search';

export const WIDTH_LABEL: Record<ViewportName, string> = {
	mobile: 'Phone',
	tablet: 'Tablet',
	desktop: 'Desktop',
};

/** How many frames a row holds, as the address's `scale` spells it. */
export const ACROSS = [
	{ label: 'Fit', scale: undefined, across: 'fit' },
	{ label: 'Two', scale: 0.5, across: 'two' },
	{ label: 'One', scale: 1, across: 'one' },
] as const;

/** The row size an address asks for. Any scale it never wrote reads as the nearer of the two. */
export function acrossOf(scale: number | undefined): (typeof ACROSS)[number]['across'] {
	if (scale === undefined) return 'fit';
	return scale <= 0.5 ? 'two' : 'one';
}

/** The frame clock, in words. `0` is off: every animation at its end. */
export const MOTION = [
	{ label: 'Real time', rate: 1 },
	{ label: 'Half', rate: 0.5 },
	{ label: 'A fifth', rate: 0.2 },
	{ label: 'Off', rate: 0 },
] as const;

function Key({ children }: { children: ReactNode }) {
	return (
		<kbd className="wb-key" aria-hidden="true">
			{children}
		</kbd>
	);
}

function Option({
	on,
	onPress,
	label,
	keyHint,
	children,
}: {
	on: boolean;
	onPress: () => void;
	/** Its name, when what it prints is not one. */
	label?: string;
	keyHint?: string;
	children: ReactNode;
}) {
	return (
		<button type="button" aria-pressed={on} aria-label={label} onClick={onPress}>
			{children}
			{keyHint !== undefined && <Key>{keyHint}</Key>}
		</button>
	);
}

function Group({
	name,
	quiet = false,
	keyHint,
	status,
	children,
}: {
	name: string;
	/** Its options name it well enough that the bar drops the label; the sheet still prints it. */
	quiet?: boolean;
	keyHint?: string;
	status?: string;
	children: ReactNode;
}) {
	return (
		<div className="wb-op" role="group" aria-label={name}>
			<span className={quiet ? 'wb-lb wb-lb-quiet' : 'wb-lb'} aria-hidden="true">
				{name}
				{keyHint !== undefined && !quiet && <Key>{keyHint}</Key>}
			</span>
			<div className="wb-sw">{children}</div>
			{keyHint !== undefined && quiet && <Key>{keyHint}</Key>}
			{status !== undefined && <span className="wb-status">{status}</span>}
		</div>
	);
}

/**
 * The options of the stage: six switches, each in words, each with its key.
 *
 * Every one writes to the address and nothing else, so an arrangement is a link and a link to
 * another place can take it along. The states themselves are not listed here: hiding one or
 * showing one alone is on its own caption.
 */
export function Ops({
	ids,
	search,
	onChange,
	modes = true,
	parts = false,
	notes = false,
}: {
	ids: readonly string[];
	search: BenchSearch;
	onChange: (next: BenchSearch) => void;
	/** The two modes and the row size. A component drawn at its own size has neither. */
	modes?: boolean;
	/** The parts switch, where the page has named parts to outline. */
	parts?: boolean;
	/** The notes switch, where the page has notes to open: a candidate's, in Proto. */
	notes?: boolean;
}) {
	const widths = search.show === 'widths';
	const shown = shownOf(ids, search);
	const cursor = cursorOf(ids, search);
	const total = ids.length;
	const alone = shown.length === 1 && total > 1;

	const status =
		total > 1 && (widths || alone)
			? `${String(ids.indexOf(cursor ?? '') + 1)} of ${String(total)}`
			: shown.length < total
				? `${String(shown.length)} of ${String(total)}`
				: undefined;

	const set = (patch: BenchSearch) => onChange(withOption(search, patch));

	return (
		<div className="wb-ops">
			{modes && (
				<Group name="Show" quiet keyHint="W">
					<Option on={!widths} onPress={() => set({ show: undefined })}>
						States
					</Option>
					<Option on={widths} onPress={() => set({ show: 'widths' })}>
						Widths
					</Option>
				</Group>
			)}

			<Group name="States" quiet status={status}>
				<Option
					on={!widths && shown.length === total}
					onPress={() => onChange(all(search))}
					keyHint="A"
				>
					All {total}
				</Option>
				<Option
					on={false}
					onPress={() => onChange(step(ids, search, -1))}
					label="Previous state"
					keyHint="["
				>
					‹
				</Option>
				<Option
					on={false}
					onPress={() => onChange(step(ids, search, 1))}
					label="Next state"
					keyHint="]"
				>
					›
				</Option>
			</Group>

			{!widths && (
				<Group name="Width" quiet>
					{VIEWPORT_NAMES.map((name, index) => (
						<Option
							key={name}
							on={(search.width ?? 'desktop') === name}
							onPress={() => set({ width: name })}
							keyHint={String(index + 1)}
						>
							{WIDTH_LABEL[name]}
						</Option>
					))}
				</Group>
			)}

			<Group name="Frames" keyHint="T">
				<Option on={search.theme !== 'light'} onPress={() => set({ theme: 'dark' })}>
					Dark
				</Option>
				<Option on={search.theme === 'light'} onPress={() => set({ theme: 'light' })}>
					Light
				</Option>
			</Group>

			{!widths && modes && (
				<Group name="Across" keyHint="F">
					{ACROSS.map(option => (
						<Option
							key={option.across}
							on={acrossOf(search.scale) === option.across}
							onPress={() => set({ scale: option.scale })}
						>
							{option.label}
						</Option>
					))}
				</Group>
			)}

			{parts && (
				<Group name="Parts" keyHint="P">
					<Option on={search.parts !== 'on'} onPress={() => set({ parts: undefined })}>
						Off
					</Option>
					<Option on={search.parts === 'on'} onPress={() => set({ parts: 'on' })}>
						On
					</Option>
				</Group>
			)}

			{notes && (
				<Group name="Notes" keyHint="N">
					<Option on={search.notes !== 'on'} onPress={() => set({ notes: undefined })}>
						Off
					</Option>
					<Option on={search.notes === 'on'} onPress={() => set({ notes: 'on' })}>
						On
					</Option>
				</Group>
			)}

			<Group name="Motion" keyHint="M">
				{MOTION.map(option => (
					<Option
						key={option.label}
						on={search.t === undefined && (search.rate ?? 1) === option.rate}
						onPress={() => set({ t: undefined, rate: option.rate })}
					>
						{option.label}
					</Option>
				))}
			</Group>
		</div>
	);
}
