import { describe, expect, it } from 'vitest';

describe('drizzle-fixtures', () => {
	it('should have a working test environment', () => {
		expect(1 + 1).toBe(2);
	});

	it('should have environment variables loaded', () => {
		expect(process.env.DB_HOST).toBeDefined();
	});
});
