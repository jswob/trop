import { createHash } from 'node:crypto';
import path from 'node:path';

import type { Appearance } from '../lib/appearance';

import type { FrameEntry } from '../lib/frame-census';
import type { ViewportName } from '../lib/viewports';

/**
 * What `workbench shots` writes and compares, as names — the half of the verb that needs no
 * browser and no folder.
 *
 * A set of shots is the oracle for a step that must change nothing on screen: every frame at every
 * width in both appearances, taken before the step and again after it. Two sets are compared file
 * by file, so the name of a shot is the whole of its identity and a set is a flat folder.
 *
 * A set also keeps what other hosts answered while it was taken (`REMOTE_DIR`), and a later set is
 * answered from that. A font host was measured sending one of two files for the same address, one
 * load in twenty, and the letters moved half a pixel: two sets of the same code differed.
 */

/** Where a set keeps what other hosts answered, inside its folder. */
export const REMOTE_DIR = 'remote';

/** Both appearances, in the order a set is taken. */
export const APPEARANCES: readonly Appearance[] = ['dark', 'light'];

/** One picture: a frame's state at a width, in an appearance. */
export type Shot = { entry: FrameEntry; width: ViewportName; theme: Appearance };

/** The file a shot is written to. */
export function shotName({ entry, width, theme }: Shot): string {
	return `${[entry.kind, entry.id, entry.variant, width, theme].join('.')}.png`;
}

/**
 * Every shot of a set: each frame at each width in each appearance.
 *
 * Two frames that would be written to one file are refused, and so is a name a folder cannot hold:
 * a set with one picture standing for two frames checks one of them and says it checked both.
 */
export function shotsOf(
	census: FrameEntry[],
	widths: readonly ViewportName[],
	themes: readonly Appearance[]
): Shot[] {
	const shots = census.flatMap(entry =>
		widths.flatMap(width => themes.map(theme => ({ entry, width, theme })))
	);

	const taken = new Set<string>();
	for (const shot of shots) {
		const name = shotName(shot);
		const frame = `${shot.entry.id}:${shot.entry.variant}`;

		if (/[/\\]/.test(name)) throw new Error(`${frame} cannot be a file name: ${name}`);
		if (taken.has(name)) throw new Error(`two frames would be written to ${name} (${frame})`);
		taken.add(name);
	}

	return shots;
}

/**
 * The folder a set goes in, resolved — and refused when it is inside the repo.
 *
 * A set is some thousands of PNGs that nothing imports. In the tree they are what the blast-radius
 * check finds and what a careless `git add` commits.
 */
export function assertShotsDir(dir: string | undefined, repoRoot: string): string {
	if (dir === undefined || dir === '') {
		throw new Error('name a folder for the shots, outside the repo: bun workbench shots <folder>');
	}

	const resolved = path.resolve(dir);
	const fromRepo = path.relative(repoRoot, resolved);
	if (fromRepo === '' || (!fromRepo.startsWith('..') && !path.isAbsolute(fromRepo))) {
		throw new Error(`a set of shots lives outside the repo — ${resolved} is inside it`);
	}

	return resolved;
}

/** What separates two sets, by file name. */
export type SetDiff = {
	/** How many shots are in both sets and are the same picture. */
	same: number;
	/** In both sets, and not the same picture. */
	differs: string[];
	/** In the earlier set and not taken now. */
	missing: string[];
	/** Taken now and not in the earlier set. */
	added: string[];
};

/**
 * Two sets compared, each given as its files' names and digests.
 *
 * A run pointed at part of the set (`whole: false`) cannot say a shot is missing: it never tried
 * to take it.
 */
export function compareSets(
	before: ReadonlyMap<string, string>,
	after: ReadonlyMap<string, string>,
	{ whole }: { whole: boolean }
): SetDiff {
	const diff: SetDiff = { same: 0, differs: [], missing: [], added: [] };

	for (const [name, digest] of after) {
		const was = before.get(name);

		if (was === undefined) diff.added.push(name);
		else if (was === digest) diff.same += 1;
		else diff.differs.push(name);
	}

	if (whole) diff.missing = [...before.keys()].filter(name => !after.has(name));

	diff.differs.sort();
	diff.missing.sort();
	diff.added.sort();

	return diff;
}

/** The shots of a set that are written to the named files, in the set's order. */
export function shotsNamed(shots: Shot[], names: readonly string[]): Shot[] {
	return shots.filter(shot => names.includes(shotName(shot)));
}

/**
 * What differed on the first take and is the same on a later one.
 *
 * A frame that scrolls itself into place can land a pixel off when a face arrives late, which a
 * loaded machine does to one shot in a thousand. That is the frame racing and not the code
 * changing: code that changed a picture changes it on every take.
 */
export function steadied(first: SetDiff, last: SetDiff): string[] {
	return first.differs.filter(name => !last.differs.includes(name));
}

/**
 * A comparison as the terminal prints it: one line per file that is not the same, then the counts.
 *
 * A shot that only matched on a later take is named above them and counted as identical.
 */
export function diffLines(diff: SetDiff, unsteady: readonly string[] = []): string[] {
	const files = [
		...unsteady.map(
			name => `  unsteady ${name} (not the same on the first take, the same on a later one)`
		),
		...diff.differs.map(name => `  differs  ${name}`),
		...diff.missing.map(name => `  missing  ${name}`),
		...diff.added.map(name => `  added    ${name}`),
	];
	const total = diff.same + diff.differs.length + diff.missing.length + diff.added.length;
	const counts = `  ${String(total)} shots · ${String(diff.same)} identical · ${String(diff.differs.length)} ${diff.differs.length === 1 ? 'differs' : 'differ'} · ${String(diff.missing.length)} missing · ${String(diff.added.length)} added`;

	return files.length === 0 ? [counts] : [...files, '', counts];
}

/** What another host answered for one address. */
export type Answer = {
	url: string;
	status: number;
	headers: Record<string, string>;
	body: Buffer;
};

/** The file an address's answer is kept in. An address is not a file name, so its digest is. */
export function answerName(url: string): string {
	return `${createHash('sha256').update(url).digest('hex').slice(0, 32)}.json`;
}

/** Headers that describe a body in transit. A kept body is whole and unpacked, so they would lie. */
const IN_TRANSIT = ['content-encoding', 'content-length', 'transfer-encoding'];

/** An answer as it is written beside a set. */
export function packAnswer(answer: Answer): string {
	return `${JSON.stringify(
		{
			url: answer.url,
			status: answer.status,
			headers: Object.fromEntries(
				Object.entries(answer.headers).filter(([name]) => !IN_TRANSIT.includes(name.toLowerCase()))
			),
			body: answer.body.toString('base64'),
		},
		null,
		'\t'
	)}\n`;
}

/** An answer read back from its file. */
export function unpackAnswer(text: string): Answer {
	const kept = JSON.parse(text) as Omit<Answer, 'body'> & { body: string };

	return { ...kept, body: Buffer.from(kept.body, 'base64') };
}
