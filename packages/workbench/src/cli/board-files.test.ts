import { describe, expect, it } from 'vitest';

import {
	assertBoardDir,
	drawingFiles,
	frameEntry,
	frameHtml,
	pageEntry,
	sheetHrefs,
	styleEntry,
} from './board-files';

describe('assertBoardDir', () => {
	const repo = '/work/ledger';

	it('takes a folder outside the repo', () => {
		expect(assertBoardDir('/tmp/job/round-78', repo, () => true)).toBe('/tmp/job/round-78');
	});

	it('refuses a folder inside the repo, because a board is never in it', () => {
		expect(() => assertBoardDir(`${repo}/apps/ledger/design/boards`, repo, () => true)).toThrow(
			/outside the repo/
		);
		expect(() => assertBoardDir(repo, repo, () => true)).toThrow(/outside the repo/);
	});

	it('refuses a folder that is not there, and a missing name', () => {
		expect(() => assertBoardDir('/tmp/job/nowhere', repo, () => false)).toThrow(/no folder/);
		expect(() => assertBoardDir(undefined, repo, () => true)).toThrow(/name the board/);
	});
});

describe('drawingFiles', () => {
	it('takes the board’s own drawings and nothing the kit wrote', () => {
		expect(
			drawingFiles([
				'person-tally.tsx',
				'board.html',
				'frame.js',
				'frame.html',
				'.kit',
				'notes.md',
				'person-bar.tsx',
			])
		).toEqual(['person-bar.tsx', 'person-tally.tsx']);
	});
});

describe('frameEntry', () => {
	it('starts a frame on the board’s drawings, each named from the folder above', () => {
		const source = frameEntry('/repo/packages/workbench/src/lib/board-frame', [
			'person-bar.tsx',
			'person-tally.tsx',
		]);

		expect(source).toContain(`import './entry.css';`);
		expect(source).toContain(
			`import { startFrame } from '/repo/packages/workbench/src/lib/board-frame';`
		);
		expect(source).toContain(`import * as drawing0 from '../person-bar.tsx';`);
		expect(source).toContain(`import * as drawing1 from '../person-tally.tsx';`);
		expect(source).toContain('startFrame([drawing0, drawing1]);');
	});

	it('starts a frame with no drawings when the board has none', () => {
		expect(frameEntry('/repo/packages/workbench/src/lib/board-frame', [])).toContain(
			'startFrame([]);'
		);
	});
});

describe('pageEntry', () => {
	it('starts the board on its own document', () => {
		const source = pageEntry('/repo/packages/workbench/src/lib/board-page');

		expect(source).toContain(
			`import { startBoard } from '/repo/packages/workbench/src/lib/board-page';`
		);
		expect(source).toContain('export const { mount, theme } = startBoard(document);');
	});
});

describe('styleEntry', () => {
	it('imports the sheet every frame loads and scans the board beside it', () => {
		const source = styleEntry('/tmp/job/round-78/.kit', '/repo/apps/ledger/design/index.css');

		expect(source).toContain(`@import '../../../../repo/apps/ledger/design/index.css';`);
		expect(source).toContain(`@source '../*.{ts,tsx,html}';`);
	});
});

describe('sheetHrefs', () => {
	it('reads the faces off the tags the app’s config puts in the head', () => {
		const face =
			'https://fonts.googleapis.com/css2?family=Archivo:wght@400;500;600&family=Geist+Mono:wght@400;500&display=swap';

		expect(
			sheetHrefs([
				{ tag: 'link', attrs: { rel: 'preconnect', href: 'https://fonts.googleapis.com' } },
				{ tag: 'link', attrs: { href: face, rel: 'stylesheet' } },
				{ tag: 'meta', attrs: { name: 'theme-color', content: '#000' } },
			])
		).toEqual([face]);
	});
});

describe('frameHtml', () => {
	it('is a whole document that loads the faces, the sheet and the kit', () => {
		const html = frameHtml(['https://fonts.googleapis.com/css2?family=Archivo&display=swap']);

		expect(html.startsWith('<!doctype html>')).toBe(true);
		expect(html).toContain(
			'<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Archivo&amp;display=swap">'
		);
		expect(html).toContain('<link rel="stylesheet" href="frame.css">');
		expect(html).toContain('<div id="root"></div>');
		expect(html).toContain('<script src="frame.js"></script>');
	});
});
