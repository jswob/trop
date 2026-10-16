import { Disclose } from '@app/ui/kit/Disclose';
import { Field, controlBox } from '@app/ui/kit/Field';
import { Placeholder, Waiting, WaitingNote } from '@app/ui/kit/Waiting';
import {
	autoUpdate,
	flip,
	FloatingPortal,
	offset,
	shift,
	size,
	useDismiss,
	useFloating,
	useInteractions,
	type OpenChangeReason,
} from '@floating-ui/react';
import { Check } from 'lucide-react';
import {
	useEffect,
	useId,
	useMemo,
	useRef,
	useState,
	type ChangeEvent,
	type KeyboardEvent as ReactKeyboardEvent,
	type ReactNode,
} from 'react';
import { CONTROL_H, CONTROL_TYPE } from '@app/ui/kit/control';
import { FOCUS, FOCUS_WITHIN } from '@app/ui/kit/focus';

/**
 * One thing that can be picked, and the reason it cannot.
 *
 * `blockedBy` is a sentence rather than a boolean because the list has to say **why**: a name that
 * is simply missing from a list is indistinguishable from a bug, and the fact that rules an option
 * out is exactly what the person at the screen is trying to find out.
 */
export type SelectOption = { id: string; label: string; blockedBy?: string | null };

/**
 * The list's viewport: six rows, always, whatever is in it.
 *
 * A panel sized to its contents changes shape between *loading* and *loaded*, and again on every
 * keystroke in the filter — the same rejection as a trigger that grows, one row further down and
 * directly under the pointer. Past six rows it scrolls, with a thumb in control ink on the ground
 * rather than the browser's own plate.
 */
const LIST_VIEWPORT =
	'h-[calc(6*var(--spacing-ui-touch))] overflow-y-auto [scrollbar-color:var(--color-ui-edge)_transparent] [scrollbar-width:thin] lg:h-[calc(6*var(--spacing-ui-control))]';

/** The panel's measure: a number, and the trigger's width — never the longest label in the list. */
const PANEL_W = 'w-72';

/** The page's own margin, which the list may not cross — on every edge it comes near. */
const GUTTER = 20;

/**
 * The seam. The list is pulled one border width onto the trigger, so the two edges between them
 * resolve to a single hairline rather than a 2px bar — and it works at both ends, because a
 * negative offset is *toward* the trigger whichever side the list ends up on.
 */
const SEAM = -1;

/**
 * Where the list goes: under the trigger, unless there is no room under the trigger.
 *
 * Not `absolute top-full` with a width clamped by hand, which is wrong twice. A list opened near
 * the foot of a column runs past the glass and is **clipped** by the column's own scroller rather
 * than scrolled to — measured at 1440x900, a select at y=703 opened a 226px list that ended 60px
 * below the fold and was cut there. And a hand-written clamp is one viewport reading;
 * `@floating-ui/react` does all of them, and keeps doing them while the column scrolls.
 *
 * What the middleware is asked for, and nothing else:
 *
 * - `offset(SEAM)` — the hairline the design settled on, at whichever end the list lands.
 * - `flip` on the main axis only, so the list goes above the trigger when below will not fit. The
 *   cross axis is `shift`'s job; letting both move sideways makes the list hunt.
 * - `shift` — the horizontal clamp, inset by the page's own gutter.
 * - `size` — the width floor is the trigger, the ceiling is the room left beside it. **Never the
 *   height**: the list is six rows whatever the viewport does, the same way it is six rows whatever
 *   the data does.
 *
 * And a portal, because the fix for being clipped by an ancestor is to stop being inside one.
 */
function usePlacement(open: boolean, onClose: (reason?: OpenChangeReason) => void) {
	const floating = useFloating({
		open,
		onOpenChange: (next, _event, reason) => {
			if (!next) onClose(reason);
		},
		placement: 'bottom-start',
		strategy: 'fixed',
		middleware: [
			offset(SEAM),
			flip({ padding: GUTTER, crossAxis: false }),
			shift({ padding: GUTTER }),
			size({
				padding: GUTTER,
				apply({ rects, availableWidth, elements }) {
					/*
					 * A portal escapes `display: none` as happily as it escapes `overflow: hidden`.
					 * `kit/SplitShell` draws the wide chrome and the narrow chrome at once and lets a
					 * media query pick between them, so a select seeded open is open in *both* — and the
					 * copy nobody can see posted its list to the top corner of the page, measured at
					 * (20, −1). A reference with no box gets no list.
					 */
					elements.floating.hidden = rects.reference.width === 0;
					elements.floating.style.minWidth = `${String(Math.round(rects.reference.width))}px`;
					elements.floating.style.maxWidth = `${String(
						Math.round(Math.max(rects.reference.width, availableWidth))
					)}px`;
				},
			}),
		],
		whileElementsMounted: autoUpdate,
	});

	const { getReferenceProps, getFloatingProps } = useInteractions([useDismiss(floating.context)]);

	return { ...floating, getReferenceProps, getFloatingProps };
}

