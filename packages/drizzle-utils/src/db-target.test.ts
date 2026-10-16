import { describe, expect, test } from 'vitest';
import { createDbTargets } from './db-target';

const { dbTarget, selectedDbTarget } = createDbTargets({
	app: 'default_db',
	test: 'default_test_db',
	e2e: 'default_e2e_db',
});

const LOCAL_ENV = {
	DB_USER: 'postgres',
	DB_PASSWORD: 'password',
	DB_HOST: 'localhost',
	DB_NAME: 'app_db',
	DB_PORT: '6001',
	TEST_DB_NAME: 'app_test_db',
	TEST_DB_PORT: '6002',
	E2E_DB_NAME: 'app_e2e_db',
	E2E_DB_PORT: '6003',
};

describe('dbTarget', () => {
	test('builds each local database from its own name and port', () => {
		expect(dbTarget('app', LOCAL_ENV).url).toBe(
			'postgresql://postgres:password@localhost:6001/app_db'
		);
		expect(dbTarget('test', LOCAL_ENV).url).toBe(
			'postgresql://postgres:password@localhost:6002/app_test_db'
		);
		expect(dbTarget('e2e', LOCAL_ENV).url).toBe(
			'postgresql://postgres:password@localhost:6003/app_e2e_db'
		);
	});

	test('the app database is DATABASE_URL when it is set', () => {
		const env = {
			...LOCAL_ENV,
			DATABASE_URL: 'postgresql://u:p@db.example.com:5432/prod?sslmode=require',
		};
		expect(dbTarget('app', env)).toEqual({
			name: 'prod',
			port: '5432',
			url: 'postgresql://u:p@db.example.com:5432/prod?sslmode=require',
		});
	});

	test('the test and e2e databases never fall through to DATABASE_URL', () => {
		const env = { ...LOCAL_ENV, DATABASE_URL: 'postgresql://u:p@db.example.com:5432/prod' };
		expect(dbTarget('test', env).url).toBe(
			'postgresql://postgres:password@localhost:6002/app_test_db'
		);
		expect(dbTarget('e2e', env).url).toBe(
			'postgresql://postgres:password@localhost:6003/app_e2e_db'
		);
	});

	test('a database whose variables are unset takes its default name and port', () => {
		expect(dbTarget('app', {}).url).toBe(
			'postgresql://postgres:password@localhost:5432/default_db'
		);
		expect(dbTarget('test', {}).url).toBe(
			'postgresql://postgres:password@localhost:5433/default_test_db'
		);
		expect(dbTarget('e2e', {}).url).toBe(
			'postgresql://postgres:password@localhost:5434/default_e2e_db'
		);
	});

	test('a password with URL characters survives', () => {
		const url = dbTarget('test', { ...LOCAL_ENV, DB_PASSWORD: 'p@ss=w/rd' }).url;
		expect(decodeURIComponent(new URL(url).password)).toBe('p@ss=w/rd');
	});
});

describe('selectedDbTarget', () => {
	test('DB_TARGET picks the database, app when unset', () => {
		expect(selectedDbTarget(LOCAL_ENV).name).toBe('app_db');
		expect(selectedDbTarget({ ...LOCAL_ENV, DB_TARGET: 'test' }).name).toBe('app_test_db');
		expect(selectedDbTarget({ ...LOCAL_ENV, DB_TARGET: 'e2e' }).name).toBe('app_e2e_db');
	});

	test('an unknown DB_TARGET is refused rather than guessed', () => {
		expect(() => selectedDbTarget({ ...LOCAL_ENV, DB_TARGET: 'prod' })).toThrow(/DB_TARGET/);
	});
});
