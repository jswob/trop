/**
 * The verb and its flags, parsed once.
 *
 * Deliberately small and with no opinion about which flags exist: every verb is reached from one
 * dispatch in `main.ts`, and a parser that knew the whole set would have to be edited by every
 * verb added. It knows `--flag value`, `--flag=value` and a bare `--flag`, and nothing else.
 *
 * A value beginning with a single `-` is still a value — `--scroll -400` is a scroll up, and a
 * parser that read the minus as the next flag would silently drop it.
 */
export type Argv = {
	/** The first bare token. `''` when the command line is only flags. */
	verb: string;
	/** Every bare token after the verb, in order. */
	positional: string[];
	flags: Record<string, string | true>;
};

export function parseArgv(argv: string[]): Argv {
	const flags: Record<string, string | true> = {};
	const positional: string[] = [];
	let verb = '';

	for (let index = 0; index < argv.length; index += 1) {
		const token = argv[index] ?? '';

		if (!token.startsWith('--')) {
			if (verb === '') verb = token;
			else positional.push(token);
			continue;
		}

		const body = token.slice(2);
		const equals = body.indexOf('=');
		if (equals !== -1) {
			flags[body.slice(0, equals)] = body.slice(equals + 1);
			continue;
		}

		const next = argv[index + 1];
		if (next !== undefined && !next.startsWith('--')) {
			flags[body] = next;
			index += 1;
		} else {
			flags[body] = true;
		}
	}

	return { verb, positional, flags };
}

/** The flag's value, or `undefined` when it is absent or was given bare. */
export function flag(argv: Argv, name: string): string | undefined {
	const value = argv.flags[name];
	return typeof value === 'string' ? value : undefined;
}

/** Whether the flag was given at all, with or without a value. */
export function bool(argv: Argv, name: string): boolean {
	return argv.flags[name] !== undefined;
}

/** A comma-separated flag, split and trimmed. Absent reads as empty. */
export function list(argv: Argv, name: string): string[] {
	return (flag(argv, name) ?? '')
		.split(',')
		.map(value => value.trim())
		.filter(Boolean);
}

/** A numeric flag, or `fallback` when absent. Throws on a value that is not a number. */
export function num(argv: Argv, name: string, fallback: number): number {
	const value = flag(argv, name);
	if (value === undefined) return fallback;

	const parsed = Number(value);
	if (!Number.isFinite(parsed)) throw new Error(`--${name} takes a number, got "${value}"`);
	return parsed;
}
