"""Item 10 of the M2 run (issue #167; MILESTONE-M1.md row 8): G12's 63 over-specified vectors restated to their asks.

G12's review and fix probes (`fixes/g12/`, chunk C9 of M1) are a source group's text with `explain` asks added over
every balance, projection and `low` it holds (`pb_g9_sign_sign`: 476 asks, 472 of them `explain`). They were stored
as whole runs, with no `asks` column, so each held up to 2 811 lines of output and a change to how one ledger prints
rewrote tens of thousands of lines. The 63 are the `fixes/g12` vectors whose whole run printed 397 lines or more
(the next prints 316; listed in RESTATED below, the order is refcore's whole-run size, largest first).

Only 34 of the 63 texts differ (20 models: a text less its asks); the rest are copies under other ids. Each vector
keeps its id, text and group, and its `asks` column names the asks it is about, chosen mechanically from refcore's
whole run of its text, the vectors taken in RESTATED's order, each fact keyed by its model (the text less its asks):
- an ask that is not an explanation (`explain`, `inputs-of`, `readers-of`): kept the first time its model meets it,
  by name and expression (the source text's own questions);
- an explanation: kept when it shows its model something no ask before it showed: a new sort of head (a value, a
  hole, a refusal) or a new kind of line for its form. The form is the word, the function and the kind of its first
  argument (`explain bal` of an `accounts` place); a line's kind is its depth, its words from the explanation's
  vocabulary, the codes it prints and whether it holds a hole, with the names, days and figures left out;
- a vector that would keep nothing, a copy, keeps its first ask, so it still puts a question to its text.

So each kind of line each model's explanations print stays once per form, and the repeats on other days and in the
copies go. Every expected byte is still refcore's: m1/vectors.py stores refcore's run of the restated asks, and an ask
alone prints what it printed in its run (the ask rule, TESTING.md, known-faults.md).

    python3 -I -B engine/tests/m1/restate_g12.py            -> writes the asks column of the 63 in m1/vectors.tsv
    python3 -I -B engine/tests/m1/restate_g12.py --check    -> exit 1 if a row's asks aren't what the rule picks
    python3 -I -B engine/tests/m1/restate_g12.py --show     -> per vector: asks before and kept, lines kept

Then run m1/vectors.py, which re-stores the 63 from refcore. Standard library only; it runs refcore (run.py's
run_one, its cache and the watchdog) on each text whole, once.
"""
import csv
import importlib.util
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
TESTS = os.path.dirname(HERE)
sys.path.insert(0, TESTS)
import run as R  # noqa: E402

_spec = importlib.util.spec_from_file_location('kernel_vectors', os.path.join(TESTS, 'kernel', 'vectors.py'))
KV = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(KV)

LIST = os.path.join(HERE, 'vectors.tsv')

RESTATED = """
prr_bal_g8_proj_proj r_pb_bal-sign_sign prr_bal_g9_sign_sign pr_bal_g9_sign_sign pb_g9_sign_sign
r_pb_projunc-proj_proj prr_projunc_g8_proj_proj r_pb_proj-proj_proj prr_proj_g8_proj_proj pp_lowproj_g9_sign_sign
pp4_lowproj_g9_sign_sign pb_g5_agenda_agenda pb2_g5_agenda_agenda prr_proj_g9_sign_sign pr_proj_g9_sign_sign
pp_proj_g9_sign_sign pp4_proj_g9_sign_sign prr_projunc_g9_sign_sign r_pb_low-proj_proj prr_low_g8_proj_proj
pp_lowproj_g5_agenda_agenda prr_low_g9_sign_sign pp_low_g9_sign_sign pp_proj_g5_agenda_agenda pb_g10_observe_observe
pp_lowproj_g10_ledger_ledger prr_projunc_g10_ledger_ledger pb_g12_members_members pp_lowproj_g10_observe_observe
pp4_lowproj_g10_observe_observe pr_proj_g10_ledger_ledger prr_proj_g10_ledger_ledger pp_proj_g10_ledger_ledger
r_pb_bal-ledger_ledger prr_bal_g10_ledger_ledger pr_bal_g10_ledger_ledger pb_g10_ledger_ledger pb2_g10_ledger_ledger
pp_proj_g10_observe_observe pp4_proj_g10_observe_observe pp_low_g5_agenda_agenda pp_low_g10_observe_observe
prr_low_g10_ledger_ledger pp_low_g10_ledger_ledger r_agent_ord_order prr_bal_g5_order_order r_pb_bal-order_order
prr_proj_g5_order_order r_pb_proj-order_order prr_projunc_g5_order_order r_pb_projunc-order_order
prr_low_g5_order_order r_pb_low-order_order pb_g7_flows_touched_flows_touched pm8_g12_members_members
pm4_g12_members_members pm2_g12_members_members pm8_g5_agenda_agenda pm4_g5_agenda_agenda pm2_g5_agenda_agenda
pm8_g9_sign_sign pm4_g9_sign_sign pm2_g9_sign_sign
""".split()

