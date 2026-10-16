import { describe, expect, it } from 'vitest';

import { parseSections, preambleOf, sectionsFor, type SectionLookup } from './bans';

const FILE = `# Bans — Ledger

Concrete rules. Read at the start of every session.

---

## 1. Hue and status

- **Hue has four roles.**   (37)

## 4. Density and controls

- **28px rows.**   (35)

## 5. The table

- **The table is 34rem.**   (39)

## 10b. Waiting

- **A blank page is a loader.**   (44)

## 11. Settled — do not re-open

- **The frame is settled.**   (29)

## 12. Pre-seeded

- **No Inter.**   (0)
`;

describe('parseSections', () => {
	it('cuts the file at its headings and keeps the letter suffixes', () => {
		expect(parseSections(FILE).map(section => section.number)).toEqual([
			'1',
			'4',
			'5',
			'10b',
			'11',
			'12',
		]);
	});

	it('gives each section the range that holds its rules and nothing else', () => {
		const table = parseSections(FILE).find(section => section.number === '5');
		const text = FILE.slice(table?.start, table?.end);

		expect(text).toContain('The table is 34rem');
		expect(text).not.toContain('A blank page is a loader');
	});
});

describe('sectionsFor', () => {
	const sections = parseSections(FILE);
	const lookup: SectionLookup = {
		subjects: { grid: ['table'], colour: ['hue'], loading: ['waiting'] },
		always: ['11', '12'],
	};
	const numbers = (terms: string[], given = lookup): string[] =>
		sectionsFor(sections, terms, given).map(section => section.number);

	it('resolves a subject word to the section that holds it', () => {
		expect(numbers(['table'])).toContain('5');
	});

	it('resolves the words a round actually uses, not only the section titles', () => {
		expect(numbers(['grid'])).toContain('5');
		expect(numbers(['colour'])).toContain('1');
		expect(numbers(['loading'])).toContain('10b');
	});

	it('takes a section number directly', () => {
		expect(numbers(['10b'])).toContain('10b');
	});

	/**
	 * **The guard on the one change here with a quality risk.** A round that asks for the table and
	 * gets only the table can re-propose something §11 closed. The sections the config names as
	 * always go in whatever was asked for, and `banFind` names every section it left out.
	 */
	it('always includes the sections it is told every round reads, whatever was asked for', () => {
		const chosen = numbers(['table']);

		expect(chosen).toContain('11');
		expect(chosen).toContain('12');
		expect(numbers(['table'], { subjects: {}, always: [] })).toEqual(['5']);
	});

	it('returns sections in file order, not in the order they were asked for', () => {
		expect(numbers(['table', 'hue'])).toEqual(['1', '5', '11', '12']);
	});

	it('refuses a word it cannot place and prints the menu', () => {
		expect(() => numbers(['vibes'])).toThrow(/no section matches "vibes"/);
		expect(() => numbers(['vibes'])).toThrow(/4\. Density and controls/);
	});
});

describe('preambleOf', () => {
	it('is everything above the first section, which carries the format rule', () => {
		const preamble = preambleOf(FILE, parseSections(FILE));

		expect(preamble).toContain('Read at the start of every session.');
		expect(preamble).not.toContain('## 1.');
	});
});
