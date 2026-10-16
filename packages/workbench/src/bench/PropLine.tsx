import type { Specimen } from './specimen';

/**
 * The props of the one state showing, as switches.
 *
 * A second line under the bar, there only while one state stands alone. A prop that takes a short
 * list is a switch, a boolean is off and on, a string is typed. What is turned goes into the
 * address, so a played-with state is a link, and `Reset` puts it back as the specimen declares it.
 * A state the specimen draws whole takes nothing from the address, so it is given no switches.
 */
export function PropLine({
	spec,
	drawn = false,
	declared,
	turned,
	onTurn,
	onReset,
}: {
	spec: Specimen;
	/** The state is drawn whole by the specimen: there is nothing here to turn. */
	drawn?: boolean;
	/** What the state declares for each prop, by the value's name. */
	declared: Readonly<Record<string, string>>;
	turned: Readonly<Record<string, string>>;
	onTurn: (name: string, value: string) => void;
	onReset: () => void;
}) {
	const playable = drawn ? [] : spec.props.filter(prop => prop.kind !== 'none');

	return (
		<div className="wb-bar wb-propline" role="group" aria-label="Props of this state">
			<span className="wb-lb">Props</span>
			{playable.length === 0 && (
				<span className="wb-status">
					{drawn
						? 'This state is drawn whole, so its props are not turned from here. Pick another state.'
						: 'Nothing here takes a short list of values. Pick another state.'}
				</span>
			)}
			<div className="wb-ops">
				{playable.map(prop => {
					const now = turned[prop.name] ?? declared[prop.name] ?? '';

					return (
						<div key={prop.name} className="wb-op" role="group" aria-label={prop.name}>
							<span className="wb-lb" aria-hidden="true">
								{prop.name}
							</span>
							{prop.kind === 'text' ? (
								<input
									// The box keeps what is being typed; it is read back when the hand leaves it.
									key={now}
									className="wb-val"
									aria-label={prop.name}
									defaultValue={now}
									spellCheck={false}
									onBlur={event => {
										if (event.target.value !== now) onTurn(prop.name, event.target.value);
									}}
									onKeyDown={event => {
										if (event.key === 'Enter') event.currentTarget.blur();
									}}
								/>
							) : (
								<div className="wb-sw">
									{prop.names.map(name => (
										<button
											key={name}
											type="button"
											aria-pressed={now === name}
											onClick={() => onTurn(prop.name, name)}
										>
											{name}
										</button>
									))}
								</div>
							)}
						</div>
					);
				})}
			</div>
			{Object.keys(turned).length > 0 && (
				<button type="button" className="wb-reset" onClick={onReset}>
					Reset to declared
				</button>
			)}
		</div>
	);
}
