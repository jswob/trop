/**
 * Check whether a thrown error carries a PostgreSQL error code. postgres.js puts the code on the
 * error; Drizzle wraps that error and keeps it as `cause`, so both are read.
 * @param error - The caught error value to inspect
 * @param code - The SQLSTATE code, e.g. `23505`
 */
export function isPgError(error: unknown, code: string): boolean {
	if (typeof error !== 'object' || error === null) return false;
	if ((error as { code?: unknown }).code === code) return true;
	const cause = (error as { cause?: unknown }).cause;
	return typeof cause === 'object' && cause !== null && (cause as { code?: unknown }).code === code;
}

/**
 * A unique constraint violation (`23505`).
 * @param error - The caught error value to inspect
 */
export function isUniqueViolation(error: unknown): boolean {
	return isPgError(error, '23505');
}

/**
 * A check constraint violation (`23514`).
 * @param error - The caught error value to inspect
 */
export function isCheckViolation(error: unknown): boolean {
	return isPgError(error, '23514');
}

/**
 * An exclusion constraint violation (`23P01`), raised by an `EXCLUDE` constraint such as one
 * forbidding overlapping ranges.
 * @param error - The caught error value to inspect
 */
export function isExclusionViolation(error: unknown): boolean {
	return isPgError(error, '23P01');
}
