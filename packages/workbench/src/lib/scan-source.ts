const OPENERS = '{[(';
const CLOSERS = '}])';

/**
 * Returns the index just past a string literal or comment starting at `index`,
 * or -1 when nothing of that kind starts there. Template literals are treated as
 * plain strings; `${}` interpolation is not supported.
 */
export function skipAtomic(source: string, index: number): number {
	const char = source[index];

	if (char === '"' || char === "'" || char === '`') {
		let cursor = index + 1;
		while (cursor < source.length) {
			if (source[cursor] === '\\') {
				cursor += 2;
				continue;
			}
			if (source[cursor] === char) return cursor + 1;
			cursor += 1;
		}
		return source.length;
	}

	if (char === '/' && source[index + 1] === '/') {
		const end = source.indexOf('\n', index);
		return end === -1 ? source.length : end + 1;
	}

	if (char === '/' && source[index + 1] === '*') {
		const end = source.indexOf('*/', index + 2);
		return end === -1 ? source.length : end + 2;
	}

	return -1;
}

/**
 * Returns the index just past the bracket matching the one at `open`, ignoring
 * brackets that appear inside string literals or comments. Returns -1 if the
 * bracket is never closed.
 */
export function matchBracket(source: string, open: number): number {
	const opener = source[open];
	if (opener === undefined || !OPENERS.includes(opener)) return -1;

	let depth = 0;
	let cursor = open;

	while (cursor < source.length) {
		const skipped = skipAtomic(source, cursor);
		if (skipped !== -1) {
			cursor = skipped;
			continue;
		}

		const char = source[cursor] ?? '';
		if (OPENERS.includes(char)) depth += 1;
		else if (CLOSERS.includes(char)) {
			depth -= 1;
			if (depth === 0) return cursor + 1;
		}
		cursor += 1;
	}

	return -1;
}

/**
 * Splits an object literal — passed with its surrounding braces — into its
 * top-level `key: value` pairs. Keys are returned unquoted and values verbatim.
 */
export function objectEntries(objectText: string): [string, string][] {
	const body = objectText.slice(1, -1);
	const entries: [string, string][] = [];
	let cursor = 0;

	while (cursor < body.length) {
		const skipped = skipAtomic(body, cursor);
		if (skipped !== -1 && !isQuote(body[cursor])) {
			cursor = skipped;
			continue;
		}

		const char = body[cursor] ?? '';
		if (/\s/.test(char) || char === ',') {
			cursor += 1;
			continue;
		}

		const key = readKey(body, cursor);
		if (!key) break;

		let valueStart = key.end;
		while (valueStart < body.length && /\s/.test(body[valueStart] ?? '')) valueStart += 1;

		const valueEnd = findValueEnd(body, valueStart);
		entries.push([key.name, body.slice(valueStart, valueEnd).trim()]);
		cursor = valueEnd;
	}

	return entries;
}

function isQuote(char: string | undefined): boolean {
	return char === '"' || char === "'" || char === '`';
}

/** Reads a (possibly quoted) object key plus its `:`, returning the key and the index after the colon. */
function readKey(body: string, start: number): { name: string; end: number } | null {
	let cursor = start;
	let name: string;

	if (isQuote(body[cursor])) {
		const end = skipAtomic(body, cursor);
		name = body.slice(cursor + 1, end - 1);
		cursor = end;
	} else {
		const match = /^[A-Za-z0-9_$]+/.exec(body.slice(cursor));
		if (!match) return null;
		name = match[0];
		cursor += name.length;
	}

	while (cursor < body.length && /\s/.test(body[cursor] ?? '')) cursor += 1;
	if (body[cursor] !== ':') return null;

	return { name, end: cursor + 1 };
}

/** Returns the index of the comma that ends a top-level value, or the end of the body. */
function findValueEnd(body: string, start: number): number {
	let depth = 0;
	let cursor = start;

	while (cursor < body.length) {
		const skipped = skipAtomic(body, cursor);
		if (skipped !== -1) {
			cursor = skipped;
			continue;
		}

		const char = body[cursor] ?? '';
		if (OPENERS.includes(char)) depth += 1;
		else if (CLOSERS.includes(char)) depth -= 1;
		else if (char === ',' && depth === 0) return cursor;

		cursor += 1;
	}

	return body.length;
}
