// `bun evals:selftest`: runs the whole harness against a fake OpenRouter server. No network, no
// cost. It proves the things unit tests cannot: that promptfoo's scheduler really waits and retries
// on what our provider hands it, that a dead key stops a run, that the spending limit holds, and
// that a run which stopped, failed or was killed is finished by running it again, with nothing
// paid twice. Run it before changing the pinned promptfoo version.
import { spawn } from 'node:child_process';
import { existsSync, mkdtempSync, readdirSync, readFileSync } from 'node:fs';
import { createServer, type IncomingMessage, type ServerResponse } from 'node:http';
import type { AddressInfo } from 'node:net';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { EXIT } from '../core/integrity.ts';
import type { Report } from '../core/report.ts';

/** Calls the fake server answered per word, including the failed ones. */
const calls = new Map<string, number>();
let keyChecks = 0;
/** While true, the word `stuck` fails every time: an outage that outlasts promptfoo's retries. */
let outage = false;
const SLOW_MS = 300;

function send(
	response: ServerResponse,
	status: number,
	body: unknown,
	headers: Record<string, string> = {}
) {
	response.writeHead(status, { 'content-type': 'application/json', ...headers });
	response.end(JSON.stringify(body));
}

const answer = (text: string) => ({
	model: 'fake/upper-1',
	provider: 'Fake',
	choices: [{ finish_reason: 'stop', message: { content: JSON.stringify({ text }) } }],
	usage: { prompt_tokens: 10, completion_tokens: 2, cost: 0.001, is_byok: true },
});

/** What each word makes the server do. `n` is the call number for that word, from 1. */
function reply(word: string, n: number, response: ServerResponse): void {
	if (word === 'dead')
		return send(response, 401, { error: { code: 401, message: 'No auth credentials found' } });
	if (word === 'rate3' && n === 3) {
		return send(
			response,
			429,
			{ error: { code: 429, message: 'Rate limit exceeded' } },
			{ 'retry-after': '1' }
		);
	}
	if (word === 'flaky' && n === 1)
		return send(response, 503, { error: { code: 503, message: 'overloaded' } });
	if (word === 'in200' && n === 1) {
		return send(response, 200, {
			error: { code: 429, message: 'Rate limit reached. Please try again in 50ms.' },
		});
	}
	if (word === 'nochoices' && n === 1) return send(response, 200, { provider: 'Fake', usage: {} });
	if (word === 'stuck' && outage)
		return send(response, 503, { error: { code: 503, message: 'overloaded' } });
	if (word.startsWith('slow')) {
		setTimeout(() => {
			send(response, 200, answer(word.toUpperCase()));
		}, SLOW_MS);
		return;
	}
	send(response, 200, answer(word === 'wrong' ? word : word.toUpperCase()));
}

function handle(request: IncomingMessage, response: ServerResponse): void {
	if (request.headers.authorization === 'Bearer bad-key') {
		return send(response, 401, { error: { code: 401, message: 'Invalid key' } });
	}
	if (request.url?.endsWith('/key')) {
		keyChecks++;
		return send(response, 200, {
			data: { label: 'selftest', limit_remaining: 5, is_free_tier: false },
		});
	}
	let raw = '';
	request.on('data', (chunk: Buffer) => (raw += chunk.toString()));
	request.on('end', () => {
		const body = JSON.parse(raw) as { messages: { content: string }[] };
		const word = body.messages.at(-1)?.content.replace('WORD: ', '') ?? '';
		const n = (calls.get(word) ?? 0) + 1;
		calls.set(word, n);
		reply(word, n, response);
	});
}

interface Outcome {
	code: number;
	output: string;
}

export interface SelftestEnv {
	/** the package directory */
	root: string;
	promptfoo: string;
}

