import path from 'node:path';

import type { HeadTag } from '../config';

/**
 * What `workbench board` writes, as text — the part of the verb that can be read without a build.
 *
 * A board is a page published from a folder outside the repo. Until this verb, every drawing on it
 * was redrawn by hand in plain HTML with the tokens copied by value, so the thing approved at step
 * 2 was a look-alike of the thing built at step 3. The kit this writes puts the workbench's own
 * frames on the page instead: the same components, the same stylesheet, the same React.
 *
 * The transforms take their inputs and return source, so they are tested without a filesystem or a
 * bundler; `board.ts` is the half that touches both.
 */

/** Where the kit keeps what it generates before building, inside the board's folder. */
export const KIT_DIR = '.kit';

/** What the kit leaves beside a board's page, and what the page is published with. */
export const KIT_FILES = ['frame.js', 'frame.css', 'frame.html', 'board.js'] as const;

/**
 * The board's folder, resolved — and refused when it is not somewhere a board may be.
 *
 * A board is never in the repo (`BOARDS.md`): nothing imports it, registers it or promotes it, and
 * a kit built into the tree would leave 600 KB of bundle for the blast-radius check to find.
 */
export function assertBoardDir(
	dir: string | undefined,
	repoRoot: string,
	exists: (dir: string) => boolean
): string {
	if (dir === undefined || dir === '') {
		throw new Error(`name the board's folder: bun workbench board "$CLAUDE_JOB_DIR/tmp/round-78"`);
	}

	const resolved = path.resolve(dir);
	const fromRepo = path.relative(repoRoot, resolved);
	if (fromRepo === '' || (!fromRepo.startsWith('..') && !path.isAbsolute(fromRepo))) {
		throw new Error(
			`a board lives outside the repo, in the session's scratchpad — ${resolved} is inside it`
		);
	}
	if (!exists(resolved)) throw new Error(`no folder at ${resolved} — make it, then run this again`);

	return resolved;
}

/** The board's own drawing files: every `.tsx` directly in its folder, in a stable order. */
export function drawingFiles(names: string[]): string[] {
	return names.filter(name => name.endsWith('.tsx') && !name.startsWith('.')).sort();
}

/**
 * The frame's entry: the stylesheet, then `startFrame` on every drawing file the board has.
 *
 * It is written into `.kit/`, one folder below the drawings, which is why they are imported from
 * `../`. The library is named by its absolute path because this file is outside the repo.
 */
export function frameEntry(library: string, drawings: string[]): string {
	const imports = drawings.map((file, index) => `import * as drawing${index} from '../${file}';`);
	const list = drawings.map((_file, index) => `drawing${index}`).join(', ');

	return [
		`import './entry.css';`,
		`import { startFrame } from '${library}';`,
		...imports,
		'',
		`startFrame([${list}]);`,
		'',
	].join('\n');
}

/** The page's entry. Its two exports become `Board.mount` and `Board.theme`. */
export function pageEntry(library: string): string {
	return [
		`import { startBoard } from '${library}';`,
		'',
		'export const { mount, theme } = startBoard(document);',
		'',
	].join('\n');
}

/**
 * The frame's stylesheet: the one every frame of the workbench loads, which the app's config names.
 *
 * The chrome's own sheet is not in it, so a frame on a board has the app's `body` and nothing of
 * the bench's. The `@source` line adds the board: a stylesheet compiled from the classes it finds
 * has to be shown a drawing that may use one nothing in the repo has used yet.
 */
export function styleEntry(kitDir: string, styles: string): string {
	const sheet = path.relative(kitDir, styles).split(path.sep).join('/');

	return [`@import '${sheet}';`, '', `@source '../*.{ts,tsx,html}';`, ''].join('\n');
}

/**
 * The stylesheets the app's config puts in the document's head: where an app names its faces.
 *
 * Read off the same list the workbench's own document is served with, so a frame on a board loads
 * the faces a frame on the bench does.
 */
export function sheetHrefs(head: HeadTag[]): string[] {
	return head.flatMap(tag => {
		const href = tag.attrs?.['href'];
		return tag.tag === 'link' && tag.attrs?.['rel'] === 'stylesheet' && typeof href === 'string'
			? [href]
			: [];
	});
}

/** The document every frame on a board opens. What it draws is in the `name` its frame was given. */
export function frameHtml(fonts: string[]): string {
	const faces = fonts.map(
		href => `<link rel="stylesheet" href="${href.replaceAll('&', '&amp;')}">`
	);

	return [
		'<!doctype html>',
		'<html lang="en">',
		'<head>',
		'<meta charset="utf-8">',
		'<meta name="viewport" content="width=device-width, initial-scale=1">',
		'<title>Frame</title>',
		...faces,
		'<link rel="stylesheet" href="frame.css">',
		'</head>',
		'<body>',
		'<div id="root"></div>',
		'<script src="frame.js"></script>',
		'</body>',
		'</html>',
		'',
	].join('\n');
}
