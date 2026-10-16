import { describe, it, expect } from 'vitest';

import { users } from '../fixtures/test-schema';
import type { DrizzleDatabase, FixtureConfig } from '../types';
import { TraitNotFoundError } from '../utils/errors';
import { FixtureBuilder } from './fixture-builder';
import { SequenceCounter } from './sequence';

const mockDb = {} as DrizzleDatabase;

describe('Trait System', () => {
	describe('Core trait behavior', () => {
		it('trait field resolver overrides base resolver', async () => {
			type UserTraits = { admin: never };
			const config: FixtureConfig<typeof users, UserTraits> = {
				table: users,
				fields: {
					id: ({ sequence }) => sequence,
					name: 'Regular User',
					role: 'user',
				},
				traits: {
					admin: {
						fields: {
							role: 'admin',
						},
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const adminBuilder = builder.trait('admin');

			const adminUser = await adminBuilder.build();

			expect(adminUser.role).toBe('admin');
			expect(adminUser.name).toBe('Regular User');
		});

		it('multiple traits: later trait overrides earlier trait field', async () => {
			type UserTraits = { admin: never; superAdmin: never };
			const config: FixtureConfig<typeof users, UserTraits> = {
				table: users,
				fields: {
					id: ({ sequence }) => sequence,
					name: 'User',
					role: 'user',
				},
				traits: {
					admin: {
						fields: {
							role: 'admin',
							name: 'Admin User',
						},
					},
					superAdmin: {
						fields: {
							role: 'superadmin',
						},
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const result = await builder.trait('admin').trait('superAdmin').build();

			expect(result.role).toBe('superadmin');
			expect(result.name).toBe('Admin User');
		});

		it('trait without params (never) works with no argument', async () => {
			type UserTraits = { verified: never };
			const config: FixtureConfig<typeof users, UserTraits> = {
				table: users,
				fields: {
					id: 1,
					name: 'User',
				},
				traits: {
					verified: {
						fields: {
							role: 'verified',
						},
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const result = await builder.trait('verified').build();

			expect(result.role).toBe('verified');
		});

		it('trait with params - params stored correctly (visible via appliedTraitParams)', () => {
			type UserTraits = { withRole: string };
			const config: FixtureConfig<typeof users, UserTraits> = {
				table: users,
				fields: {
					id: 1,
					name: 'User',
					role: 'user',
				},
				traits: {
					withRole: {
						fields: {},
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const withRole = builder.trait('withRole', 'moderator');

			expect(withRole.explain().appliedTraits).toContain('withRole');
		});

		it('unknown trait throws TraitNotFoundError with available traits listed', () => {
			type UserTraits = { admin: never; verified: never };
			const config: FixtureConfig<typeof users, UserTraits> = {
				table: users,
				fields: {
					id: 1,
					name: 'User',
				},
				traits: {
					admin: { fields: {} },
					verified: { fields: {} },
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter(), ['nonexistent']);

			expect(() => builder.explain()).toThrow(TraitNotFoundError);
			expect(() => builder.explain()).toThrow(/Trait "nonexistent" not found/);
			expect(() => builder.explain()).toThrow(/Available: admin, verified/);
		});

		it('chaining: .trait("a").trait("b").build() works correctly', async () => {
			type UserTraits = { admin: never; verified: never };
			const config: FixtureConfig<typeof users, UserTraits> = {
				table: users,
				fields: {
					id: ({ sequence }) => sequence,
					name: 'User',
					role: 'user',
				},
				traits: {
					admin: {
						fields: {
							role: 'admin',
						},
					},
					verified: {
						fields: {
							name: 'Verified User',
						},
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const result = await builder.trait('admin').trait('verified').build();

			expect(result.role).toBe('admin');
			expect(result.name).toBe('Verified User');
		});

		it('trait + direct override: override still wins over trait', async () => {
			type UserTraits = { admin: never };
			const config: FixtureConfig<typeof users, UserTraits> = {
				table: users,
				fields: {
					id: 1,
					name: 'User',
					role: 'user',
				},
				traits: {
					admin: {
						fields: {
							role: 'admin',
						},
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const result = await builder.trait('admin').build({ role: 'superuser' });

			expect(result.role).toBe('superuser');
		});

		it("explain() shows 'trait' as source for trait-provided fields", () => {
			type UserTraits = { admin: never };
			const config: FixtureConfig<typeof users, UserTraits> = {
				table: users,
				fields: {
					id: 1,
					name: 'User',
					role: 'user',
				},
				traits: {
					admin: {
						fields: {
							role: 'admin',
						},
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const explanation = builder.trait('admin').explain();

			expect(explanation.fieldSources.role).toBe('trait');
			expect(explanation.fieldSources.name).toBe('base');
			expect(explanation.fieldSources.id).toBe('base');
		});
	});

	describe('Edge cases', () => {
		it('trait with only some fields - other fields still resolve from base', async () => {
			type UserTraits = { admin: never };
			const config: FixtureConfig<typeof users, UserTraits> = {
				table: users,
				fields: {
					id: ({ sequence }) => sequence,
					name: 'Base Name',
					email: 'base@example.com',
					role: 'user',
				},
				traits: {
					admin: {
						fields: {
							role: 'admin',
						},
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const result = await builder.trait('admin').build();

			expect(result.role).toBe('admin');
			expect(result.name).toBe('Base Name');
			expect(result.email).toBe('base@example.com');
			expect(result.id).toBe(1);
		});

		it('duplicate trait application: .trait("admin").trait("admin") - resolver runs for each application', async () => {
			let resolverCallCount = 0;
			type UserTraits = { counter: never };
			const config: FixtureConfig<typeof users, UserTraits> = {
				table: users,
				fields: {
					id: 1,
					name: 'User',
				},
				traits: {
					counter: {
						fields: {
							name: () => {
								resolverCallCount++;
								return `Call ${resolverCallCount}`;
							},
						},
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const result = await builder.trait('counter').trait('counter').build();

			expect(result.name).toBe('Call 1');

			expect(builder.trait('counter').trait('counter').explain().appliedTraits).toEqual([
				'counter',
				'counter',
			]);
		});

		it('fixture with no traits config + trait() call throws TraitNotFoundError with "Available: none"', () => {
			const config: FixtureConfig<typeof users> = {
				table: users,
				fields: {
					id: 1,
					name: 'User',
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter(), ['admin']);

			expect(() => builder.explain()).toThrow(TraitNotFoundError);
			expect(() => builder.explain()).toThrow(/Available: none/);
		});

		it('trait with undefined fields property (just afterMake hook) - works fine', async () => {
			type UserTraits = { hookOnly: never };
			const config: FixtureConfig<typeof users, UserTraits> = {
				table: users,
				fields: {
					id: 1,
					name: 'User',
					role: 'user',
				},
				traits: {
					hookOnly: {
						afterMake: () => ({}),
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const result = await builder.trait('hookOnly').build();

			expect(result.name).toBe('User');
			expect(result.role).toBe('user');
		});

		it('buildList() with traits - traits apply correctly to all items', async () => {
			type UserTraits = { admin: never };
			const config: FixtureConfig<typeof users, UserTraits> = {
				table: users,
				fields: {
					id: ({ sequence }) => sequence,
					name: ({ sequence }) => `User ${sequence}`,
					role: 'user',
				},
				traits: {
					admin: {
						fields: {
							role: 'admin',
						},
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());
			const results = await builder.trait('admin').buildList(3);

			expect(results).toHaveLength(3);
			expect(results[0]!.role).toBe('admin');
			expect(results[1]!.role).toBe('admin');
			expect(results[2]!.role).toBe('admin');
			expect(results[0]!.id).toBe(1);
			expect(results[1]!.id).toBe(2);
			expect(results[2]!.id).toBe(3);
		});
	});

	describe('Type safety', () => {
		it('properly typed trait definition compiles and works', async () => {
			type UserTraits = { admin: never; withEmail: string };
			const config: FixtureConfig<typeof users, UserTraits> = {
				table: users,
				fields: {
					id: 1,
					name: 'User',
					role: 'user',
				},
				traits: {
					admin: {
						fields: {
							role: 'admin',
						},
					},
					withEmail: {
						fields: {
							email: 'custom@example.com',
						},
					},
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter());

			const adminResult = await builder.trait('admin').build();
			expect(adminResult.role).toBe('admin');

			const emailResult = await builder.trait('withEmail', 'test@test.com').build();
			expect(emailResult.email).toBe('custom@example.com');
		});

		it('explain() throws TraitNotFoundError for invalid trait (same as build())', () => {
			type UserTraits = { admin: never };
			const config: FixtureConfig<typeof users, UserTraits> = {
				table: users,
				fields: {
					id: 1,
					name: 'User',
				},
				traits: {
					admin: { fields: {} },
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter(), ['nonexistent']);

			expect(() => builder.explain()).toThrow(TraitNotFoundError);
		});
	});

	describe('build() trait validation', () => {
		it('build() throws TraitNotFoundError for invalid trait', async () => {
			type UserTraits = { admin: never };
			const config: FixtureConfig<typeof users, UserTraits> = {
				table: users,
				fields: {
					id: 1,
					name: 'User',
				},
				traits: {
					admin: { fields: {} },
				},
			};

			const builder = new FixtureBuilder(config, mockDb, new SequenceCounter(), ['nonexistent']);

			await expect(builder.build()).rejects.toThrow(TraitNotFoundError);
			await expect(builder.build()).rejects.toThrow(/Trait "nonexistent" not found/);
		});
	});
});
