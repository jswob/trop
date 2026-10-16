import type { ViewportName } from '../lib/viewports';

/** What a measure has to come to: a number of px, a computed value, or a range. */
export type Expected = number | string | { min?: number; max?: number };

/** One expectation for every width, or one a width. A width left out is not checked. */
export type AtWidths = Expected | Partial<Record<ViewportName, Expected>>;

/**
 * What one named part has to measure.
 *
 * `width`, `height`, `lines` and `chars` are read off each drawing of the part, `count` is how many
 * drawings there are, and any other name is the computed style of that name (`font-size`,
 * `box-shadow`). Every drawing of the part has to hold.
 */
export type PartCheck = {
	part: string;
	/** The states it is checked in. Absent, it is every state that draws the part. */
	in?: readonly string[];
	expect: Record<string, AtWidths>;
};

/**
 * The room between two parts as a reader sees it: from the foot of the first to the head of the
 * second, or across. A part's own padding counts as room where it draws no edge and no ground on
 * that side, so the number does not depend on which of the two the padding was written on.
 */
export type GapCheck = {
	gap: readonly [from: string, to: string];
	/** Measured from the first part's right edge to the second's left, not top to bottom. */
	across?: true;
	in?: readonly string[];
	is: AtWidths;
};

export type Check = PartCheck | GapCheck;

/**
 * The numbers a thesis committed to, written down when it is approved and before anything is built.
 *
 * It is what makes the check independent of the build: whoever builds the candidate cannot bend a
 * number that was fixed before they started. A part is found by `data-part="<name>"` in the frame,
 * which the build puts there, unless `parts` names a selector for it.
 */
export type Thesis = {
	/** The frame id it is checked against, in every state that id has. */
	id: string;
	/** The one-line bet, as the board printed it. */
	bet: string;
	/** A selector for a part that is not marked with `data-part`. */
	parts?: Record<string, string>;
	checks: readonly Check[];
};

/** Declares a thesis file's export, typed. */
export const thesis = (declared: Thesis): Thesis => declared;

/** One drawing of a part, as the frame reports it. */
export type PartReading = {
	width: number;
	height: number;
	top: number;
	right: number;
	bottom: number;
	left: number;
	/** How many lines its text runs to. */
	lines: number;
	chars: number;
	/**
	 * On each side, how much of the box is room and not part: its padding, where it draws no edge
	 * and no ground of its own there. A head with 16px under its words and nothing drawn around
	 * them shows 16px of room, whichever element the padding was written on.
	 */
	room: { top: number; right: number; bottom: number; left: number };
	/** The computed styles that were asked for, by property. */
	style: Record<string, string>;
};

/** Every part a thesis names, read in one state at one width. */
export type FrameReading = {
	state: string;
	width: ViewportName;
	parts: Record<string, PartReading[]>;
};

/** What the page is asked for: a part, where to find it, and which computed styles to read. */
export type Ask = { name: string; select: string; props: string[] };

/** What one check came to across every state and width it was measured in. */
export type Verdict = {
	/** `row height`, `head to row`. */
	what: string;
	/** What the thesis said, in words. */
	said: string;
	ok: boolean;
	/** In how many states and widths it was measured. */
	measured: number;
	/** What it measured where it held: `28`, or `28 to 28.3`. */
	held: string;
	/** One line for each place it did not hold. */
	misses: string[];
};

const BOX = ['width', 'height', 'lines', 'chars'] as const;
type BoxMeasure = (typeof BOX)[number];

const isBox = (measure: string): measure is BoxMeasure =>
	(BOX as readonly string[]).includes(measure);
const isGap = (check: Check): check is GapCheck => 'gap' in check;

/** Half a pixel: what sub-pixel layout moves a box by without anyone having changed it. */
const SLACK = 0.5;

/** The parts a thesis names, each with its selector and the styles its checks read. */
export function asks(thesis: Thesis): Ask[] {
	const found = new Map<string, Ask>();

	const ask = (name: string): Ask => {
		const known = found.get(name);
		if (known !== undefined) return known;

		const made = { name, select: thesis.parts?.[name] ?? `[data-part~="${name}"]`, props: [] };
		found.set(name, made);
		return made;
	};

	for (const check of thesis.checks) {
		if (isGap(check)) {
			check.gap.forEach(ask);
			continue;
		}

		const part = ask(check.part);
		for (const measure of Object.keys(check.expect)) {
			if (measure !== 'count' && !isBox(measure) && !part.props.includes(measure)) {
				part.props.push(measure);
			}
		}
	}

	return [...found.values()];
}

const isRange = (expected: Expected): expected is { min?: number; max?: number } =>
	typeof expected === 'object';

function isPerWidth(value: AtWidths): value is Partial<Record<ViewportName, Expected>> {
	return typeof value === 'object' && ['mobile', 'tablet', 'desktop'].some(width => width in value);
}

function expectedAt(value: AtWidths, width: ViewportName): Expected | undefined {
	return isPerWidth(value) ? value[width] : value;
}

const shown = (value: number | string): string =>
	typeof value === 'number' ? String(Math.round(value * 10) / 10) : value;

function sayOne(expected: Expected): string {
	if (!isRange(expected)) return shown(expected);
	if (expected.min !== undefined && expected.max !== undefined) {
		return `from ${shown(expected.min)} to ${shown(expected.max)}`;
	}
	if (expected.max !== undefined) return `at most ${shown(expected.max)}`;
	return `at least ${shown(expected.min ?? 0)}`;
}

