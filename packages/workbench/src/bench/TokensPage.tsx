import { Page } from './Page';
import { useChrome } from './chrome';
import { HOST } from './host';
import { HOST_SOURCES } from './host-sources';
import { contrastOf, over, readColor, type Rgba } from './measure';
import { readTokens, usedBy, type Token, type Tried } from './tokens';

const TOKENS = readTokens(HOST.themeCss);
const USED = usedBy(TOKENS, HOST_SOURCES);
const APPEARANCES = ['dark', 'light'] as const;
type Appearance = (typeof APPEARANCES)[number];

const WHITE: Rgba = [255, 255, 255, 1];
let probe: HTMLElement | null = null;

/** A colour as the browser reads it, whatever form it was typed in. Nothing, for what is not one. */
function resolve(value: string): Rgba | null {
	if (probe === null) {
		probe = document.createElement('i');
		probe.hidden = true;
		document.body.append(probe);
	}

	probe.style.color = '';
	probe.style.color = value;
	return probe.style.color === '' ? null : readColor(getComputedStyle(probe).color);
}

/** The property a token of each kind is read as, to ask the browser whether a typed value is one. */
const READ_AS: Partial<Record<Token['kind'], string>> = {
	colour: 'color',
	type: 'font-size',
	space: 'width',
};

/** Whether the browser can read a typed value as what the token is. A page that cannot ask says yes. */
function readable(token: Token, value: string): boolean {
	const property = READ_AS[token.kind];
	if (property === undefined || typeof CSS === 'undefined') return true;

	return CSS.supports(property, value);
}

/** The key a tried value is kept under: a colour has one an appearance, a length has one. */
const triedKey = (token: Token, appearance: Appearance): string =>
	token.light === undefined ? token.name : `${token.name}.${appearance}`;

/**
 * The system's tokens, read from the stylesheet that declares them.
 *
 * Nobody writes this page. A colour is a chip with its dark and its light value, its contrast on
 * each ground, and how many files spend it. A value can be **tried**: typed into its box it is
 * drawn here, reaches every frame on every page of the bench, and rides in the address. It is
 * never written to the stylesheet. Trying is looking; changing a token is still a round.
 */
export function TokensPage() {
	const { tried, setTried } = useChrome();

	const valueOf = (token: Token, appearance: Appearance): string =>
		tried[triedKey(token, appearance)] ??
		(appearance === 'light' ? (token.light ?? token.dark) : token.dark);

	const ground = TOKENS.find(token => token.name === HOST.ground);
	const groundOf = (appearance: Appearance): Rgba =>
		over((ground && resolve(valueOf(ground, appearance))) ?? WHITE, WHITE);

	const contrast = (token: Token, appearance: Appearance): string => {
		if (token.name === HOST.ground) return 'the ground';

		const colour = resolve(valueOf(token, appearance));
		return colour === null ? 'unread' : contrastOf(colour, groundOf(appearance)).toFixed(2);
	};

	const commit = (token: Token, appearance: Appearance, typed: string) => {
		const key = triedKey(token, appearance);
		const declared = appearance === 'light' ? (token.light ?? token.dark) : token.dark;
		const value = typed.trim();
		const rest: Tried = Object.fromEntries(
			Object.entries(tried).filter(([other]) => other !== key)
		);

		setTried(value === '' || value === declared ? rest : { ...rest, [key]: value });
	};

	const box = (token: Token, appearance: Appearance) => {
		const value = valueOf(token, appearance);
		const isTried = triedKey(token, appearance) in tried;
		const isBad = isTried && !readable(token, value);
		const named = `${token.key}, ${token.light === undefined ? 'value' : appearance}`;

		return (
			<input
				// The box keeps what is being typed; it is read back when the hand leaves it.
				key={value}
				className={['wb-val', isTried && 'wb-val-tried', isBad && 'wb-val-bad']
					.filter(Boolean)
					.join(' ')}
				aria-label={isTried ? `${named}, tried` : named}
				aria-invalid={isBad || undefined}
				title={isBad ? 'Not a value the browser can read' : undefined}
				defaultValue={value}
				spellCheck={false}
				onBlur={event => {
					if (event.target.value.trim() !== value) commit(token, appearance, event.target.value);
				}}
				onKeyDown={event => {
					if (event.key === 'Enter') event.currentTarget.blur();
				}}
			/>
		);
	};

	const files = (token: Token): string => {
		const count = USED[token.name]?.length ?? 0;
		return `${String(count)} ${count === 1 ? 'file' : 'files'}`;
	};

	const of = (kind: Token['kind']): Token[] => TOKENS.filter(token => token.kind === kind);
	const face = TOKENS.find(token => token.name === HOST.face)?.dark;

	return (
		<Page
			name="Tokens"
			description="Read from the stylesheet that declares them. A value typed here is tried, never written."
			readout={`${String(TOKENS.length)} tokens`}
		>
			<div className="wb-stage">
				<h2 className="wb-h">Colour, dark and light</h2>
				<div className="wb-chips">
					{of('colour').map(token => (
						<div key={token.name} className="wb-chip">
							<div className="wb-sws" aria-hidden="true">
								{APPEARANCES.map(appearance => (
									<i
										key={appearance}
										style={{ background: ground ? valueOf(ground, appearance) : undefined }}
									>
										<i style={{ background: valueOf(token, appearance) }} />
									</i>
								))}
							</div>
							<div className="wb-meta">
								<b>{token.key}</b>
								<span>{files(token)}</span>
							</div>
							{box(token, 'dark')}
							{box(token, 'light')}
							<div className="wb-meta">
								<span>on its ground</span>
								<span>
									{token.name === HOST.ground
										? 'the ground'
										: `${contrast(token, 'dark')}, ${contrast(token, 'light')}`}
								</span>
							</div>
						</div>
					))}
				</div>

				<div className="wb-two">
					<section aria-label="Type">
						<h2 className="wb-h">Type</h2>
						{of('type').map(token => (
							<div key={token.name} className="wb-type">
								<b>{token.key}</b>
								{box(token, 'dark')}
								<span>{files(token)}</span>
								<span style={{ fontSize: valueOf(token, 'dark'), fontFamily: face }}>
									Modify order, Mar 8
								</span>
							</div>
						))}
						{of('face').map(token => (
							<div key={token.name} className="wb-type">
								<b>{token.key}</b>
								<span className="wb-wide">{token.dark}</span>
								<span>{files(token)}</span>
							</div>
						))}
					</section>
					<section aria-label="Space">
						<h2 className="wb-h">Space</h2>
						{of('space').map(token => (
							<div key={token.name} className="wb-space">
								<b>{token.key}</b>
								{box(token, 'dark')}
								<span>{files(token)}</span>
								<i style={{ width: valueOf(token, 'dark') }} />
							</div>
						))}
					</section>
				</div>
			</div>
		</Page>
	);
}
