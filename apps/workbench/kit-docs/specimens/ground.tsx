import type { ReactNode } from 'react';

/** The room round a component, for the outlines of its parts. */
const MARGIN = 24;

/**
 * What a component stands on in its frame: the app's own ground, face and ink, and a margin for the
 * outlines of its parts. It hugs what it holds, so the frame around it can be cut to the component.
 *
 * `data-specimen` is how the bench finds it, to measure the box and the parts inside it. It can hold
 * the keyboard and is never a stop in it, with no ring: it is where `frames` lands it, not a control.
 *
 * A measure it is given is never wider than the glass: a 512px column in a 390px frame would run
 * off the right edge, where the frame is cut and no check looks.
 */
export function Ground({ width, children }: { width?: number; children: ReactNode }) {
	return (
		<div
			data-specimen
			tabIndex={-1}
			className="bg-ui-bg font-ui-sans text-ui-ink"
			style={{ alignSelf: 'flex-start', width: 'fit-content', padding: MARGIN, outline: 'none' }}
		>
			{width === undefined ? (
				children
			) : (
				<div style={{ width, maxWidth: `calc(100vw - ${String(2 * MARGIN)}px)` }}>{children}</div>
			)}
		</div>
	);
}

/**
 * What a component that needs a whole screen stands on: the frame itself, positioned, so an overlay
 * laid over `inset-0` lands on the glass. Nothing is cut to it; the bench fits the frame to its stage.
 */
export function Screen({ children }: { children: ReactNode }) {
	return (
		<div
			data-specimen
			tabIndex={-1}
			className="relative flex min-h-0 flex-1 flex-col bg-ui-bg font-ui-sans text-ui-ink"
			style={{ outline: 'none' }}
		>
			{children}
		</div>
	);
}
