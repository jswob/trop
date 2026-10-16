import { readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';

import { flag, list, type Argv } from './args';
import { host } from './host';

export type Section = {
	/** `5`, `10b`, `10c` — the file's own numbering, letter suffixes and all. */
	number: string;
	title: string;
	heading: string;
	/** Character offsets into the source, `end` exclusive. */
	start: number;
	end: number;
};

/**
 * The bans file, cut at its `##` headings.
 *
 * **This is the one change here with a real quality risk, so it is worth stating what it does and
 * does not do.** `BUILD.md` is right that a ban cannot be summarised — the exact wording *is* the
 * check, and "no shadow on any surface" is checkable where "keep it flat" is not. Nothing here
 * summarises anything: `find` prints the sections verbatim. What it removes is the obligation to
 * read the whole file to build one component.
 *
 * The risk is an agent reading one section and missing a cross-cutting rule in another, so the
 * preamble and the sections the config's `bans.always` names (what is settled, what was
 * pre-seeded) are **always** included whatever was asked for, and the footer names every section
 * that was left out so nothing goes missing in silence.
 */
export function parseSections(source: string): Section[] {
	const matches = [...source.matchAll(/^## ([0-9]+[a-z]?)\. (.+)$/gm)];

	return matches.map((match, index) => ({
		number: match[1] ?? '',
		title: match[2] ?? '',
		heading: match[0],
		start: match.index ?? 0,
		end: matches[index + 1]?.index ?? source.length,
	}));
}

/** How a round's words find sections, from the config's `bans`. */
export type SectionLookup = {
	/** The words a round uses for a subject, mapped to words in the titles that hold its rules. */
	subjects: Record<string, string[]>;
	/** Sections that go in whatever was asked for, because they are about every round. */
	always: string[];
};

export function sectionsFor(
	sections: Section[],
	terms: string[],
	{ subjects, always }: SectionLookup
): Section[] {
	const wanted = new Set<string>(always);

	for (const term of terms) {
		const key = term.trim().toLowerCase();
		const byNumber = sections.find(section => section.number === key);
		if (byNumber !== undefined) {
			wanted.add(byNumber.number);
			continue;
		}

		const needles = subjects[key] ?? [key];
		let hit = false;
		for (const section of sections) {
			if (needles.some(needle => section.title.toLowerCase().includes(needle))) {
				wanted.add(section.number);
				hit = true;
			}
		}

		if (!hit) {
			throw new Error(
				`no section matches "${term}" — try one of ${sections.map(section => section.number).join(', ')}, or a word from a title:\n${sections.map(section => `  ${section.number}. ${section.title}`).join('\n')}`
			);
		}
	}

	return sections.filter(section => wanted.has(section.number));
}

/** The header above §1: the format rule, and what belongs in the other files. */
export function preambleOf(source: string, sections: Section[]): string {
	const first = sections[0];
	return source.slice(0, first?.start ?? 0).trimEnd();
}

async function readStdin(): Promise<string> {
	if (process.stdin.isTTY === true) return '';

	const chunks: Buffer[] = [];
	for await (const chunk of process.stdin) chunks.push(chunk as Buffer);
	return Buffer.concat(chunks).toString('utf8');
}

export function banFind(argv: Argv): number {
	const bans = host().bans;
	const source = readFileSync(bans.file, 'utf8');
	const sections = parseSections(source);
	const terms = [...list(argv, 'for'), ...list(argv, 'section')];

	if (terms.length === 0) {
		process.stdout.write(
			`${sections.map(section => `  ${section.number.padEnd(4)} ${section.title}`).join('\n')}\n\n` +
				`Name what the round is about, by section number or a word from a title:  workbench ban find --for ${sections
					.slice(0, 2)
					.map(section => section.number)
					.join(',')}\n` +
				`Everything, as the skill's own rule asks when a round is not scoped:  cat ${path.relative(process.cwd(), bans.file)}\n`
		);
		return 0;
	}

	const chosen = sectionsFor(sections, terms, bans);
	const omitted = sections.filter(section => !chosen.includes(section));

	process.stdout.write(`${preambleOf(source, sections)}\n\n`);
	for (const section of chosen) process.stdout.write(`${source.slice(section.start, section.end)}`);

	process.stdout.write(
		`\n---\n\nPrinted §${chosen.map(section => section.number).join(' §')} verbatim, plus the preamble.\n` +
			`**Not printed**, and a rule in one of these still binds: ${omitted.map(section => `§${section.number} ${section.title}`).join(' · ')}.\n` +
			`Read the whole file when the round crosses subjects — nothing here is a summary, but a\n` +
			`selection is still a selection.\n`
	);
	return 0;
}

/**
 * A ban, appended at the end of the section that owns it.
 *
 * The body comes in on stdin so it can be markdown with bullets and blockquotes without a shell
 * escaping layer. What this replaces is a `python3` heredoc that located an anchor, asserted it
 * appeared exactly once and spliced — the shape that appears 96 times in one round's transcript.
 */
export async function banAdd(argv: Argv): Promise<number> {
	const sectionKey = flag(argv, 'section');
	const round = flag(argv, 'round');
	const title = flag(argv, 'title');

	if (sectionKey === undefined || round === undefined || title === undefined) {
		throw new Error(
			'ban add needs --section, --round and --title, with the body on stdin:\n' +
				`  cat <<'MD' | bun workbench ban add --section 5 --round 50 --title "The table's measure"\n` +
				'  - **A rule you could check.**   (50 — "the user\'s own words")\n' +
				'  MD'
		);
	}

	const body = (await readStdin()).trim();
	if (body === '') throw new Error('the body is empty — pipe the rules in on stdin');

	const file = host().bans.file;
	const source = readFileSync(file, 'utf8');
	const sections = parseSections(source);
	const section = sections.find(entry => entry.number === sectionKey);
	if (section === undefined) {
		throw new Error(
			`no section ${sectionKey} — the file has ${sections.map(entry => entry.number).join(', ')}`
		);
	}

	const unmarked = body
		.split('\n')
		.filter(line => /^\s*-\s/.test(line) && !/\(\d+[a-z]?[\s—,)]/.test(line));

	const block = `\n### ${title} (${round})\n\n${body}\n`;
	const at = section.end;
	writeFileSync(file, `${source.slice(0, at).trimEnd()}\n${block}\n${source.slice(at)}`, 'utf8');

	process.stdout.write(
		`appended to §${section.number} ${section.title} as "### ${title} (${round})"\n`
	);

	if (unmarked.length > 0) {
		process.stdout.write(
			`\n  ${unmarked.length} rule${unmarked.length === 1 ? ' carries' : 's carry'} no round number. Each ban carries the round\n` +
				`  that produced it, so a later round can strike it and say which one killed it:\n${unmarked.map(line => `    ${line.trim().slice(0, 76)}`).join('\n')}\n`
		);
	}

	return 0;
}

export async function ban(argv: Argv): Promise<number> {
	const action = argv.positional[0] ?? 'find';

	if (action === 'find' || action === 'list') return banFind({ ...argv, positional: [] });
	if (action === 'add') return banAdd(argv);

	throw new Error(`ban takes "find" or "add", got "${action}"`);
}
