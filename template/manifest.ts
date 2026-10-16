/**
 * What the cut takes from the source app, what it leaves there, and what is the template's own.
 *
 * - `exclude`: source paths never copied. The domain stays in the source app; so do the
 *   self-hosted production path, the skill-activation hook and the app's own design history.
 * - `owned`: template paths the cut copies only while they are missing, and never overwrites or
 *   deletes. They hold what the template needed changed by hand (a barrel cut to the neutral
 *   exports, the root scripts) or what only the template has (the example resource). A source
 *   change to one is reported at the next cut.
 * - `seed`: the same, never reported. The UI package and the design root are copied once and are
 *   the template's from then on (D5): nothing of the source's style flows into them again. The
 *   toolkit's packages, and the copies `toolkit sync` makes of their skills, are here too: this repo
 *   is their home (D1), and the source takes them by version.
 * - `keep`: what the cut never copies nor deletes: its own files, and the lockfile.
 * - `paths`, `text`: renames applied to every copied file, path first, then contents. A text rule
 *   with `only` applies to the template paths it names. The source takes the `@jswob/*` packages
 *   from the registry; the cut gives them back their `workspace:*` ranges.
 * - `words`: after the cut, every file this repo keeps (but the cut's own) is searched for them, the
 *   way `toolkit check` searches a package; each hit is listed.
 *
 * Globs are `path.matchesGlob`'s (`*`, `**`, `{a,b}`), against paths from the repo root.
 */

export type Manifest = {
	/** The source app's words, which no file here may hold; `allow` names where one is the template's own. */
	words: { list: string[]; allow: Record<string, string[]> };
	exclude: string[];
	owned: string[];
	seed: string[];
	keep: string[];
	paths: [from: string, to: string][];
	text: { from: RegExp; to: string; only?: string[] }[];
};

const API = 'apps/api/src';
const WEB = 'apps/web/src';
const UI = 'packages/ui/src';
const BENCH = 'apps/workbench';

/** The API modules that are the app's domain: everything but sign-in and wake. */
const API_MODULES = [
	'assignments',
	'availability-exclusions',
	'meetings',
	'participant-traits',
	'participants',
	'role-eligibility-windows',
	'role-type-eligibility',
	'role-types',
	'rules',
	'solve-jobs',
	'standing-statistics',
	'task-rules',
	'task-type-rules',
	'task-types',
	'trait-definitions',
];

/** The UI package's domain: the kit's ledger family and the domain pages (D5 keeps the rest). */
const UI_DOMAIN = [
	`${UI}/kit/{Processor,QuorumMeter,RuleBar,RuleList,RuleSentence,SeatMark,Sheet,SpacingTimeline,Status,TimeBar}*`,
	`${UI}/kit/{ledger,rule-picks,rule-words,sheet-*,solve-shape,spacing-timeline,rail-typing,split-where}*`,
	`${UI}/lib/day-range*`,
	`${UI}/views/{meeting-*,people*,person-*,role-own*,roles*,share-run*,solver-*,task-own*,tasks*,timeline-foot*,trait-*,traits*}`,
];

