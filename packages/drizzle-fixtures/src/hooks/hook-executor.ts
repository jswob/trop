import type { AnyPgTable } from 'drizzle-orm/pg-core';
import type { TraitAfterMakeContext } from '../types';
import { HookExecutionError } from '../utils/errors';

/**
 * Execute a lifecycle hook with proper error handling.
 * No-op if hook is undefined.
 */
export async function executeHook<TCtx>(
	hookName: string,
	hook: ((ctx: TCtx) => void | Promise<void>) | undefined,
	context: TCtx
): Promise<void> {
	if (!hook) return;
	try {
		await hook(context);
	} catch (error: unknown) {
		const cause = error instanceof Error ? error : new Error(String(error));
		throw new HookExecutionError(hookName, cause);
	}
}

/**
 * Execute a trait afterMake hook and return its augmentation.
 * Returns undefined if hook is not defined.
 */
export async function executeTraitAfterMake<
	TModel,
	TParams,
	TAugmentation,
	TTable extends AnyPgTable,
>(
	traitName: string,
	afterMake:
		| ((
				ctx: TraitAfterMakeContext<TModel, TParams, TTable>
		  ) => TAugmentation | Promise<TAugmentation>)
		| undefined,
	context: TraitAfterMakeContext<TModel, TParams, TTable>
): Promise<TAugmentation | undefined> {
	if (!afterMake) return undefined;
	try {
		const result = await afterMake(context);
		if (result !== null && result !== undefined && typeof result !== 'object') {
			throw new Error(`Expected afterMake to return an object, got ${typeof result}`);
		}
		return result;
	} catch (error: unknown) {
		const cause = error instanceof Error ? error : new Error(String(error));
		throw new HookExecutionError(`trait.${traitName}.afterMake`, cause);
	}
}
