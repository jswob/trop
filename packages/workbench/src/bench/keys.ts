import { VIEWPORT_NAMES } from '../lib/viewports';

import { ACROSS, MOTION, acrossOf } from './Ops';
import { all, step, withOption, type BenchSearch } from './search';

const after = <T>(list: readonly T[], index: number): T | undefined =>
	list[(index + 1) % list.length];

/**
 * What one key does to the stage, as the address it leaves. Nothing for a key the bar does not
 * print.
 *
 * Each key is the one printed on its switch, so the bar is the only place they are written down.
 */
export function keyAction(
	key: string,
	ids: readonly string[],
	search: BenchSearch,
	/** Which switches the page's bar has. A key whose switch is not printed does nothing. */
	has: { modes?: boolean; parts?: boolean; notes?: boolean } = {}
): BenchSearch | undefined {
	const widths = search.show === 'widths';
	const modes = has.modes ?? true;

	switch (key.length === 1 ? key.toLowerCase() : key) {
		case ']':
			return step(ids, search, 1);
		case '[':
			return step(ids, search, -1);
		case 'a':
			return all(search);
		case 'w':
			return modes ? withOption(search, { show: widths ? undefined : 'widths' }) : undefined;
		case 'p':
			return has.parts === true
				? withOption(search, { parts: search.parts === 'on' ? undefined : 'on' })
				: undefined;
		case 'n':
			return has.notes === true
				? withOption(search, { notes: search.notes === 'on' ? undefined : 'on' })
				: undefined;
		case 't':
			return withOption(search, { theme: search.theme === 'light' ? 'dark' : 'light' });
		case 'm': {
			const now = MOTION.findIndex(option => option.rate === (search.rate ?? 1));
			return withOption(search, { t: undefined, rate: after(MOTION, now)?.rate });
		}
		case 'f': {
			if (widths || !modes) return undefined;
			const now = ACROSS.findIndex(option => option.across === acrossOf(search.scale));
			return withOption(search, { scale: after(ACROSS, now)?.scale });
		}
		case '1':
		case '2':
		case '3':
			return widths ? undefined : withOption(search, { width: VIEWPORT_NAMES[Number(key) - 1] });
		default:
			return undefined;
	}
}
