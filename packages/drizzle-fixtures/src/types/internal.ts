/**
 * Internal types for fixture composition.
 * Separated to avoid circular dependencies with main types file.
 */
import type { SequenceCounter } from '../core/sequence';

/**
 * Internal options passed between fixtures during composition.
 * Not part of public API - used by createUseHelper.
 */
export interface FixtureInternalOptions {
	sequence?: SequenceCounter;
	resolutionStack?: string[];
}
