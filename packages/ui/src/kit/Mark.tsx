/**
 * The app's glyph — a placeholder until the app draws its own: a filled square beside an open one.
 * Replace the drawing and keep the box, 34×16, which the rail and the sign-in are laid out around.
 */
/**
 * `rail` is the mark in the chrome, 34×16. `display` is the one place it is drawn large — the
 * centred screen sign-in stands on, at three times the rail's size, the only picture the app shows
 * whole.
 */
const SIZE = {
	rail: 'h-4 w-[34px]',
	display: 'h-12 w-[102px]',
} as const;

export function Mark({ size = 'rail' }: { size?: keyof typeof SIZE } = {}) {
	return (
		<svg viewBox="0 0 34 16" className={SIZE[size]} role="img" aria-label="Logo">
			<rect x={0} y={0} width={16} height={16} fill="var(--color-ui-ink)" />
			<rect x={18.5} y={0.5} width={15} height={15} fill="none" stroke="var(--color-ui-ink-3)" />
		</svg>
	);
}
