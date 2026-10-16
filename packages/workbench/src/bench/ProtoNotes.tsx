/**
 * What a candidate says past its one line: its bet in full, and what it would hand over.
 *
 * A view has no such panel, because a settled place needs no arguing for. A candidate is a bet
 * still being made, and the type it proposes is the handoff, so both are read off its own file.
 */
export function ProtoNotes({ bet, shape }: { bet: string; shape: string | null }) {
	return (
		<div className="wb-two">
			<section aria-label="The bet">
				<h2 className="wb-h">The bet</h2>
				<p className="wb-prose">{bet}</p>
			</section>
			<section aria-label="What it would hand over">
				<h2 className="wb-h">What it would hand over</h2>
				{shape === null ? (
					<p className="wb-prose">It declares no shape yet.</p>
				) : (
					<pre className="wb-code">{shape}</pre>
				)}
			</section>
		</div>
	);
}
