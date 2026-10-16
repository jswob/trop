import type { ChangeEvent, ReactNode } from 'react';

import { CONTROL_H, CONTROL_TYPE } from '@app/ui/kit/control';
import { FOCUS } from '@app/ui/kit/focus';

/**
 * The box every control in the system shares: an edge on all four sides, at the standing control
 * height, with the ground showing through it.
 *
 * Three things it is not:
 *
 * - **Not a rule under the value.** An edge under the value alone reads as a generic form and gives
 *   the box no boundary — a control is a box.
 * - **Not a filled interior.** A 4% recess to say *type here* costs contrast: `ui-ink-3` over that
 *   ground is 4.59:1 against the system's 5.0 floor, and 4.10:1 over the hover ground, so a
 *   placeholder in a well fails AA the moment a pointer is over it. The interior is the ground.
 * - **Not outlined in `ui-rule`.** 8% white is the rows' own hairline, 1.23:1. The edge is
 *   `ui-edge` at 35%, which is 3.22:1 and over the non-text floor.
 *
 * `disabled` drops the edge to `ui-rule`: a control nobody can operate is not a control, and taking
 * its boundary away says that without taking the value away.
 *
 * `typing` is for the boxes a hand puts characters into, and swaps the body size for
 * {@link CONTROL_TYPE}. It is a flag rather than the default because the two are the same
 * property: written side by side on one element, which size wins is decided by where Tailwind
 * happens to order the two font-size utilities in its stylesheet, and measured here the 13px did.
 * It is also deliberately **not** on by default — a closed `Select` prints a real name in its box,
 * and a name's length cap is measured at the 13px body size.
 */
export function controlBox({
	invalid = false,
	disabled = false,
	typing = false,
}: { invalid?: boolean; disabled?: boolean; typing?: boolean } = {}): string {
	const edge = invalid ? 'border-ui-red' : disabled ? 'border-ui-rule' : 'border-ui-edge';
	const ink = disabled ? 'text-ui-ink-3' : 'text-ui-ink';
	const ground = disabled ? '' : 'hover:bg-ui-row';
	const size = typing ? CONTROL_TYPE : 'text-ui-body';

	return `flex ${CONTROL_H} w-full items-center gap-2 border px-2 ${size} ${edge} ${ink} ${ground}`;
}

/**
 * A control's label, its hint and its validation message — everything around the box.
 *
 * The label sits **above** the box, never inside it as a cell of its own: a label cell is a fixed
 * column inside a control, and at 390px it eats the value's whole measure.
 *
 * Invalid says so three ways — the edge, the `!` and the sentence. Red means *the form will not take
 * this*; it is never a status and never appears in a row beside a status glyph.
 */
export function Field({
	label,
	labelHidden = false,
	hint = null,
	error = null,
	errorId,
	width = 'w-full',
	children,
}: {
	label: string;
	/** For a field whose column heading already says what it is. The name survives for a reader. */
	labelHidden?: boolean;
	hint?: string | null;
	error?: string | null;
	/** Wire it to the control's own `aria-describedby`. */
	errorId?: string;
	width?: string;
	children: ReactNode;
}) {
	return (
		<div className={width}>
			<span className={labelHidden ? 'sr-only' : 'mb-1 block text-ui-body text-ui-ink-3'}>
				{label}
			</span>
			{children}
			{error === null ? null : (
				<span id={errorId} className="mt-1 flex items-start gap-1.5 text-ui-body text-ui-red">
					<span aria-hidden>!</span>
					{error}
				</span>
			)}
			{hint === null || error !== null ? null : (
				<span className="mt-1 block text-ui-body text-ui-ink-3">{hint}</span>
			)}
		</div>
	);
}

/**
 * Free text: the one thing in this system that is a box because a hand fills it.
 *
 * The box is the input itself, so the ring lands around the edge and the whole 32/40px box is the
 * target. Anything that needs a chevron is a `Select`, not this — a disclosure glyph on something
 * that does not open promises a press that does nothing.
 *
 * Uncontrolled unless `onChange` is given, because most of what this kit draws is a still.
 *
 * `secret` makes it a password box, as sign-in's is: the characters drawn as dots and the browser's
 * saved password offered. Same box, same edge, same error — a secret is still free text.
 *
 * `maxLength` refuses a keystroke past it, at the length the API would refuse: nothing is drawn
 * for it, the character simply does not land.
 */
export function TextField({
	label,
	labelHidden,
	value = '',
	placeholder,
	hint,
	error = null,
	disabled = false,
	secret = false,
	maxLength,
	width,
	onChange,
}: {
	label: string;
	labelHidden?: boolean;
	value?: string;
	placeholder?: string;
	hint?: string | null;
	error?: string | null;
	disabled?: boolean;
	secret?: boolean;
	maxLength?: number;
	width?: string;
	onChange?: (next: string) => void;
}) {
	const errorId = error === null ? undefined : `${label}-error`;
	const bound = onChange
		? { value, onChange: (event: ChangeEvent<HTMLInputElement>) => onChange(event.target.value) }
		: { defaultValue: value };

	return (
		<Field
			label={label}
			labelHidden={labelHidden}
			hint={hint}
			error={error}
			errorId={errorId}
			width={width}
		>
			<input
				type={secret ? 'password' : 'text'}
				autoComplete={secret ? 'current-password' : undefined}
				{...bound}
				placeholder={placeholder}
				maxLength={maxLength}
				disabled={disabled}
				aria-label={label}
				aria-invalid={error === null ? undefined : true}
				aria-describedby={errorId}
				className={`${controlBox({ invalid: error !== null, disabled, typing: true })} bg-transparent placeholder:text-ui-ink-3 ${FOCUS}`}
			/>
		</Field>
	);
}
