import { IconButton } from '@app/ui/kit/Button';
import { X } from 'lucide-react';
import { useEffect, useLayoutEffect, useRef, type ReactNode } from 'react';

import { FOCUS } from '@app/ui/kit/focus';

/**
 * A panel over the page. The one exception to "one ground": the backdrop
 * darkens the page behind rather than lifting the panel in front, so the panel is
 * still the same ground with the same edge every other surface has. No shadow, no
 * lightness delta, no radius.
 *
 * The close is a real `IconButton`, at the 32/40 control floor: a smaller glyph would sit under
 * the floor in the one surface a finger reaches for first.
 *
 * **It closes, and it holds the keyboard while it is open.** `onClose` is what `Escape` and the
 * close do: the caller's own way of keeping things as they are. Without `onClose` it cannot be
 * closed, which is how it stands while its errand runs.
 *
 * It takes the keyboard when it opens, on the panel itself so no answer is chosen for the reader,
 * and hands it back to whatever raised it when it goes. It does not pull the keyboard back from
 * something that took it on the way out, and a raiser that has left the page is the frame's to
 * answer for.
 */
export function Modal({
	title,
	children,
	footer,
	onClose,
}: {
	title: string;
	children: ReactNode;
	footer: ReactNode;
	/** Closing without answering: `Escape` and the close. Absent, neither does anything. */
	onClose?: () => void;
}) {
	const panel = useRef<HTMLDivElement>(null);

	useLayoutEffect(() => {
		const raiser = document.activeElement instanceof HTMLElement ? document.activeElement : null;
		const dialog = panel.current;
		dialog?.focus({ preventScroll: true });

		return () => {
			const within = dialog?.contains(document.activeElement) === true;
			if (within && raiser?.isConnected === true) raiser.focus({ preventScroll: true });
		};
	}, []);

	useEffect(() => {
		if (onClose === undefined) return;

		const onKey = (event: KeyboardEvent) => {
			if (event.key !== 'Escape' || event.defaultPrevented) return;
			event.preventDefault();
			onClose();
		};

		window.addEventListener('keydown', onKey);
		return () => {
			window.removeEventListener('keydown', onKey);
		};
	}, [onClose]);

	return (
		<>
			<div aria-hidden className="absolute inset-0 bg-ui-scrim" />
			<div className="absolute inset-0 flex items-center justify-center">
				<div
					ref={panel}
					role="dialog"
					aria-label={title}
					tabIndex={-1}
					className={`w-[420px] border border-ui-edge bg-ui-bg text-ui-ink ${FOCUS}`}
				>
					<div className="flex h-11 items-center justify-between border-b border-ui-rule pr-1 pl-4">
						<span className="text-ui-lead">{title}</span>
						<IconButton label="Close" onClick={onClose}>
							<X aria-hidden className="h-4 w-4" />
						</IconButton>
					</div>
					<div className="flex flex-col gap-3 px-4 py-4">{children}</div>
					<div className="flex items-center justify-end gap-2 border-t border-ui-rule px-4 py-3">
						{footer}
					</div>
				</div>
			</div>
		</>
	);
}
