/**
 * The three databases a checkout talks to, each as one full connection URL. Every script, config
 * and test that opens a database asks here, so one aimed at the test or e2e database can never
 * fall through to `DATABASE_URL`: a seed empties every table of whatever it is pointed at.
 */

export type DbTargetName = 'app' | 'test' | 'e2e';

export interface DbTarget {
	/** Database name. */
	name: string;
	port: string;
	/** Full connection URL. */
	url: string;
}

export type Env = Record<string, string | undefined>;

export interface DbTargets {
	/**
	 * One database's connection. `app` is `DATABASE_URL` when it is set (production), otherwise
	 * the local compose database; `test` and `e2e` are always local and never read `DATABASE_URL`.
	 * @param target - Which database
	 * @param env - Environment to read, `process.env` by default
	 */
	dbTarget: (target: DbTargetName, env?: Env) => DbTarget;
	/**
	 * The database this process is meant for, chosen by `DB_TARGET` (`app` when unset). Tests and
	 * the e2e server set it rather than overriding `DB_NAME` and `DB_PORT`, which a `DATABASE_URL`
	 * would silently win over.
	 * @param env - Environment to read, `process.env` by default
	 */
	selectedDbTarget: (env?: Env) => DbTarget;
}

/** The variables each local database is read from, and the compose file's port for it. */
const LOCAL: Record<DbTargetName, { name: string; port: string; defaultPort: string }> = {
	app: { name: 'DB_NAME', port: 'DB_PORT', defaultPort: '5432' },
	test: { name: 'TEST_DB_NAME', port: 'TEST_DB_PORT', defaultPort: '5433' },
	e2e: { name: 'E2E_DB_NAME', port: 'E2E_DB_PORT', defaultPort: '5434' },
};

function isTargetName(value: string): value is DbTargetName {
	return value in LOCAL;
}

/**
 * The app's database targets. Host, user and password come from `DB_HOST`, `DB_USER` and
 * `DB_PASSWORD`; each database's name and port from its own pair of variables.
 * @param defaultNames - Each database's name when its variable is unset
 */
export function createDbTargets(defaultNames: Record<DbTargetName, string>): DbTargets {
	function dbTarget(target: DbTargetName, env: Env = process.env): DbTarget {
		if (target === 'app' && env.DATABASE_URL) {
			const parsed = new URL(env.DATABASE_URL);
			return { name: parsed.pathname.slice(1), port: parsed.port || '5432', url: env.DATABASE_URL };
		}

		const local = LOCAL[target];
		const name = env[local.name] || defaultNames[target];
		const port = env[local.port] || local.defaultPort;
		const user = encodeURIComponent(env.DB_USER || 'postgres');
		const password = encodeURIComponent(env.DB_PASSWORD || 'password');
		const host = env.DB_HOST || 'localhost';
		return { name, port, url: `postgresql://${user}:${password}@${host}:${port}/${name}` };
	}

	function selectedDbTarget(env: Env = process.env): DbTarget {
		const selected = env.DB_TARGET || 'app';
		if (!isTargetName(selected)) {
			throw new Error(`DB_TARGET must be app, test or e2e, got "${selected}"`);
		}
		return dbTarget(selected, env);
	}

	return { dbTarget, selectedDbTarget };
}