/**
 * Everything a `Select` does, kept out of it.
 *
 * The shell is controlled, because a screen that cannot say *this one is open* cannot draw the state
 * it exists to draw. This is the state that goes with it, so no caller re-derives what *picking*
 * means: filtering is case-insensitive over the label, picking closes the list and clears the
 * filter, and a blocked option cannot become the value.
 */
export function useSelect({
	options,
	value: initial = null,
	open: startOpen = false,
	onPick: picked,
}: {
	options: SelectOption[];
	value?: SelectOption | null;
	/** Seeded open, so a workbench state can draw the list without anybody clicking. */
	open?: boolean;
	/**
	 * Told after a pickable option has been taken and the list has shut.
	 *
	 * Opt-in. It exists because the one thing a caller cannot do from `props.onPick` is learn that
	 * a pick happened **without re-deciding what pickable means** — the `blockedBy` guard lives here
	 * and is not allowed a second copy. A caller whose select is a way of starting something
	 * (`+ Add a project`, say) needs exactly that and nothing else.
	 */
	onPick?: (option: SelectOption) => void;
}) {
	const [value, setValue] = useState<SelectOption | null>(initial);
	const [open, setOpen] = useState(startOpen);
	const [filter, setFilter] = useState('');

	/** The value's own entry in the list, if the list has it and it can be taken. */
	function at(chosen: SelectOption | null) {
		if (chosen === null) return null;
		const entry = options.find(option => option.id === chosen.id);
		return entry && !entry.blockedBy ? entry.id : null;
	}

	/**
	 * Where the keyboard stands in the list — and it starts **on the value**.
	 *
	 * A list that opened with nothing active would send the first `ArrowDown` to the top of the list
	 * whatever was chosen, so moving one step from *3 min* to *5 min* would mean walking the whole
	 * list to get back to where the box already said it was. Only a value that can still be picked
	 * is stood on; with none, nothing is active and the first arrow finds the first pickable option.
	 */
	const [activeId, setActiveId] = useState<string | null>(() => (startOpen ? at(initial) : null));

	const shown = useMemo(() => {
		const needle = filter.trim().toLowerCase();
		return needle ? options.filter(option => option.label.toLowerCase().includes(needle)) : options;
	}, [options, filter]);

	const pickable = shown.filter(option => !option.blockedBy);

	const openList = () => {
		setActiveId(at(value));
		setOpen(true);
	};

	const close = () => {
		setOpen(false);
		setFilter('');
		setActiveId(null);
	};

	const pick = (option: SelectOption) => {
		if (option.blockedBy) return;
		setValue(option);
		close();
		picked?.(option);
	};

	/**
	 * Emptying the value, as an errand rather than a state change.
	 *
	 * It closes the list, because **an action taken from the list ends the list** — the same rule
	 * picking a name follows. A footer that only changed the value would leave the panel standing
	 * over the page with nothing left in it to do.
	 */
	const clear = () => {
		setValue(null);
		close();
	};

	const move = (step: 1 | -1) => {
		if (pickable.length === 0) return;
		const at = pickable.findIndex(option => option.id === activeId);
		const next = at === -1 ? (step === 1 ? 0 : pickable.length - 1) : at + step;
		setActiveId(pickable[(next + pickable.length) % pickable.length]?.id ?? null);
	};

	return {
		value,
		open,
		filter,
		shown,
		setValue,
		clear,
		/** Spread onto `Select`: everything it is controlled by, in one object. */
		props: {
			value,
			open,
			filter,
			options: shown,
			activeId,
			onOpen: openList,
			onClose: close,
			onFilter: setFilter,
			onPick: pick,
			onMove: move,
		},
	};
}

/** One row of the list. The tick marks the value; the reason sits where the tick would be. */
function OptionRow({
	option,
	selected,
	active,
	onPick,
}: {
	option: SelectOption;
	selected: boolean;
	active: boolean;
	onPick: (option: SelectOption) => void;
}) {
	const blocked = Boolean(option.blockedBy);

	return (
		<li
			id={`option-${option.id}`}
			role="option"
			aria-selected={selected}
			aria-disabled={blocked || undefined}
			onMouseDown={blocked ? undefined : event => event.preventDefault()}
			onClick={blocked ? undefined : () => onPick(option)}
			className={`flex ${CONTROL_H} items-center gap-3 px-2 text-ui-body ${
				blocked
					? 'text-ui-ink-3'
					: `cursor-pointer text-ui-ink ${active || selected ? 'bg-ui-row' : 'hover:bg-ui-row'}`
			}`}
		>
			<span className="min-w-0 truncate">{option.label}</span>
			{option.blockedBy ? (
				<span className="ml-auto shrink-0 text-ui-ink-3">{option.blockedBy}</span>
			) : null}
			{selected ? <Check aria-hidden className="ml-auto h-3.5 w-3.5 shrink-0 text-ui-ink" /> : null}
		</li>
	);
}

