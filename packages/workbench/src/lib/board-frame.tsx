import { createRoot } from 'react-dom/client';

import { FrameDocument } from '../components/FrameDocument';
import type { Appearance } from './appearance';

import { parseFrameSpec } from './board-spec';
import { writePosition } from './frame-position';
import { stageEntryById } from './registry';
import type { Subject } from './subject';

function isSubject(value: unknown): value is Subject {
	if (typeof value !== 'object' || value === null) return false;

	const candidate = value as { id?: unknown; variants?: unknown };
	return typeof candidate.id === 'string' && Array.isArray(candidate.variants);
}

/**
 * Every `Subject` a board's own files export.
 *
 * A board's drawing file has the shape of a prototype — it exports a `Subject` — so the drawing a
 * board approves moves into `proto/` as the file it already is. The export's name is not asked
 * for: `workbench new` writes `export const personTally`, and a board may as well.
 */
export function boardSubjects(modules: Record<string, unknown>[]): Subject[] {
	return modules.flatMap(module => Object.values(module).filter(isSubject));
}

/**
 * What a frame says when its name finds nothing.
 *
 * It sets no colour and no face of its own: the frame's document carries the app's, and a message
 * drawn in one system's tokens would have to be redrawn for the next.
 */
function Missing({ children }: { children: string }) {
	return (
		<div style={{ padding: 20 }}>
			<p>{children}</p>
			<p style={{ marginTop: 8, opacity: 0.7 }}>
				Settled frames are listed by bun workbench frames. A board&rsquo;s own are the .tsx files
				beside its page.
			</p>
		</div>
	);
}

/**
 * Draws the frame a board asked for, in the document `workbench board` writes as `frame.html`.
 *
 * The frame is named by the `name` its `<iframe>` was given (`lib/board-spec`). A board's own
 * drawings are looked up before the workbench's registry, and everything else — the ground, the
 * appearance, the motion — is `FrameDocument`, the same component the workbench's frame route
 * draws.
 *
 * A name that says where the frame has walked to (`where`) is written onto the frame's own address
 * before anything draws, so a board pins a place mid-walk the way the stage's `Open` does.
 *
 * It answers its parent twice over a message: once to say what it measured, so a board can tell a
 * frame that drew from one that never loaded, and whenever the board turns its appearance over.
 */
export function startFrame(modules: Record<string, unknown>[]): void {
	const element = document.getElementById('root');
	if (element === null) throw new Error('the frame document has no #root');

	const own = boardSubjects(modules);
	const find = (id: string): Subject | undefined =>
		own.find(subject => subject.id === id) ?? stageEntryById(id);

	const spec = parseFrameSpec(window.name);
	if (spec?.where !== undefined) writePosition(spec.where);

	let theme: Appearance = spec?.theme ?? 'dark';
	const root = createRoot(element);

	const draw = (): void => {
		root.render(
			spec === null ? (
				<Missing>{`This frame was given no name to draw: “${window.name}”.`}</Missing>
			) : (
				<FrameDocument
					{...spec}
					theme={theme}
					find={find}
					missing={(what, name, of) => (
						<Missing>
							{what === 'entry'
								? `No frame called ${name}.`
								: `${of ?? spec.id} has no state called ${name}.`}
						</Missing>
					)}
				/>
			)
		);
	};

	window.addEventListener('message', event => {
		const data: unknown = event.data;
		if (typeof data !== 'object' || data === null) return;

		const next = (data as { boardTheme?: unknown }).boardTheme;
		if (next !== 'dark' && next !== 'light') return;

		theme = next;
		draw();
	});

	draw();
	window.parent.postMessage(
		{ boardFrame: window.name, width: window.innerWidth, height: window.innerHeight },
		'*'
	);
}