EXPLAINS = ('explain', 'inputs-of', 'readers-of')
# The explanation's vocabulary (E §10, L §5.18): the words a line's shape keeps; any other word is a name.
WORDS = frozenset("""
balance of at the end start after before from opening reading stated anchor anchored pad padded no movement movements
cut law date unfulfilled left out refused past planned plan observed modelled adjustment adjustments settled
settlement decision decided projection low value hole holes its lifespan version source held assumed pinned booked
open uncertain when false true fulfils same gap windows window reads removed by more steps sum min max day instant
status prerun postrun run until rule rules band row rows given won lost read input inputs def cell cells key keys
readers nothing missing counted listed walk walked not yet price priced unit units leg legs amount change
""".split())
NUMBERISH = re.compile(r'-?\d[\d ]*(?:\.\d+)?|\d{4}-\d{2}-\d{2}')
QUOTED = re.compile(r'"[^"]*"')
CODE = re.compile(r'\b[A-Z]+(?:-[A-Z]+)+\b')
WORD = re.compile(r'[a-z][a-z-]*')


def rows():
    with open(LIST, encoding='utf-8', newline='') as f:
        r = csv.DictReader(f, delimiter='\t', quoting=csv.QUOTE_NONE)
        return r.fieldnames, list(r)


def texts(c):
    """A vector's texts, in the order its command line names its files."""
    out = []
    for f in c['files'].split():
        with open(R.ROOT + f, encoding='utf-8') as fh:
            out.append(fh.read())
    return out


def text_asks(c):
    """The asks of a vector's texts, each (name, expression), in text order."""
    return [a for t in texts(c) for a in re.findall(r'^ask (\w+): (.*)$', t, re.M)]


def model_of(c):
    """The model a vector's asks are put to: its texts without their ask lines and the lines indented under them."""
    keep, in_ask = [], False
    for t in texts(c):
        for line in t.split('\n'):
            if line.startswith('ask '):
                in_ask = True
                continue
            if in_ask and line.startswith(' '):
                continue
            in_ask = False
            keep.append(line)
    return '\n'.join(keep)


def place_kinds(model):
    """Each place's declared kind: `place konto: accounts …` gives konto → accounts."""
    return dict(re.findall(r'^place (\w+)(?:\[[^\]]*\])?: (\w+)', model, re.M))


def blocks(output, names):
    """Each ask's lines in a whole run's output: an unindented line naming the ask opens its block."""
    by, cur = {n: [] for n in names}, None
    for line in output.split('\n'):
        if line and not line.startswith(' '):
            m = re.match(r'(\w+)[ :(\[]', line)
            cur = m.group(1) if m and m.group(1) in by else None
        if cur is not None and line:
            by[cur].append(line)
    return by


