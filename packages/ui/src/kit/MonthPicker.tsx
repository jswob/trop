import { IconButton } from '@app/ui/kit/Button';
import { Figure } from '@app/ui/kit/Figure';
import { CONTROL_H } from '@app/ui/kit/control';
import { FOCUS, FORCED_GROUND, FORCED_PRESS, FORCED_RING } from '@app/ui/kit/focus';
import { WEEKDAYS, dayName, monthStep, type MonthCell, type MonthPage } from '@app/ui/kit/month';

/**
 * The month picker: a range of days chosen by eye, against what is already taken.
 *
 * A day range is not two text boxes — picking a day is done by looking — so the calendar is
 * the control and it is always open wherever a range is supplied. It is drawn as a plain
 * seven-column grid with **no edge anywhere on it**: no box around the control, no rail beside
 * it, no rule between the rows, nothing printed between the weeks. Every figure stays full ink
 * so the month can be scanned; only spill days and past days step down the ramp.
 *
 * **A past day is fainter, never refused**: a run can be recorded after the fact and a past run's
 * days moved, so a day behind today takes the same ground, pointer and press as any other. Its
 * `ui-ink-3` figure is what says it is behind you.
 *
 * Three marks, each a different kind of thing, because a day can be all three at once:
 *
 * | state  | mark                                                    |
 * | ------ | ------------------------------------------------------- |
 * | picked | a full-bleed 2px `ui-ink` rule at the foot of the cell    |
 * | booked | a short 2px `ui-ink-3` stub under the figure              |
 * | today  | a `ui-ink` circle around the figure                       |
 *
 * Adjacent picked days make **one continuous length** that breaks where the week does, so a
 * run reads as a measured interval rather than a highlight, and no word is printed under its
 * ends. Bookings never join, so three booked days stay three marks. Hue is never used: a
 * picked day is not the one measure a screen moves and it is not validation, which are the
 * only two jobs colour has in this system.
 *
 * **The circle is the system's one exception to radius 0.** It is deliberate: today has to be
 * findable in one glance across 35 cells, it cannot be a rectangle because focus is a
 * rectangle, and it cannot be a line because both foot marks are lines. Curve is the only channel left, and one round thing in a
 * square system is unmistakable for exactly that reason. It does not license a second.
 *
 * A day that is **both** picked and booked draws **only the run**, identical to any other
 * picked day. The stub and the rule want the same pixels, and every way of showing both at
 * once — the stub over the rule, the rule knocked out under the figure — puts a gap or a
 * weight change in the middle of a length whose whole job is to be continuous. The run wins
 * because it is the thing being decided; what the day already holds is named in words under
 * the picker, where there is room to say it.
 *
 * What a booked day actually holds is named by the caller, under the picker, where there is
 * room for `Errands` and a 50px cell has none.
 */

/** A state a pointer would normally have to produce. Drawn on purpose in a variant grid. */
export type DayForce = 'hover' | 'focus' | 'active' | null;

export function MonthDay({
	cell,
	onPick,
	force = null,
}: {
	cell: MonthCell;
	onPick?: (iso: string) => void;
	force?: DayForce;
}) {
	const picked = cell.picked !== null;

	const ink = cell.past ? 'text-ui-ink-3' : cell.outside ? 'text-ui-ink-2' : 'text-ui-ink';

	const ground =
		force === 'active' ? FORCED_PRESS : force === 'hover' ? FORCED_GROUND : 'hover:bg-ui-row';

	const ring = force === 'focus' ? FORCED_RING : FOCUS;

	return (
		<button
			type="button"
			aria-pressed={picked}
			aria-label={dayName(cell)}
			onClick={onPick === undefined ? undefined : () => onPick(cell.iso)}
			className={`relative flex ${CONTROL_H} w-full cursor-pointer items-center justify-center text-ui-body ${ink} ${ground} ${ring}`}
		>
			<span
				className={
					cell.today
						? 'flex size-6 items-center justify-center rounded-full border border-ui-ink'
						: ''
				}
			>
				<Figure>{cell.date}</Figure>
			</span>

			{picked ? (
				<span
					aria-hidden
					className="pointer-events-none absolute inset-x-0 bottom-0 h-0.5 bg-ui-ink"
				/>
			) : null}
			{cell.also && !picked ? (
				<span
					aria-hidden
					className="pointer-events-none absolute inset-x-0 bottom-0 h-0.5 bg-ui-ink-3"
				/>
			) : null}
			{cell.booked !== null && !picked && !cell.also ? (
				<span aria-hidden className="pointer-events-none absolute bottom-0 h-0.5 w-5 bg-ui-ink-3" />
			) : null}
		</button>
	);
}

/** Which month, and the way to the ones either side. */
export function MonthHead({
	page,
	onStep,
	className = '',
}: {
	page: MonthPage;
	onStep?: (by: number) => void;
	className?: string;
}) {
	return (
		<div className={`flex items-center justify-between ${className}`}>
			<IconButton
				label={`Go back to ${monthStep(page, -1)}`}
				onClick={onStep === undefined ? undefined : () => onStep(-1)}
			>
				<span className="text-ui-lead">‹</span>
			</IconButton>
			<span className="text-ui-lead font-medium text-ui-ink">{page.label}</span>
			<IconButton
				label={`Go on to ${monthStep(page, 1)}`}
				onClick={onStep === undefined ? undefined : () => onStep(1)}
			>
				<span className="text-ui-lead">›</span>
			</IconButton>
		</div>
	);
}

/** The initials over their one hairline — the only rule on the control — and the days under it. */
export function MonthGrid({
	page,
	onPick,
	className = '',
}: {
	page: MonthPage;
	onPick?: (iso: string) => void;
	className?: string;
}) {
	return (
		<div className={className}>
			<div className="grid grid-cols-7 border-b border-ui-rule pb-1.5">
				{WEEKDAYS.map(day => (
					<span key={day} className="text-center text-ui-body text-ui-ink-3">
						{day}
					</span>
				))}
			</div>
			<div className="grid grid-cols-7 gap-y-1 pt-2">
				{page.weeks.map(week =>
					week.map(cell => <MonthDay key={cell.iso} cell={cell} onPick={onPick} />)
				)}
			</div>
		</div>
	);
}

export function MonthPicker({
	page,
	onStep,
	onPick,
	className = '',
}: {
	page: MonthPage;
	onStep?: (by: number) => void;
	onPick?: (iso: string) => void;
	className?: string;
}) {
	return (
		<div className={className}>
			<MonthHead page={page} onStep={onStep} />
			<MonthGrid page={page} onPick={onPick} />
		</div>
	);
}
