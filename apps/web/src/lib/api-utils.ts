/** Error with structured details from API responses. */
export class ApiError extends Error {
	status?: number;
	details?: Record<string, unknown>;

	constructor(message: string, status?: number, details?: Record<string, unknown>) {
		super(message);
		this.name = 'ApiError';
		this.status = status;
		this.details = details;
	}
}

export function extractErrorMessage(error: unknown): string {
	if (error instanceof Error) {
		return error.message;
	}

	if (typeof error === 'string') {
		return error;
	}

	if (error && typeof error === 'object') {
		if ('message' in error) return String(error.message);
		if ('error' in error && typeof error.error === 'string') {
			return (error as { error: string }).error;
		}
	}

	return 'An unknown error occurred';
}