def shape(lines):
    """The kind of an explanation: its head's sort and each line's depth, vocabulary words and codes."""
    head = lines[0] if lines else ''
    body = head.split(':', 1)[1] if ':' in head else head
    sort = 'refused' if 'refused:' in body else 'hole' if '?"' in body else 'value'
    kinds = set()
    for line in lines[1:]:
        depth = (len(line) - len(line.lstrip(' '))) // 2
        bare = NUMBERISH.sub(' ', QUOTED.sub(' ', line))
        words = tuple(w for w in WORD.findall(bare) if w in WORDS)
        kinds.add((depth, words, tuple(CODE.findall(line)), '?"' in line))
    return sort, frozenset(kinds)


def form(expr, kinds):
    """An explanation's form: its word, the function it explains and the kind of that function's first argument
    (the place's declared kind; the argument itself where it is no place)."""
    m = re.match(r'(explain|inputs-of|readers-of)\s+([\w-]+)?\s*\(?\s*(\w+)?', expr)
    if not m:
        return (expr,)
    word, fn, arg = m.groups()
    return (word, fn, kinds.get(arg, arg))


def pick(c, whole, seen):
    """The asks kept for one vector (text order) and their output lines in the whole run. `seen` holds what the
    vectors before it in RESTATED's order already show, each fact keyed by its model: an ask that isn't an
    explanation, by its name and expression; an explanation, by its form with its head's sort and with each kind of
    line it prints. An ask is kept when it shows something not yet seen; a vector that would keep none keeps its
    first ask, so it still puts a question to its text."""
    asks = text_asks(c)
    by = blocks(whole, [n for n, _ in asks])
    model = model_of(c)
    kinds = place_kinds(model)
    kept = []
    for name, expr in asks:
        if not expr.startswith(EXPLAINS):
            mine = {(model, 'ask', name, expr)}
        else:
            sort, lines = shape(by[name])
            f = form(expr, kinds)
            mine = {(model, f, 'head', sort)} | {(model, f, k) for k in lines}
        if not mine <= seen:
            seen |= mine
            kept.append(name)
    if not kept and asks:
        kept.append(asks[0][0])
    return asks, kept, sum(len(by[n]) for n in kept)


def main(argv):
    R.need_watchdog()
    fields, rs = rows()
    mine = {'fixes/g12/' + n for n in RESTATED}
    chosen = [r for r in rs if r['id'] in mine]
    if len(chosen) != len(RESTATED) or len(set(RESTATED)) != 63:
        sys.exit('restate_g12: %d of the 63 found in vectors.tsv' % len(chosen))
    whole = [dict(r, asks='') for r in chosen]
    pairs = KV.checks_of(whole)
    res = KV.refcore_runs(pairs)
    want = {}
    report = []
    seen = set()
    for (v, c), e in sorted(zip(pairs, res), key=lambda p: RESTATED.index(p[0][0]['id'][len('fixes/g12/'):])):
        asks, kept, lines = pick(c, e['out'], seen)
        want[v['id']] = ' '.join(kept)
        report.append((v['id'], len(asks), len(kept), e['out'].count('\n'), lines))
    if '--show' in argv:
        for vid, na, nk, before, after in report:
            print('%-48s asks %4d -> %3d   lines %5d -> %4d' % (vid, na, nk, before, after))
        print('the 63: %d lines of output in their whole runs, about %d kept' % (
            sum(r[3] for r in report), sum(r[4] for r in report)))
        return 0
    if '--check' in argv:
        bad = [r['id'] for r in chosen if r['asks'] != want[r['id']]]
        if bad:
            print('restate_g12: %d rows of vectors.tsv aren\'t what the rule picks: %s' % (len(bad), ' '.join(bad[:5])))
            return 1
        print('restate_g12: the 63 rows name the asks the rule picks (%d asks)' % sum(r[2] for r in report))
        return 0
    for r in rs:
        if r['id'] in want:
            r['asks'] = want[r['id']]
    with open(LIST, 'w', encoding='utf-8', newline='') as f:
        w = csv.DictWriter(f, fieldnames=fields, delimiter='\t', quoting=csv.QUOTE_NONE, lineterminator='\n')
        w.writeheader()
        w.writerows(rs)
    print('restate_g12: the 63 rows of vectors.tsv now name %d asks; run m1/vectors.py' % sum(r[2] for r in report))
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
