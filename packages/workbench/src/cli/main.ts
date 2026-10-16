#!/usr/bin/env bun
import process from 'node:process';

import { findConfig, loadConfig } from '../config';
import { census } from '../probe/census';
import { probe } from '../probe/run';
import { serve } from '../vite';
import { bool, flag, parseArgv, type Argv } from './args';
import { ban } from './bans';
import { board } from './board';
import { newCandidate, rmCandidate } from './candidates';
import { check } from './check';
import { conform } from './conform';
import { drive } from './drive';
import { frames } from './frames';
import { host, setHost } from './host';
import { promote } from './promote';
import { shots } from './shots';
import { skill } from './skill';
import { kit, tokens } from './specimens';
import { sweep } from './sweep';

/**
 * `bun workbench <verb>` — the mechanical half of a design round.
 *
 * Everything here is clerking that was being done by hand: building a frame address, pressing a
 * control and reading what moved, registering a candidate, appending a ban at the right anchor,
 * checking the design root. Three sessions of transcripts put ~80% of a round's tool calls in
 * `Bash`, and the single commonest shape was a `python3` heredoc doing text surgery on
 * `proto/index.ts` or `BANS.md` — 96 of them in one round.
 *
 * **What is deliberately not here**, because automating it would cost the thing the loop is for:
 * writing theses (one thread, by hand, with the bans in front of it), choosing a winner (the
 * user's, always), and summarising a ban (the exact wording *is* the check). This CLI moves paper,
 * it does not design.
 */
const HELP = `bun workbench <verb> [--config workbench.config.ts]

  serve   [--host]                               The bench, until it is stopped. --host opens it to
                                                 the network, for a phone.

  frames  [--space view] [--frames id:state,…] [--json]
          Every frame-able state with the address that opens it.

  drive   <id>:<state> [--width mobile|tablet|desktop]
          [--list] [--all] [--do "click:Name; press:Escape"] [--shot out.png] [--motion]
          [--theme light]
          Press the live frame and report what moved. --list names every target.

  sweep   <id>[:<state>] | --space views         Press every control at every width and print only
          [--widths mobile,desktop] [--every]    what went wrong: nothing moved, keyboard lost,
          [--all] [--theme light]                Escape did not close, text wider than its box.
          [--accept [--only <regex>]]            Hold the findings that are a design, once read.

  conform [<id>…] [--widths mobile,desktop]      Hold the frames against the thesis's numbers
          [--all] [--theme light]                (theses/<id>.thesis.ts). Prints what was not kept.

  shots   <folder> [--against <folder>]          One PNG of every frame at every width in both
          [--space views] [--frames id:state,…]  appearances, motion off, into a folder outside the
          [--widths mobile] [--theme light]      repo. --against takes them again and names every
          [--workers 4]                          file that differs from that earlier set.

  new     <id> --title "…" [--space proto]      Scaffold a candidate, its thesis file, and register it.
          [--from drawing.tsx]                   Start it from a board's approved drawing instead.
  rm      <id>                                   Delete a candidate and unregister it.
  promote <id> --to kit|views [--name Pascal]    Move it out of proto and report its callers.
  promote kit/<Name>|views/<name> --to app [--dry]
          Move a settled module into the app's settled root. Refuses on any workbench-only import.

  ban     find --for sheet[,density]             The exact wording of the sections that apply.
          add  --section 5 --round 50 --title "…"   Body on stdin.

  board   <folder>                               Build the board kit into a board's folder, outside the repo:
          the workbench's real frames and the board's own drawings, for a page to mount.

  check   [--skill] [--allow path,…]             Design-scoped lint, types, tests, census, the skill's
                                                 copy, blast radius. --skill lets a round write the skill.
  skill                                          Copy the design skill into the repo: toolkit sync for
                                                 this package alone. Never writes PROJECT.md.

  kit     [<id>] [--state s] [--parts]           A kit component's page, read back from the bench:
          [--width mobile] [--theme light]       its states, props, callers and measured parts.
  tokens                                         Every token with its values, contrast and users.

  probe   [--space views | --all]                The measured floor on every frame at every width:
          [--frames id:state,…] [--widths mobile] sideways scroll, overflow, hit-testing, visible
          [--theme light] [--full] [--quiet]     focus, contrast. Prints what the baseline does not
          [--accept]                             hold; --accept holds what was printed, once read.
  census                                         Every state the system has and every component that
                                                 draws it, read off the source.
`;

type Verb = (argv: Argv) => number | Promise<number>;

/** `serve` — the bench, until the process is stopped. It never returns a status of its own. */
async function serveVerb(argv: Argv): Promise<number> {
	await serve(host(), { host: bool(argv, 'host') });

	return new Promise<number>(() => undefined);
}

const VERBS: Record<string, Verb> = {
	serve: serveVerb,
	board,
	frames,
	kit,
	tokens,
	drive,
	sweep,
	conform,
	shots,
	new: newCandidate,
	rm: rmCandidate,
	promote,
	ban,
	check,
	probe,
	census,
	skill,
};

async function main(): Promise<void> {
	const argv = parseArgv(process.argv.slice(2));

	if (argv.verb === '' || argv.verb === 'help' || argv.flags['help'] !== undefined) {
		process.stdout.write(HELP);
		return;
	}

	const verb = VERBS[argv.verb];
	if (verb === undefined) {
		process.stderr.write(`unknown verb "${argv.verb}"\n\n${HELP}`);
		process.exitCode = 1;
		return;
	}

	try {
		setHost(await loadConfig(findConfig(process.cwd(), flag(argv, 'config'))));
		process.exitCode = await verb(argv);
	} catch (error) {
		process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
		process.exitCode = 1;
	}
}

await main();