function say(value: AtWidths): string {
	if (!isPerWidth(value)) return sayOne(value);

	return Object.entries(value)
		.map(([width, expected]) => `${sayOne(expected)} at ${width}`)
		.join(', ');
}

const plain = (text: string): string => text.trim().toLowerCase().replace(/\s+/g, ' ');

function holds(found: number | string, expected: Expected): boolean {
	if (typeof expected === 'string') return plain(String(found)) === plain(expected);

	const number = typeof found === 'number' ? found : Number.parseFloat(found);
	if (!Number.isFinite(number)) return false;
	if (!isRange(expected)) return Math.abs(number - expected) <= SLACK;

	return (
		(expected.min === undefined || number >= expected.min - SLACK) &&
		(expected.max === undefined || number <= expected.max + SLACK)
	);
}

/** What held, said once: a number as the span it ran over, anything else as itself. */
function span(values: (number | string)[]): string {
	const numbers = values.filter((value): value is number => typeof value === 'number');
	if (numbers.length === 0 || numbers.length !== values.length) {
		return [...new Set(values.map(shown))].join(', ');
	}

	const least = Math.min(...numbers);
	const most = Math.max(...numbers);
	return shown(least) === shown(most) ? shown(least) : `${shown(least)} to ${shown(most)}`;
}

const where = (reading: FrameReading): string => `in ${reading.state} at ${reading.width}`;

function measure(part: PartReading, name: string): number | string {
	if (isBox(name)) return part[name];

	const value = part.style[name] ?? '';
	const number = Number.parseFloat(value);
	return /^-?[\d.]+(px)?$/.test(value.trim()) && Number.isFinite(number) ? number : value;
}

function judgePart(check: PartCheck, name: string, readings: FrameReading[]): Verdict {
	const expected = check.expect[name] ?? '';
	const verdict: Verdict = {
		what: `${check.part} ${name}`,
		said: say(expected),
		ok: true,
		measured: 0,
		held: '',
		misses: [],
	};
	const held: (number | string)[] = [];
	let drawn = false;

	for (const reading of readings) {
		if (check.in !== undefined && !check.in.includes(reading.state)) continue;

		const wanted = expectedAt(expected, reading.width);
		if (wanted === undefined) continue;

		const parts = reading.parts[check.part] ?? [];
		if (parts.length > 0) drawn = true;

		if (name === 'count') {
			verdict.measured += 1;
			if (holds(parts.length, wanted)) held.push(parts.length);
			else verdict.misses.push(`${String(parts.length)}, ${where(reading)}`);
			continue;
		}

		if (parts.length === 0) {
			if (check.in !== undefined) verdict.misses.push(`not drawn, ${where(reading)}`);
			continue;
		}

		verdict.measured += 1;
		const off = new Map<string, number>();
		for (const part of parts) {
			const found = measure(part, name);
			if (holds(found, wanted)) held.push(found);
			else off.set(shown(found), (off.get(shown(found)) ?? 0) + 1);
		}

		for (const [found, count] of off) {
			verdict.misses.push(
				`${found} on ${String(count)} of ${String(parts.length)}, ${where(reading)}`
			);
		}
	}

	if (name !== 'count' && !drawn && verdict.misses.length === 0) {
		verdict.misses.push(`no state draws a part called "${check.part}"`);
	}

	verdict.ok = verdict.misses.length === 0;
	verdict.held = span(held);
	return verdict;
}

function judgeGap(check: GapCheck, readings: FrameReading[]): Verdict {
	const [from, to] = check.gap;
	const verdict: Verdict = {
		what: `${from} to ${to}`,
		said: say(check.is),
		ok: true,
		measured: 0,
		held: '',
		misses: [],
	};
	const held: number[] = [];

	for (const reading of readings) {
		if (check.in !== undefined && !check.in.includes(reading.state)) continue;

		const wanted = expectedAt(check.is, reading.width);
		if (wanted === undefined) continue;

		const first = reading.parts[from]?.[0];
		const second = reading.parts[to]?.[0];
		if (first === undefined || second === undefined) {
			if (check.in !== undefined) verdict.misses.push(`not drawn, ${where(reading)}`);
			continue;
		}

		verdict.measured += 1;
		const found =
			check.across === true
				? second.left + second.room.left - (first.right - first.room.right)
				: second.top + second.room.top - (first.bottom - first.room.bottom);
		if (holds(found, wanted)) held.push(found);
		else verdict.misses.push(`${shown(found)}, ${where(reading)}`);
	}

	if (verdict.measured === 0 && verdict.misses.length === 0) {
		verdict.misses.push(`no state draws both "${from}" and "${to}"`);
	}

	verdict.ok = verdict.misses.length === 0;
	verdict.held = span(held);
	return verdict;
}

/**
 * Holds a thesis's numbers against what the frames measured.
 *
 * A check on a part that nothing draws fails: a number nobody can measure would otherwise pass for
 * ever, which is how a renamed part turns a check into decoration.
 */
export function judge(thesis: Thesis, readings: FrameReading[]): Verdict[] {
	return thesis.checks.flatMap(check =>
		isGap(check)
			? [judgeGap(check, readings)]
			: Object.keys(check.expect).map(name => judgePart(check, name, readings))
	);
}
