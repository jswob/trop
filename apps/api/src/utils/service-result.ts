import { status } from 'elysia';

export type ErrorCode = 'NOT_FOUND' | 'VALIDATION' | 'CONFLICT' | 'INTERNAL';

export interface ServiceSuccess<T> {
	ok: true;
	data: T;
}

export interface ServiceError {
	code: ErrorCode;
	message: string;
	details?: Record<string, unknown>;
}

export interface ServiceFailure {
	ok: false;
	error: ServiceError;
}

export type ServiceResult<T> = ServiceSuccess<T> | ServiceFailure;

export function ok<T>(data: T): ServiceSuccess<T> {
	return { ok: true, data };
}

export function fail(
	code: ErrorCode,
	message: string,
	details?: Record<string, unknown>
): ServiceFailure {
	return { ok: false, error: { code, message, ...(details !== undefined && { details }) } };
}

/**
 * Turn a ServiceError into the route's response: its status, with the message and any details.
 *
 * The statuses are written out as literals rather than looked up. Eden Treaty keeps a failure on
 * the client's `error` channel, and `data` the success payload, only for a literal status; a
 * `number` puts the error body into `data`'s type.
 * @param error - The service error to answer with
 */
export function mapError(error: ServiceError) {
	const body = {
		error: error.message,
		...(error.details !== undefined && { details: error.details }),
	};

	switch (error.code) {
		case 'NOT_FOUND':
			return status(404, body);
		case 'VALIDATION':
			return status(422, body);
		case 'CONFLICT':
			return status(409, body);
		case 'INTERNAL':
			return status(500, body);
	}
}