export async function selftest(env: SelftestEnv): Promise<number> {
	const server = createServer(handle);
	await new Promise<void>(resolve => server.listen(0, '127.0.0.1', resolve));
	const base = `http://127.0.0.1:${(server.address() as AddressInfo).port}`;
	const scratch = mkdtempSync(join(tmpdir(), 'llm-evals-selftest-'));

	/** One CLI command in its own process. `killWhen`: kill it, and promptfoo under it, once this holds. */
	const cli = (
		args: string[],
		{ key = 'selftest-key', killWhen = undefined as (() => boolean) | undefined } = {}
	) =>
		new Promise<Outcome>(resolve => {
			const child = spawn('bun', [join(env.root, 'src', 'selftest', 'cli.ts'), ...args], {
				cwd: env.root,
				// Its own process group, so a kill reaches promptfoo too.
				detached: true,
				env: {
					...process.env,
					OPENROUTER_BASE_URL: base,
					OPENROUTER_API_KEY: key,
					SELFTEST_ROOT: scratch,
					SELFTEST_PROMPTFOO: env.promptfoo,
				},
			});
			let output = '';
			child.stdout.on('data', (chunk: Buffer) => (output += chunk.toString()));
			child.stderr.on('data', (chunk: Buffer) => (output += chunk.toString()));
			const watch = killWhen
				? setInterval(() => {
						if (!killWhen() || child.pid === undefined) return;
						clearInterval(watch);
						process.kill(-child.pid, 'SIGKILL');
					}, 25)
				: undefined;
			child.on('close', code => {
				clearInterval(watch);
				resolve({ code: code ?? 1, output });
			});
		});

	const latestRun = () => {
		const runs = readdirSync(join(scratch, '.runs')).sort();
		return join(scratch, '.runs', runs.at(-1) ?? '');
	};
	const reportOf = (dir: string) =>
		JSON.parse(readFileSync(join(dir, 'report.json'), 'utf8')) as Report;
	const spent = (dir: string) =>
		existsSync(join(dir, 'spend.log'))
			? readFileSync(join(dir, 'spend.log'), 'utf8')
					.split('\n')
					.reduce((sum, line) => sum + (Number(line) || 0), 0)
			: 0;

	const failures: string[] = [];
	const check = (what: string, ok: boolean, detail = '') => {
		process.stdout.write(`${ok ? '✓' : '✕'} ${what}${ok || !detail ? '' : `\n    ${detail}`}\n`);
		if (!ok) failures.push(what);
	};
	const callsFor = (word: string) => calls.get(word) ?? 0;
	const callsTo = (words: string[]) => words.reduce((sum, w) => sum + callsFor(w), 0);
	const allCalls = () => callsTo([...calls.keys()]);

	try {
		process.stdout.write(`Self-test against a fake model at ${base} (runs kept in ${scratch})\n\n`);

		// 1. A run with a rate limit, an outage, an error inside a 200, a broken reply and a wrong answer.
		let run = await cli(['run', 'selftest', '--group', 'main', '--max-cost', '1']);
		let dir = latestRun();
		const firstRun = dir.split('/').at(-1) ?? '';
		let report = reportOf(dir);
		const main = report.variants[0];
		check(
			'a run with technical failures still ends trusted (exit 0)',
			run.code === EXIT.ok,
			run.output.slice(-600)
		);
		check(
			'every row is scored: failures were retried, none counted as an answer',
			report.integrity.scored === 7 && report.integrity.retryable === 0,
			JSON.stringify(report.integrity)
		);
		check('one row per case, three samples inside it', main?.cases === 7);
		check(
			'only the wrong answer is wrong',
			main?.everWrong.cases === 1,
			JSON.stringify(main?.everWrong)
		);
		check(
			'the health check ran first: one key check and one canary call',
			keyChecks === 1 && callsFor('canary') === 1
		);
		check(
			'a 429 on the third sample costs exactly one extra call',
			callsFor('rate3') === 4,
			`calls: ${callsFor('rate3')}`
		);
		check('an outage (503) is retried', callsFor('flaky') === 4, `calls: ${callsFor('flaky')}`);
		check(
			'a rate limit inside an HTTP 200 is retried, not scored',
			callsFor('in200') === 4,
			`calls: ${callsFor('in200')}`
		);
		check(
			'a reply without choices is retried, not scored',
			callsFor('nochoices') === 4,
			`calls: ${callsFor('nochoices')}`
		);
		check(
			'an untroubled case takes exactly its three samples',
			callsFor('ok-one') === 3,
			`calls: ${callsFor('ok-one')}`
		);
		const record = JSON.parse(readFileSync(join(dir, 'record.json'), 'utf8')) as {
			results: { prompts: { metrics: { namedScores: Record<string, number> } }[] };
		};
		const header = record.results.prompts[0]?.metrics.namedScores ?? {};
		check(
			'the viewer header carries the margin on cases ever wrong',
			typeof header.ever_wrong_at_most_pct === 'number' && header.ever_wrong === 1,
			JSON.stringify(header)
		);

		// 2. The same run again is answered from the cache: no call, nothing spent.
		const cachedBefore = allCalls();
		run = await cli(['run', 'selftest', '--group', 'main', '--max-cost', '0']);
		check(
			'a repeated run is answered from the cache at a limit of $0, with no call at all',
			run.code === EXIT.ok && allCalls() === cachedBefore,
			run.output.slice(-400)
		);

		// 3. A dead key stops the run instead of failing every row.
		run = await cli(['run', 'selftest', '--group', 'dead', '--max-cost', '1']);
		check(
			'a 401 aborts the run (exit 4)',
			run.code === EXIT.aborted,
			`exit ${run.code}\n${run.output.slice(-400)}`
		);
		check(
			'…after one call, not one per sample',
			callsFor('dead') === 1,
			`calls: ${callsFor('dead')}`
		);

		// 4. A failed health check means no paid call at all.
		const canaryBefore = callsFor('canary');
		run = await cli(['run', 'selftest', '--group', 'budget', '--max-cost', '1'], {
			key: 'bad-key',
		});
		check(
			'a rejected key fails the health check (exit 4)',
			run.code === EXIT.aborted,
			`exit ${run.code}`
		);
		check('…with zero model calls', callsFor('b1') === 0 && callsFor('canary') === canaryBefore);

		// 5. The spending limit stops a run, and running it again pays for nothing twice.
		const budget = ['b1', 'b2', 'b3', 'b4', 'b5', 'b6'];
		run = await cli(['run', 'selftest', '--group', 'budget', '--max-cost', '0.008']);
		dir = latestRun();
		check(
			'the spending limit stops the run (exit 2)',
			run.code === EXIT.over_budget,
			`exit ${run.code}\n${run.output.slice(-400)}`
		);
		check('…without crossing it', spent(dir) <= 0.008 + 1e-9, `spent ${spent(dir)}`);
		run = await cli(['run', 'selftest', '--group', 'budget', '--max-cost', '1']);
		report = reportOf(latestRun());
		check(
			'the same run again with a higher limit finishes it (exit 0)',
			run.code === EXIT.ok && report.integrity.scored === 6,
			`exit ${run.code} ${JSON.stringify(report.integrity)}`
		);
		check(
			'…and every sample was paid for exactly once',
			callsTo(budget) === 18,
			`calls: ${callsTo(budget)}`
		);

		// 6. A run killed mid-way is finished by running it again; what it had answered is not paid twice.
		const slow = ['slow1', 'slow2', 'slow3', 'slow4', 'slow5', 'slow6'];
		run = await cli(['run', 'selftest', '--group', 'slow', '--max-cost', '1'], {
			killWhen: () => callsTo(slow) >= 7,
		});
		const beforeKill = callsTo(slow);
		check(
			'a run can be killed mid-way',
			run.code !== EXIT.ok && beforeKill < 18,
			`exit ${run.code}`
		);
		run = await cli(['run', 'selftest', '--group', 'slow', '--max-cost', '1']);
		report = reportOf(latestRun());
		check(
			'the same run again finishes it (exit 0)',
			run.code === EXIT.ok && report.integrity.scored === 6,
			`exit ${run.code} ${JSON.stringify(report.integrity)}`
		);
		// Only the call in flight at the kill can be paid twice: its answer never reached the cache.
		check(
			'…and the answers taken before the kill were not paid again',
			callsTo(slow) <= 18 + 1,
			`calls: ${callsTo(slow)} (${beforeKill} before the kill)`
		);

		// 7. An outage that outlasts promptfoo's retries: no numbers, and the same run again heals it.
		outage = true;
		run = await cli(['run', 'selftest', '--group', 'stuck', '--max-cost', '1']);
		check(
			'a row that never got an answer makes the run untrusted (exit 3)',
			run.code === EXIT.untrusted,
			`exit ${run.code}\n${run.output.slice(-400)}`
		);
		check(
			'…and no result is reported',
			run.output.includes('no result is reported') && !run.output.includes('cases ever wrong')
		);
		outage = false;
		run = await cli(['run', 'selftest', '--group', 'stuck', '--max-cost', '1']);
		check(
			'the same run again, once the outage is over, ends trusted (exit 0)',
			run.code === EXIT.ok,
			`exit ${run.code}\n${run.output.slice(-400)}`
		);
		check(
			'…and the case that had answered was not paid again',
			callsFor('ok-four') === 3,
			`calls: ${callsFor('ok-four')}`
		);

		// 8. No limit, no run: `run` says what it can cost and stops.
		let before = allCalls();
		run = await cli(['run', 'selftest', '--group', 'main']);
		check(
			'a run without --max-cost prints what it can cost and makes no call (exit 1)',
			run.code === EXIT.usage &&
				run.output.includes('21 model calls at most') &&
				allCalls() === before,
			`exit ${run.code}\n${run.output.slice(-400)}`
		);

		// 9. Held-out cases need a reason, and every run on them is written down.
		const ledger = join(scratch, 'key', 'heldout-ledger.jsonl');
		before = allCalls();
		run = await cli(['run', 'selftest', '--split', 'heldout', '--max-cost', '1']);
		check(
			'a held-out run without --reason is refused (exit 1) before any call',
			run.code === EXIT.usage && allCalls() === before && !existsSync(ledger),
			`exit ${run.code}`
		);
		run = await cli([
			'run',
			'selftest',
			'--split',
			'heldout',
			'--reason',
			'self-test',
			'--max-cost',
			'1',
		]);
		const logged = existsSync(ledger) ? readFileSync(ledger, 'utf8').trim().split('\n') : [];
		check(
			'with a reason it runs and is written in the ledger',
			run.code === EXIT.ok && logged.length === 1 && logged[0]?.includes('self-test') === true,
			`exit ${run.code}, ledger: ${logged.join(' | ')}`
		);

		// 10. `try` shows the answers to cases typed by hand; `report` reads an older run again.
		run = await cli(['try', 'selftest', '--word', 'hello']);
		check(
			'try prints the answer to a typed case',
			run.code === EXIT.ok && run.output.includes('HELLO'),
			`exit ${run.code}\n${run.output.slice(-400)}`
		);
		run = await cli(['report', firstRun]);
		check(
			'report reads an older run again',
			run.code === EXIT.ok &&
				run.output.includes(firstRun) &&
				run.output.includes('cases ever wrong'),
			`exit ${run.code}\n${run.output.slice(-400)}`
		);
	} finally {
		server.close();
	}

	process.stdout.write(
		failures.length
			? `\n${failures.length} check${failures.length === 1 ? '' : 's'} failed.\n`
			: '\nAll checks passed.\n'
	);
	return failures.length ? 1 : 0;
}
