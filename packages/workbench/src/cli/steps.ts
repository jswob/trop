/**
 * `--do` scripts: what a hand does to a frame, as data.
 *
 * The whole reason this is a parsed script rather than a sequence of calls is that a round has to
 * be able to **repeat** what it did. A round deleted thirteen listed states on the evidence that a
 * press reaches them, and every one of those presses was a bespoke driver script written, run once
 * and deleted — so the evidence was a number in a report with nothing behind it. A step script is
 * the same evidence with the experiment still attached.
 */
export type Step =
	| { kind: 'click'; target: string }
	| { kind: 'press'; key: string }
	| { kind: 'fill'; target: string; value: string }
	| { kind: 'tab'; times: number }
	| { kind: 'scroll'; by: number }
	| { kind: 'wait'; ms: number };

const STEP = /^(click|press|fill|tab|scroll|wait)(?::([\s\S]*))?$/;

function required(value: string, step: string, what: string): string {
	const trimmed = value.trim();
	if (trimmed === '') throw new Error(`bad step "${step}" — ${what}`);
	return trimmed;
}

function count(value: string, step: string, fallback: number): number {
	if (value.trim() === '') return fallback;

	const parsed = Number(value.trim());
	if (!Number.isFinite(parsed))
		throw new Error(`bad step "${step}" — "${value.trim()}" is not a number`);
	return parsed;
}

/**
 * Steps are separated by `;`, which means a `;` cannot appear inside a name or a filled value.
 * That is a real limit and it is the right trade: the alternative is a quoting scheme, and a
 * quoting scheme in a shell argument is two escaping layers deep before anything has been pressed.
 */
export function parseSteps(source: string): Step[] {
	return source
		.split(';')
		.map(part => part.trim())
		.filter(Boolean)
		.map((part): Step => {
			const match = STEP.exec(part);
			if (!match) {
				throw new Error(
					`bad step "${part}" — expected click:<name>, press:<key>, fill:<name>=<value>, tab[:n], scroll[:px] or wait[:ms]`
				);
			}

			const kind = match[1] ?? '';
			const rest = match[2] ?? '';

			switch (kind) {
				case 'click':
					return { kind: 'click', target: required(rest, part, 'click takes the target name') };
				case 'press':
					return { kind: 'press', key: required(rest, part, 'press takes a key, e.g. Escape') };
				case 'fill': {
					const equals = rest.indexOf('=');
					if (equals === -1) throw new Error(`bad step "${part}" — fill takes name=value`);
					return {
						kind: 'fill',
						target: required(rest.slice(0, equals), part, 'fill takes the field name'),
						value: rest.slice(equals + 1),
					};
				}
				case 'tab':
					return { kind: 'tab', times: count(rest, part, 1) };
				case 'scroll':
					return { kind: 'scroll', by: count(rest, part, 400) };
				default:
					return { kind: 'wait', ms: count(rest, part, 200) };
			}
		});
}

/** How one step reads back in the report, so the log says what was done rather than what was asked. */
export function describeStep(step: Step): string {
	switch (step.kind) {
		case 'click':
			return `click "${step.target}"`;
		case 'press':
			return `press ${step.key}`;
		case 'fill':
			return `fill "${step.target}" with "${step.value}"`;
		case 'tab':
			return `tab ×${step.times}`;
		case 'scroll':
			return `scroll ${step.by > 0 ? '+' : ''}${step.by}px`;
		case 'wait':
			return `wait ${step.ms}ms`;
	}
}

function normalise(value: string): string {
	return value.replace(/\s+/g, ' ').trim().toLowerCase();
}

function escapeRegExp(value: string): string {
	return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/**
 * Which of `names` a pattern means — **by tiers, and never by "the first one that matched"**.
 *
 * This is a round's failure written as code. `/^Configure Spring order/` matched a control in the
 * side column as well as the one on the document, the driver took the first, and the run reported *"2630
 * chars -> 2630 chars"* — a press that did nothing, indistinguishable from a state that does not
 * exist. Reading the file to find the real label cost the round-trip; taking the first match cost
 * the wrong conclusion, which is worse.
 *
 * So: exact wins outright, then a glob if the pattern has a `*`, then a prefix, then a substring.
 * The first tier with any match is the answer and later tiers are not consulted — a name that is
 * exactly right is never outvoted by two names that merely contain it. **Ambiguity inside a tier is
 * returned as ambiguity**, for the caller to refuse with the candidates named.
 */
export function matchTargets(names: string[], pattern: string): number[] {
	const wanted = normalise(pattern);
	const all = names.map(normalise);

	const exact = all.flatMap((name, index) => (name === wanted ? [index] : []));
	if (exact.length > 0) return exact;

	/*
	 * A pattern with a `*` is a glob and nothing else. Falling through to the prefix and substring
	 * tiers would run them against a string that still has the `*` in it, so `Mar 8*` would match
	 * nothing anywhere and report the frame as having no such control — a wrong answer dressed as
	 * a missing one. A glob that matches nothing is a glob that matches nothing.
	 */
	if (wanted.includes('*')) {
		const expression = new RegExp(`^${wanted.split('*').map(escapeRegExp).join('.*')}$`);
		return all.flatMap((name, index) => (expression.test(name) ? [index] : []));
	}

	const prefixed = all.flatMap((name, index) => (name.startsWith(wanted) ? [index] : []));
	if (prefixed.length > 0) return prefixed;

	return all.flatMap((name, index) => (name.includes(wanted) ? [index] : []));
}
