import { describe, expect, test } from 'vitest';
import { databaseSsl } from './ssl';

const HOSTED = 'postgresql://u:p@db.example.com/app?sslmode=require';
const LOCAL = 'postgresql://u:p@localhost:5432/app';

describe('databaseSsl', () => {
	test('follows the URL when DATABASE_SSL is unset', () => {
		expect(databaseSsl(HOSTED, {})).toBe('require');
		expect(databaseSsl(LOCAL, {})).toBe(false);
	});

	test('DATABASE_SSL wins over the URL either way', () => {
		expect(databaseSsl(LOCAL, { DATABASE_SSL: 'true' })).toBe('require');
		expect(databaseSsl(HOSTED, { DATABASE_SSL: 'false' })).toBe(false);
	});
});
