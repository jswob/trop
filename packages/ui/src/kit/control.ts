/**
 * The height every control stands at: 40px where the pointer may be a finger, 32px
 * from `lg` up where it is a mouse.
 *
 * Deliberately not `--spacing-ui-row`. A row is a line of data the eye scans and it
 * stays at 28px; a control is a target a hand has to hit. Tying them together means
 * every attempt to tighten the ledger shrinks the buttons with it.
 */
export const CONTROL_H = 'h-ui-touch lg:h-ui-control';

/**
 * A line of the document at the control floor: the label and whatever it comes to.
 *
 * The shape a view would otherwise write out by hand. A control that **opens or closes** one of
 * these lines takes `edge` — `kit/Button`'s `LabelEdge` — so its ground stops on the measure
 * rather than 4px past it; without it the row reports `scrollWidth − clientWidth = 4` and the
 * word at the end of the line stops 4px short of the band's aside above it.
 */
export const CONTROL_ROW = `flex ${CONTROL_H} items-center gap-3`;

/**
 * The size a control's text computes at **while a hand is typing into it**.
 *
 * 16px below `lg`, the system's own 13px body above it, and the split is not a taste decision. A
 * phone browser zooms the whole page the moment focus lands on a form control whose computed
 * font-size is under 16px — it is Safari's fix for a desktop form on a small screen, it cannot be
 * turned off from CSS, and the page it leaves behind is magnified and panned sideways, so a layout
 * that was exactly as wide as the glass is suddenly neither. Every size in this system is under
 * 16px, so every control tripped it.
 *
 * The other way out is `maximum-scale=1` in the viewport meta, and it is not taken: it is a
 * standing instruction that the page may not be magnified, which is a WCAG 1.4.4 failure written to
 * work around a browser default.
 *
 * It goes on the **typing surface**, never on the box around it. `CONTROL_H` already says a control
 * is a different size for a finger than for a mouse; this says the same thing about the one line of
 * text a finger is putting characters into, and nothing else on the screen moves.
 */
export const CONTROL_TYPE = 'text-[16px] lg:text-ui-body';
