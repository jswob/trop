import type { KitEntry } from '@jswob/design-workbench';

/**
 * The kit, as a catalogue.
 *
 * Each entry's page is its specimen, beside the component it draws (`Button.specimen.tsx`), and
 * `specimens/index.test` fails when the two stop listing the same things.
 *
 * A component is listed the day it enters the kit — which is on its second caller, never its
 * first. The page prints the count of callers rather than rounding it up, so a component promoted
 * early says so.
 *
 * **The frame is the standing exception to the second-caller rule, and the page shows it rather
 * than hides it.** `SplitShell` reports one caller and always will: exactly one file outside the
 * kit may draw chrome, so a frame with two callers would mean two places disagreeing about where
 * you are. `PageSpread` has one for the same reason — the shell is the only thing that puts a page
 * in the frame. The rule exists to stop a part of one screen being promoted on a guess about the
 * next; a frame is not a part of a screen, it is the screen.
 *
 * **The symbol list is the thing that renders, not everything the module exports.** A type another
 * module names in an annotation is not a caller, and counting it would print callers a component
 * does not have. The number the rule turns on is only worth reading if it counts uses, and a
 * specimen is documentation, not use (`lib/kit-callers`).
 */
export const KIT_ENTRIES: KitEntry[] = [
	{
		id: 'month-picker',
		name: 'MonthPicker',
		files: ['packages/ui/src/kit/MonthPicker.tsx', 'packages/ui/src/kit/month.ts'],
		symbols: ['MonthPicker', 'MonthGrid', 'MonthHead', 'MonthDay'],
		line: 'A range of days, chosen by eye against what is already taken.',
	},
	{
		id: 'split-shell',
		name: 'SplitShell · useSplit',
		files: ['packages/ui/src/kit/SplitShell.tsx'],
		symbols: ['SplitShell', 'useSplit'],
		line: 'The frame: the ledger on the left, one destination on the right, and the rail that switches between them standing on the split. Controlled — useSplit is the behaviour that goes with it.',
	},
	{
		id: 'page-spread',
		name: 'PageSpread · PageColumn · PageSection',
		files: ['packages/ui/src/kit/PageSpread.tsx', 'packages/ui/src/kit/PageSection.tsx'],
		symbols: ['PageSpread', 'PageColumn', 'PageHead', 'PageSection', 'PageUnit'],
		line: 'The page that goes in the frame’s right half: it opens with its own name, its columns are set at the measure, and how many there are is what the flow comes to rather than a setting. A page is built of sections, and a section says for itself what the column rule may cut.',
	},
	{
		id: 'paged-list',
		name: 'PagedList · usePagedList',
		files: ['packages/ui/src/kit/PagedList.tsx', 'packages/ui/src/kit/paged-list.ts'],
		symbols: ['PagedList', 'usePagedList'],
		line: 'A list of records of unknown length: found by name, sorted by any column, read twenty at a time in one continuous column. Every name is the way to its record’s own page; nothing opens in place. Controlled — usePagedList is the behaviour that goes with it.',
	},
	{
		id: 'plinth',
		name: 'Plinth',
		files: ['packages/ui/src/kit/Plinth.tsx'],
		symbols: ['Plinth', 'BAND_CAP', 'HoldingProvider'],
		line: 'The foot of a thing: every control that finishes, leaves or destroys what is above it, as a ruled block rather than a row. Three to a band, the destroying member alone below at every width, and one slot for the reason a member cannot be pressed.',
	},
	{
		id: 'open-rows',
		name: 'OpenBlock · ActRow · AddRow',
		files: ['packages/ui/src/kit/OpenRows.tsx'],
		symbols: ['OpenBlock', 'OpenThread', 'ActRow', 'AddRow', 'RowGap', 'OPEN_ROW'],
		line: 'What a row of a list opens: more rows in the list’s own grid, each its words and the one word its press does.',
	},
	{
		id: 'period-sentence',
		name: 'PeriodSentence',
		files: ['packages/ui/src/kit/PeriodSentence.tsx', 'packages/ui/src/kit/period.ts'],
		symbols: ['PeriodSentence'],
		line: 'A list of periods stated as the sentence that is also its control: each end a press, a pressed end’s month grid under the whole sentence, the thread tying the two to their row.',
	},
	{
		id: 'waiting',
		name: 'Waiting · Placeholder',
		files: ['packages/ui/src/kit/Waiting.tsx'],
		symbols: ['Waiting', 'Placeholder', 'PlaceholderRows', 'WaitingNote'],
		line: 'What a page draws while it waits: footprints under one glaze passing down the page, and a note that prints the word and counts the seconds.',
	},
	{
		id: 'rail-shell',
		name: 'RailShell',
		files: ['packages/ui/src/kit/RailShell.tsx'],
		symbols: ['RailShell', 'RailIcon'],
		line: 'The below-lg half of the frame, and a whole chrome in its own right: one slug at the foot that the rails fold into, and the register that raises out of it.',
	},
	{
		id: 'ruled-scroll',
		name: 'RuledScroll',
		files: ['packages/ui/src/kit/RuledScroll.tsx'],
		symbols: ['RuledScroll'],
		line: 'A scroll region with its own bar off; the page-break rule that ends it thickens over the share instead.',
	},
	{
		id: 'button',
		name: 'Button · IconButton · TextButton',
		files: ['packages/ui/src/kit/Button.tsx'],
		symbols: ['Button', 'IconButton', 'ICON_BOX', 'TextButton'],
		line: 'Every action a hand can take, at three rungs. Emphasis is how much surface the button owns — a plate, a hairline box, a bare word — and no rung is a hue. The text button is the quiet rung with its padding taken back out.',
	},
	{
		id: 'field',
		name: 'Field · TextField · Select',
		files: ['packages/ui/src/kit/Field.tsx', 'packages/ui/src/kit/Select.tsx'],
		symbols: ['Field', 'TextField', 'Select', 'useSelect', 'controlBox'],
		line: 'What a field is: one box with an edge on all four sides, the free text that fills it, and the select that does not change size when it opens — or move anything under it when it does.',
	},
	{
		id: 'figure',
		name: 'Figure',
		files: ['packages/ui/src/kit/Figure.tsx'],
		symbols: ['Figure'],
		line: 'Tabular figures, for numbers that have to line up down a column.',
	},
	{
		id: 'mark',
		name: 'Mark',
		files: ['packages/ui/src/kit/Mark.tsx'],
		symbols: ['Mark'],
		line: 'The placeholder glyph, a filled square beside an open one, at the rail’s size and at sign-in’s.',
	},
	{
		id: 'modal',
		name: 'Modal',
		files: ['packages/ui/src/kit/Modal.tsx'],
		symbols: ['Modal'],
		line: 'The one overlay. For a decision that cannot be undone, and for nothing else.',
	},
	{
		id: 'constants',
		name: 'FOCUS · CONTROL_H',
		files: ['packages/ui/src/kit/focus.ts', 'packages/ui/src/kit/control.ts'],
		symbols: ['FOCUS', 'CONTROL_H'],
		line: 'The two class constants every control shares: a visible ring, and 40px below lg / 32px above.',
	},
];