/** The kit's own loader, at the size the list already is. */
function LoadingRows() {
	return (
		<div>
			<Waiting at={0.35}>
				<div aria-hidden>
					{[132, 98, 164].map(width => (
						<div key={width} className={`flex ${CONTROL_H} items-center px-2`}>
							<Placeholder width={width} />
						</div>
					))}
				</div>
			</Waiting>
			<p className={`flex ${CONTROL_H} items-center border-t border-ui-rule px-2`}>
				<WaitingNote />
			</p>
		</div>
	);
}

/**
 * A select: one box that does not change size when it opens.
 *
 * Closed, it is the box with the value in it and a chevron at the inset. Open, **the same box**
 * carries the filter on its own value line — which is what buys a search field without buying a row
 * of height — and the list hangs under it as a separate object, pulled up by its own border width so
 * the seam between the two resolves to one hairline. Nothing on the page moves, which is the whole
 * point on a dense page: the rows under the box stay where they were while somebody is reading
 * them.
 *
 * What the caller must not do: hand it a `footer` that comes and goes with the data. A row that
 * appears only when a value is chosen changes the panel's height as the value changes, which is the
 * same ban one level down. Draw it always, disabled when there is nothing to do, or never.
 *
 * Keyboard: the open box is a `combobox` — up and down move the active option, `Enter` takes it,
 * `Escape` closes, and a click anywhere else closes it too.
 *
 * Focus comes home. The closed box and the open one are two elements, so shutting the list
 * unmounts the one that had focus and the browser drops it on the body — measured, a pick left the
 * next `Tab` starting from the top of the page. After a pick, `Escape` or
 * any other close the select made itself, the closed box takes focus back. **Not** after a click
 * somewhere else: that click was aimed at something, and focus belongs to it. And never over a
 * control that has already claimed focus — a pick that opens something (`onPick` starting an
 * errand) keeps the focus the errand gave out.
 */
