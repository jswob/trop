/**
 * Re-cuts this template from the app it was made from.
 *
 *   bun template/cut.ts <source repo> [<commit>]
 *
 * Every file the source tracks at the commit is copied here, renamed by the manifest's rules, except
 * what the manifest excludes. A file the manifest marks as the template's own (`owned`, `seed`) is
 * copied only while it is missing here, and never overwritten or deleted after. A file this repo
 * tracks that the cut did not write, and that is not the template's own, is deleted: what the
 * source dropped leaves the template too. The report lists the owned files the source changed
 * since the last cut, so their changes can be carried over by hand; seeded ones are never listed.
 *
 * `template/source.json` records the commit, so the next cut is a diff against this one.
 */

import { spawnSync } from 'node:child_process';
import {
	chmodSync,
	existsSync,
	mkdirSync,
	mkdtempSync,
	readFileSync,
	rmSync,
	statSync,
	writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

import manifest, { type Manifest } from './manifest';

type Source = { repo: string; commit: string; cutAt: string };

const TEMPLATE = path.resolve(import.meta.dirname, '..');
const RECORD = path.join(TEMPLATE, 'template', 'source.json');

function git(cwd: string, args: string[]): string {
	const run = spawnSync('git', args, { cwd, encoding: 'utf8', maxBuffer: 1 << 28 });
	if (run.status !== 0) throw new Error(`git ${args.join(' ')} failed in ${cwd}:\n${run.stderr}`);
	return run.stdout;
}

/** Node's and Bun's `path.matchesGlob`, so the cut's rules run under either (its tests run on Node). */
const matches = (file: string, patterns: readonly string[]) =>
	patterns.some(pattern => path.matchesGlob(file, pattern));

/** The template's path for a source path: the first rule whose prefix it starts with. */
export function renamePath(file: string, rules: Manifest['paths']): string {
	const rule = rules.find(([from]) => file.startsWith(from));
	return rule === undefined ? file : rule[1] + file.slice(rule[0].length);
}

/** A text file's contents with every rule that applies to its (template) path. */
export function renameText(file: string, text: string, rules: Manifest['text']): string {
	return rules.reduce((out, rule) => {
		if (rule.only !== undefined && !matches(file, rule.only)) return out;
		return out.replace(rule.from, rule.to);
	}, text);
}

/** Every file here that git keeps or would keep. */
function present(): string[] {
	const files = git(TEMPLATE, ['ls-files', '--cached', '--others', '--exclude-standard']);
	return [...new Set(files.split('\n'))]
		.filter(file => file !== '' && existsSync(path.join(TEMPLATE, file)))
		.sort();
}

/** Each line here, outside the cut's own files, that holds one of the source's words. */
function wordHits(): string[] {
	const { list, allow } = manifest.words;
	const pattern = new RegExp(
		`(?<![\\p{L}\\p{N}])(${list.map(w => w.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')).join('|')})`,
		'giu'
	);
	const kept = manifest.keep;
	const files = git(TEMPLATE, ['ls-files', '--cached', '--others', '--exclude-standard']).split(
		'\n'
	);
	return [...new Set(files)].sort().flatMap(file => {
		const full = path.join(TEMPLATE, file);
		if (file === '' || matches(file, kept) || !existsSync(full)) return [];
		const bytes = readFileSync(full);
		if (bytes.includes(0)) return [];
		return bytes
			.toString('utf8')
			.split('\n')
			.flatMap((line, index) =>
				[...line.matchAll(pattern)]
					.map(match => (match[1] ?? '').toLowerCase())
					.filter(word => !matches(file, allow[word] ?? []))
					.map(word => `${file}:${index + 1} ${word}`)
			);
	});
}

function previousCut(): Source | null {
	return existsSync(RECORD) ? (JSON.parse(readFileSync(RECORD, 'utf8')) as Source) : null;
}

function cut(sourceRepo: string, commitish: string) {
	const repo = path.resolve(sourceRepo);
	const commit = git(repo, ['rev-parse', `${commitish}^{commit}`]).trim();
	const before = previousCut();

	const { exclude, owned, seed, keep: kept } = manifest;

	const tree = mkdtempSync(path.join(tmpdir(), 'template-cut-'));
	try {
		const archive = spawnSync('git', ['archive', '--format=tar', commit], {
			cwd: repo,
			maxBuffer: 1 << 30,
		});
		if (archive.status !== 0) throw new Error(`git archive failed: ${String(archive.stderr)}`);
		const untar = spawnSync('tar', ['-x', '-C', tree], { input: archive.stdout });
		if (untar.status !== 0) throw new Error(`tar failed: ${String(untar.stderr)}`);

		const written = new Set<string>();
		const seeded: string[] = [];
		const ownedInSource = new Map<string, string>();
		for (const from of git(repo, ['ls-tree', '-r', '--name-only', commit]).split('\n')) {
			if (from === '' || matches(from, exclude)) continue;
			const to = renamePath(from, manifest.paths);
			if (matches(to, kept)) continue;

			const target = path.join(TEMPLATE, to);
			const own = matches(to, owned);
			if (own || matches(to, seed)) {
				if (own) ownedInSource.set(from, to);
				// The template's own from the moment it exists here: only a missing one is copied
				if (existsSync(target)) continue;
				seeded.push(to);
			} else {
				written.add(to);
			}

			const origin = path.join(tree, from);
			const bytes = readFileSync(origin);
			mkdirSync(path.dirname(target), { recursive: true });
			writeFileSync(
				target,
				bytes.includes(0) ? bytes : renameText(to, bytes.toString('utf8'), manifest.text)
			);
			chmodSync(target, statSync(origin).mode & 0o777);
		}

		const removed: string[] = [];
		for (const file of git(TEMPLATE, ['ls-files']).split('\n')) {
			if (file === '' || written.has(file) || matches(file, owned) || matches(file, seed)) continue;
			if (matches(file, kept)) continue;
			rmSync(path.join(TEMPLATE, file), { force: true });
			removed.push(file);
		}

		// Matched against the file list: a scan cannot follow a brace that spans folders
		const here = present();
		const missing = manifest.owned.filter(pattern => !here.some(file => matches(file, [pattern])));

		// A cut taken from a branch the source later rebased onto master can name a commit that is gone
		const beforeKnown =
			before !== null &&
			spawnSync('git', ['cat-file', '-e', `${before.commit}^{commit}`], { cwd: repo }).status === 0;
		const changedOwned =
			before === null || !beforeKnown || before.commit === commit
				? []
				: git(repo, ['diff', '--name-only', before.commit, commit])
						.split('\n')
						.filter(file => ownedInSource.has(file))
						.map(file => `${file} → ${ownedInSource.get(file)}`);

		const hits = wordHits();

		const record: Source = { repo: path.basename(repo), commit, cutAt: new Date().toISOString() };
		writeFileSync(RECORD, `${JSON.stringify(record, null, '\t')}\n`);

		console.log(
			`Cut ${record.repo}@${commit.slice(0, 7)}: ${written.size} files written, ${seeded.length} seeded, ${removed.length} removed.`
		);
		if (seeded.length > 0)
			console.log(`\nSeeded (the template's own from now on):\n  ${seeded.join('\n  ')}`);
		if (removed.length > 0) console.log(`\nRemoved:\n  ${removed.join('\n  ')}`);
		if (missing.length > 0)
			console.log(`\nOwned patterns matching no file here:\n  ${missing.join('\n  ')}`);
		if (hits.length > 0)
			console.log(`\nThe source's words, still here (${hits.length}):\n  ${hits.join('\n  ')}`);
		if (before === null)
			console.log('\nFirst cut: no earlier commit to compare the owned files against.');
		else if (!beforeKnown)
			console.log(
				`\nThe last cut's commit, ${before.commit.slice(0, 7)}, is not in the source any more: compare the owned files by hand.`
			);
		else if (changedOwned.length > 0)
			console.log(
				`\nThe source changed these since ${before.commit.slice(0, 7)}; they are the template's own, so carry the change over by hand:\n  ${changedOwned.join('\n  ')}`
			);
	} finally {
		rmSync(tree, { recursive: true, force: true });
	}
}

if (import.meta.main) {
	const [source, commit = 'HEAD'] = process.argv.slice(2);
	if (source === undefined) {
		console.error('Usage: bun template/cut.ts <source repo> [<commit>]');
		process.exit(1);
	}
	cut(source, commit);
}
