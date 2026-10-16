import { PgDialect, integer, pgTable, text } from 'drizzle-orm/pg-core';
import { describe, expect, test } from 'vitest';
import { caseBy } from './case-by';

const items = pgTable('items', {
	id: text('id').primaryKey(),
	position: integer('position'),
});

const render = (pairs: Parameters<typeof caseBy>[2]) =>
	new PgDialect().sqlToQuery(caseBy(items.id, items.position, pairs));

describe('caseBy', () => {
	test('gives each key its own value, cast to the column type', () => {
		const { sql, params } = render([
			['a', 1],
			['b', 2],
		]);

		expect(sql).toBe(
			'(case when "items"."id" = $1 then $2::integer when "items"."id" = $3 then $4::integer end)'
		);
		expect(params).toEqual(['a', 1, 'b', 2]);
	});

	test('refuses an empty list, which would set every row to null', () => {
		expect(() => render([])).toThrow('caseBy needs at least one pair');
	});
});
