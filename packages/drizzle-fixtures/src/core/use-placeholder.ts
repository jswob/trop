import type { UseHelper } from '../types';

/**
 * Creates a placeholder UseHelper that throws an informative error.
 *
 * The real `use()` helper requires the fixture registry and builder components.
 * This placeholder ensures users get a clear error message if they try to use
 * `use()` before it's available.
 *
 * @returns A UseHelper function that throws when called
 */
export function createPlaceholderUseHelper(): UseHelper {
	return (() => {
		throw new Error(
			'[drizzle-fixtures] use() is not available yet. ' +
				'For now, use direct values or async resolvers that do not depend on other fixtures.'
		);
	}) as UseHelper;
}
