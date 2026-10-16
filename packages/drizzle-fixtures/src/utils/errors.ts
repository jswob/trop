/**
 * Base error for all drizzle-fixtures errors
 */
export class FixtureError extends Error {
	constructor(message: string) {
		super(`[drizzle-fixtures] ${message}`);
		this.name = 'FixtureError';
	}
}

/**
 * Thrown when a trait name doesn't exist
 */
export class TraitNotFoundError extends FixtureError {
	constructor(traitName: string, availableTraits: string[]) {
		super(`Trait "${traitName}" not found. Available: ${availableTraits.join(', ') || 'none'}`);
		this.name = 'TraitNotFoundError';
	}
}

/**
 * Thrown when a field resolver fails
 */
export class FieldResolverError extends FixtureError {
	constructor(fieldName: string, originalError: Error) {
		super(`Resolver for field "${fieldName}" failed: ${originalError.message}`);
		this.name = 'FieldResolverError';
		this.cause = originalError;
	}
}

/**
 * Thrown when circular fixture dependencies are detected
 */
export class CircularDependencyError extends FixtureError {
	constructor(chain: string[]) {
		super(`Circular dependency detected: ${chain.join(' -> ')}`);
		this.name = 'CircularDependencyError';
	}
}

/**
 * Thrown when database operations fail
 */
export class DatabaseOperationError extends FixtureError {
	constructor(
		tableName: string,
		operation: 'insert' | 'delete' | 'update' | 'select',
		originalError: Error
	) {
		super(`Database ${operation} on "${tableName}" failed: ${originalError.message}`);
		this.name = 'DatabaseOperationError';
		this.cause = originalError;
	}
}

/**
 * Thrown when fixture configuration is invalid
 */
export class ValidationError extends FixtureError {
	constructor(field: string, reason: string) {
		super(`Invalid configuration for "${field}": ${reason}`);
		this.name = 'ValidationError';
	}
}

/**
 * Thrown when a lifecycle hook fails
 */
export class HookExecutionError extends FixtureError {
	public readonly hookName: string;

	constructor(hookName: string, cause: Error) {
		super(`Hook "${hookName}" failed: ${cause.message}`);
		this.name = 'HookExecutionError';
		this.hookName = hookName;
		this.cause = cause;
	}
}
