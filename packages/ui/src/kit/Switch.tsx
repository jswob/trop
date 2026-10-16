import { MOTION } from './Button';
import { CONTROL_H } from './control';
import { FOCUS } from './focus';

const CHOSEN = `bg-ui-ink text-ui-bg hover:bg-ui-ink-2 active:bg-ui-ink-3`;
const UNCHOSEN = `text-ui-ink-2 hover:bg-ui-row hover:text-ui-ink active:bg-ui-beat`;

/** One setting a switch offers: the value it stands for, and the word it is drawn as. */
export type SwitchOption<T> = { value: T; word: string };

/**
 * **One of a few words, chosen.** The chosen word stands on the `primary` rung's ink plate and
 * every other is a bare `ui-ink-2` word, each at the control's height with 12px either side. A
 * press chooses at once: there is nothing to confirm or save, so there is no second control.
 *
 * Controlled. The caller holds the value — Account's appearance, say — and the words are the
 * wire: each is a button that says whether it is pressed.
 */
export function Switch<T>({
	label,
	options,
	value,
	choose,
}: {
	/** The group's accessible name — what is being chosen. */
	label: string;
	options: readonly SwitchOption<T>[];
	value: T;
	choose: (value: T) => void;
}) {
	return (
		<div role="group" aria-label={label} className="flex flex-wrap gap-1">
			{options.map(option => {
				const chosen = option.value === value;
				return (
					<button
						key={option.word}
						type="button"
						aria-pressed={chosen}
						onClick={() => {
							choose(option.value);
						}}
						className={`${CONTROL_H} cursor-pointer px-3 text-ui-body whitespace-nowrap ${chosen ? CHOSEN : UNCHOSEN} ${MOTION} ${FOCUS}`}
					>
						{option.word}
					</button>
				);
			})}
		</div>
	);
}