export function Select({
	label,
	labelHidden,
	width,
	value,
	placeholder = 'Choose',
	options,
	blockedHeading,
	emptyNote = 'Nothing to choose',
	filterLabel = 'Filter',
	loading = false,
	disabled = false,
	error = null,
	hint = null,
	open,
	filter,
	activeId = null,
	onOpen,
	onClose,
	onFilter,
	onPick,
	onMove,
	footer = null,
	describedBy,
}: {
	/**
	 * The closed box's accessible name, and the list's. The box is announced as the label and then
	 * its value — *Snooze for, 3 min* — because a name that stopped at the label would leave a
	 * screen reader to open the list to learn what was chosen. With no value it is the label.
	 */
	label: string;
	labelHidden?: boolean;
	width?: string;
	value: SelectOption | null;
	placeholder?: string;
	options: SelectOption[];
	/** The heading over the options these facts rule out — it names the fact. */
	blockedHeading?: string;
	/** What the list says when there is nothing in it. Words, never footprints. */
	emptyNote?: string;
	filterLabel?: string;
	loading?: boolean;
	disabled?: boolean;
	error?: string | null;
	hint?: string | null;
	open: boolean;
	filter: string;
	activeId?: string | null;
	onOpen: () => void;
	onClose: () => void;
	onFilter: (next: string) => void;
	onPick: (option: SelectOption) => void;
	onMove: (step: 1 | -1) => void;
	/** One row under the list, inside the panel. Drawn always or never — never with the data. */
	footer?: ReactNode;
	/**
	 * The id of a sentence somewhere else that the closed box should be announced with.
	 *
	 * `kit/Button`'s prop, for the same reason: the caller has a better place to print the words,
	 * and the box owes only the wire. It rides alongside the error rather than instead of it — both
	 * ids, space-separated, when both exist. A caller that patched the attribute onto the box from an
	 * effect instead would have to re-run it on every close, because the box is re-mounted each time.
	 */
	describedBy?: string;
}) {
	const listId = useId();
	const errorId = error === null ? undefined : `${listId}-error`;
	const described = [errorId, describedBy].filter(Boolean).join(' ') || undefined;

	const trigger = useRef<HTMLButtonElement>(null);
	/** Whether the next close hands focus back to the box. Only an outside press says no. */
	const homing = useRef(false);

	const {
		// eslint-disable-next-line @typescript-eslint/unbound-method -- Reason: floating-ui's ref setters are stable callbacks that never read `this`
		refs: { setReference, setFloating },
		floatingStyles,
		getReferenceProps,
		getFloatingProps,
	} = usePlacement(open, reason => {
		if (reason === 'outside-press') homing.current = false;
		onClose();
	});

	useEffect(() => {
		if (open) {
			homing.current = true;
			return;
		}
		if (!homing.current) return;
		homing.current = false;
		const held = document.activeElement;
		if (held === null || held === document.body) trigger.current?.focus();
	}, [open]);

	const eligible = options.filter(option => !option.blockedBy);
	const blocked = options.filter(option => option.blockedBy);
	const empty = !loading && options.length === 0;

	const onKeyDown = (event: ReactKeyboardEvent) => {
		if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
			event.preventDefault();
			onMove(event.key === 'ArrowDown' ? 1 : -1);
			return;
		}
		if (event.key === 'Enter') {
			const active = options.find(option => option.id === activeId);
			if (active) {
				event.preventDefault();
				onPick(active);
			}
		}
	};

	const boxClass = controlBox({ invalid: error !== null, disabled });

	return (
		<Field
			label={label}
			labelHidden={labelHidden}
			hint={hint}
			error={error}
			errorId={errorId}
			width={width}
		>
			<div ref={setReference} {...getReferenceProps()}>
				{open ? (
					/* A label, not a bare box: the target is the box, so the input inside it is never
					   the 30px it measures between the borders. */
					<label className={`${boxClass} ${FOCUS_WITHIN}`}>
						<input
							autoFocus
							role="combobox"
							aria-label={filterLabel}
							aria-expanded
							aria-controls={listId}
							aria-activedescendant={activeId ? `option-${activeId}` : undefined}
							value={filter}
							onChange={(event: ChangeEvent<HTMLInputElement>) => onFilter(event.target.value)}
							onKeyDown={onKeyDown}
							placeholder={value?.label ?? placeholder}
							className={`h-full min-w-0 flex-1 bg-transparent ${CONTROL_TYPE} text-ui-ink outline-none placeholder:text-ui-ink-3`}
						/>
						<Disclose open />
					</label>
				) : (
					<button
						ref={trigger}
						type="button"
						aria-haspopup="listbox"
						aria-expanded={false}
						aria-label={value ? `${label}, ${value.label}` : label}
						aria-invalid={error === null ? undefined : true}
						aria-describedby={described}
						disabled={disabled}
						onClick={onOpen}
						className={`${boxClass} cursor-pointer text-left ${FOCUS}`}
					>
						<Value value={value} placeholder={placeholder} />
					</button>
				)}

				{open ? (
					<FloatingPortal>
						<div
							ref={setFloating}
							style={floatingStyles}
							{...getFloatingProps()}
							className={`z-20 border border-ui-edge bg-ui-bg ${PANEL_W}`}
						>
							<div className={LIST_VIEWPORT}>
								{loading ? <LoadingRows /> : null}
								{empty ? (
									<p className={`flex ${CONTROL_H} items-center px-2 text-ui-body text-ui-ink-3`}>
										{emptyNote}
									</p>
								) : null}
								{loading || empty ? null : (
									<ul id={listId} role="listbox" aria-label={label}>
										{eligible.map(option => (
											<OptionRow
												key={option.id}
												option={option}
												selected={option.id === value?.id}
												active={option.id === activeId}
												onPick={onPick}
											/>
										))}
										{blocked.length === 0 ? null : (
											<li
												role="presentation"
												className="flex h-ui-row items-center border-t border-ui-rule px-2 text-ui-body text-ui-ink-3"
											>
												{blockedHeading ?? 'Unavailable'}
											</li>
										)}
										{blocked.map(option => (
											<OptionRow
												key={option.id}
												option={option}
												selected={false}
												active={false}
												onPick={onPick}
											/>
										))}
									</ul>
								)}
							</div>
							{footer}
						</div>
					</FloatingPortal>
				) : null}
			</div>
		</Field>
	);
}

/**
 * The value, or the word standing in for one. Placeholder ink says *nothing picked*.
 *
 * The chevron is unconditional, and the inset is the chevron's alone — three glyphs in a 176px box
 * would cost the name four characters. A disclosure glyph appears on what opens, and on nothing
 * else. It is `kit/Disclose`, so the mapping from open to glyph is drawn in one place.
 */
function Value({ value, placeholder }: { value: SelectOption | null; placeholder: string }) {
	return (
		<>
			<span className={`min-w-0 truncate ${value ? '' : 'text-ui-ink-3'}`}>
				{value?.label ?? placeholder}
			</span>
			<Disclose open={false} className="ml-auto" />
		</>
	);
}