const manifest: Manifest = {
	words: {
		list: ['meeting', 'quorum', 'solver', 'participant', 'worship', 'trait', 'ms-'],
		// A fixture's traits are the fixtures library's own API, and the skills teach it; this
		// repo's own notes on the cut name the source
		allow: {
			meeting: ['TEMPLATE.md'],
			solver: ['TEMPLATE.md'],
			trait: [
				'packages/drizzle-fixtures/**',
				'.claude/skills/elysia-drizzle-api-best-practices/**',
				'packages/toolkit/claude/skills/elysia-drizzle-api-best-practices/**',
			],
		},
	},
	exclude: [
		// The source's patches of the packages are fixes on their way here (`toolkit upstream`)
		'patches/@jswob*',
		// The solver and its skill
		'services/**',
		'.claude/skills/cp-sat/**',
		'.github/scripts/extract-solver-url.sh',
		// The skill-activation hook (D7)
		'.claude/hooks/**',
		'.claude/skills/skill-rules.json',
		// The self-hosted production path (D8)
		'docker-compose.prod.yml',
		'.env.production.example',
		'.env.cloud.example',
		'scripts/import-cloud-db.sh',
		'docker/web/**',
		'docs/self-hosted-deployment.md',
		'apps/api/scripts/{migrate-prod,export-prod-data}.ts',
		// The app's history and plans
		'docs/{ROADMAP.md,quorum-migration/**}',
		// API: migrations (the template's baseline is generated anew), the domain modules and tables
		'apps/api/drizzle/**',
		`${API}/modules/{${API_MODULES.join(',')}}/**`,
		`${API}/db/schema/{assignments,meetings,participant*,role*,rule*,solve-jobs,standing-statistics,task*,trait-definitions}.ts`,
		`${API}/test/fixtures/*.fixture.ts`,
		`${API}/constants.ts`,
		// What only the app's own seed tables need
		`${API}/db/seed-tables.test.ts`,
		// Web: the domain's hooks, adapters, routes and helpers
		`${WEB}/hooks/api/**`,
		`${WEB}/quorum/adapters/{api-rule,candidates,ground,meeting-*,person-*,role-*,rule-*,seat-*,sheet-*,solve-*,solver-*,task-*,today,trait-*,use-ground,use-landing,use-meeting,use-people,use-person-*,use-roles,use-tasks,use-traits}*`,
		`${WEB}/routes/{meeting,people,roles,solver,tasks,traits}*`,
		`${WEB}/routes/-components/{ground-ledger,person-*,solver-*,use-*}*`,
		`${WEB}/lib/{datetime,rule-weight}.ts`,
		`${WEB}/quorum/declare-hold.test.tsx`,
		'apps/web/public/**',
		// UI: the domain kit and pages
		...UI_DOMAIN,
		// Workbench app: the app's design history, its places, their fixtures and its open rounds
		`${BENCH}/{HISTORY,PERSON-PAGE,ROLE-PAGE,QUORUM-SYSTEM}.md`,
		`${BENCH}/fixtures/{assignments,datetime,fixtures,ids,index,meeting*,meetings,participants,people*,person*,role-types,roles*,row-types,rules,sheet-ids*,solve-jobs,solver-*,task*,tasks,trait*,traits}.*`,
		`${BENCH}/views/{meeting-*,people-*,person-*,place-sheet*,roles-*,solver-*,tasks-*,timeline-*,traits-*}`,
		`${BENCH}/kit/{BoardShell*,sheet-shape.test.ts,spacing-timeline.test.ts}`,
		`${BENCH}/kit-docs/stress.tsx`,
		`${BENCH}/probe/**`,
		`${BENCH}/theses/**`,
	],
	owned: [
		// The root: what names the app, runs it and ships it. The API's image builds the packages as
		// workspaces here, in blocks setup strips
		'{README,TEMPLATE,CLAUDE}.md',
		'{package.json,.env.example,docker-compose.yml,.lintstagedrc.json,eslint.config.js,vitest.config.ts}',
		// Renovate's settings for this repo, which has the packages as workspaces; setup writes an app's
		'renovate.json',
		'.claude/settings.json',
		'.claude/skills/*/PROJECT.md',
		'{.github,deploy,docker,docs}/**',
		// The packages' release: changesets, the publish script beside the setup, and the Renovate
		// presets apps take them by
		'{.changeset,renovate,scripts}/**',
		// API: the barrels and lists that name the tables, and the example resource
		'apps/api/{package.json,README.md,CLAUDE.md,drizzle-snapshots.config.ts}',
		'apps/api/drizzle/**',
		`${API}/{routes.ts,db/schema/index.ts,db/schema/notes.ts,db/seed.ts,db/seed-tables.ts,db/seed-core.test.ts}`,
		`${API}/test/fixtures/{index,note.fixture}.ts`,
		`${API}/modules/notes/**`,
		// Web: the frame's wiring, the routes and hooks, and the example resource
		'apps/web/{index.html,README.md,CLAUDE.md}',
		'apps/web/{public,e2e/tests}/**',
		`${WEB}/routeTree.gen.ts`,
		`${WEB}/frame/{declare,pages,pages.test}.ts`,
		`${WEB}/routes/{__root.tsx,-components/frame.tsx,-components/ground.tsx,-components/use-notes-page*,notes*}`,
		`${WEB}/hooks/api/**`,
		// Web: the sign-in, the frame's channel and the routes every app keeps, whose comments and
		// tests the source writes in its own domain
		`${WEB}/{index.css,boot.test.ts}`,
		`${WEB}/providers/{AuthProvider,SignedInReads}*`,
		`${WEB}/frame/{declare-channel.tsx,declare.test.tsx,declare-frame.test.tsx,boundaries.test.ts}`,
		`${WEB}/frame/adapters/{sign-in-server.ts,use-read-failure*}`,
		`${WEB}/routes/{$,account,index}.tsx`,
		`${WEB}/routes/-components/hold-source.ts`,
	],
	seed: [
		'packages/ui/**',
		`${BENCH}/**`,
		// The toolkit's home (D1) and what `toolkit sync` copies out of it
		'packages/{config,drizzle-fixtures,drizzle-snapshots,drizzle-utils,llm-evals,toolkit,workbench}/**',
		'.claude/toolkit-lock.json',
		'.claude/{agents,commands,rules}/**',
		'.claude/skills/{design,elysia-drizzle-api-best-practices,llm-evals,plan-reader,react-frontend-best-practices,smart-commit,upstream}/**',
	],
	// The lockfile is never copied nor deleted: the source's takes the packages from the registry,
	// this one has them as workspaces, and `bun install` after the cut writes it
	keep: ['template/**', 'bun.lock'],
	paths: [['apps/web/src/quorum/', 'apps/web/src/frame/']],
	text: [
		// The source takes the packages by version; here they are workspaces
		{
			from: /("@jswob\/[\w.-]+"): "[^"]*"/g,
			to: '$1: "workspace:*"',
			only: ['**/package.json'],
		},
		{ from: /@meetings-scheduler\//g, to: '@app/' },
		{ from: /@\/quorum\//g, to: '@/frame/' },
		{ from: /quorum-appearance/g, to: 'app-appearance' },
		{ from: /meetings[-_]scheduler/g, to: 'app' },
		// The design system's token prefix: Quorum's `q-`, the seed's `ui-`
		{
			from: /(?<!\w)q-(?=[a-z])/g,
			to: 'ui-',
			only: ['{packages/ui,apps/web,apps/workbench}/**', '.claude/skills/*/PROJECT.md'],
		},
	],
};

export default manifest;
