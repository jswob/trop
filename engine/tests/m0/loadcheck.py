"""Milestone M0's row 3 (issue #56; MILESTONE-M0.md): "tools/loadcheck.py (--selftest first) agrees with the loader
over the 79 models, the household and the changes" (BUILD-ORDER § M0, Green). Standard library only.

The realities: each of the 79 models of engine/spec/models/ (a file each; LH-15 and LH-16 at full size), the
household engine/spec/amendments/household/ (a folder: all its .tally files), and X's 7 changes and 3 variants,
engine/tests/integration/amendments/real11/c1..c7, v3..v5 (each a folder, the household with the change made).

The script loads engine/spec/tools/loadcheck.py read-only (importlib, as tools/sums.py loads check_frozen), runs its
--selftest first (it stops unless the planted refusals are all found and no look-alike is), then its scan over each
reality, and refcore's load probe on the same reality (`run <its files> --ask zz_m0_probe`, as m0/vectors.py runs it:
PyPy through run.py's run_one, the cache, the watchdog). They agree when:

  - the scan refuses: refcore refuses with one of the scan's codes at one of its places (file and line), or
  - the scan is clean: refcore loads clean (the probe's REQ-ASK line alone), or refuses with a code the scan doesn't
    check (LOAD-BAND, say; the scan parses no expression and resolves no name).

Any other pair is a STOP-REPORT (the scan, refcore or the spec is wrong): the script stops and writes nothing. It writes
m0/loadcheck.refcore, refcore's probe of each reality in m0/load.refcore's layout (seven netstrings: id, group, cwd,
the argument words, the file table, the exit code, the output; a table's path is relative to engine/tests/, so a
spec text is `../spec/…`), m0/loadcheck.tsv (id, the scan's verdict, refcore's first line), and
runner/src/m0_spec_texts.rs (an include_str! per spec text, beside m0_texts.rs's). milestone/m0_loadcheck.rs holds the
engine's probe to refcore's on each reality, so the engine agrees with the scan where refcore does.

    python3 -I -B engine/tests/m0/loadcheck.py            -> writes the three files
    python3 -I -B engine/tests/m0/loadcheck.py --check    -> exit 1 if any isn't what the runs give
"""
import importlib.util
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(HERE))
sys.path.insert(0, HERE)
import run as R  # noqa: E402
import vectors as V  # noqa: E402

SCAN = R.ROOT + 'engine/spec/tools/loadcheck.py'
STORE = os.path.join(HERE, 'loadcheck.refcore')
VERDICTS = os.path.join(HERE, 'loadcheck.tsv')
SPEC_RS = R.T + 'runner/src/m0_spec_texts.rs'
TESTS = 'engine/tests'
CHANGES = ['c1', 'c2', 'c3', 'c4', 'c5', 'c6', 'c7', 'v3', 'v4', 'v5']


def scan_module():
    spec = importlib.util.spec_from_file_location('loadcheck', SCAN)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def realities():
    """[(id, group, cwd, [files, relative to cwd])], the models, the household, the changes."""
    out = []
    models = 'engine/spec/models'
    for n in sorted(f for f in os.listdir(R.ROOT + models) if f.endswith('.tally')):
        out.append(('models/' + n[:-len('.tally')], 'models', models, [n]))
    folders = [('household', 'household', 'engine/spec/amendments/household')]
    folders += [('changes/' + c, 'changes', 'engine/tests/integration/amendments/real11/' + c) for c in CHANGES]
    for rid, group, cwd in folders:
        files = sorted(os.path.relpath(p, R.ROOT + cwd).replace(os.sep, '/') for p in V.collect(R.ROOT + cwd))
        out.append((rid, group, cwd, files))
    return out


def probe(rs):
    """refcore's load probe on each reality, through m0/vectors.py's runner."""
    vecs = []
    for rid, group, cwd, files in rs:
        s = {'id': rid, 'cwd': cwd, 'args': files, 'table': [(f, cwd + '/' + f) for f in files]}
        vecs.append(({'id': rid}, V.probe_check(s, V.PROBE)))
    return V.load_runs(vecs)


def scanned(mod, group, cwd, files):
    """The scan's refusals on a reality (a model's file, or a folder): [(code, file:line relative to cwd)]."""
    path = R.ROOT + cwd
    r = mod.check(mod.reality_of_file(path + '/' + files[0]) if group == 'models' else mod.reality_of_dir(path))
    return [(code, os.path.relpath(loc, path) if loc.startswith(path) else loc) for code, loc, _, _ in r.refusals]


def agree(scan_codes, refusals, out):
    """Why the scan and refcore disagree on a reality, or None."""
    m = re.match(r'refused: (\S+) .*?(?:\(([^()]*:\d+)\))?$', out.split('\n', 1)[0])
    if refusals:
        if not m or (m.group(1), m.group(2)) not in [(c, loc) for c, loc in refusals]:
            return 'the scan refuses %s; refcore prints %r' % (refusals[0], out[:200])
        return None
    if out == V.PROBED or (m and m.group(1) not in scan_codes):
        return None
    return 'the scan is clean; refcore prints %r' % out[:200]


def netstrings(b):
    out, at = [], 0
    while at < len(b):
        colon = b.index(b':', at)
        n = int(b[at:colon])
        out.append(b[colon + 1:colon + 1 + n])
        at = colon + n + 2
    return out


def controls(mod, scan_codes):
    """The controls: M0's code fixtures (codes/m0.refcore) of a code the scan checks, each scanned from its stored file
    table; the scan must refuse each as refcore does, so the agreement isn't only "both clean". [(id, why or None)]"""
    ns = netstrings(open(R.T + 'codes/m0.refcore', 'rb').read())
    out = []
    for k in range(0, len(ns), 7):
        vid, code, out_ = ns[k].decode(), ns[k + 1].decode(), ns[k + 6].decode()
        if code not in scan_codes:
            continue
        r = mod.Reality(vid)
        pairs = [p.decode() for p in netstrings(ns[k + 4])]
        for key, text in zip(pairs[0::2], pairs[1::2]):
            r.files[key] = [mod.Line('%s:%d' % (key, n), t) for n, t in enumerate(text.split('\n'), 1)]
        refusals = [(c, loc) for c, loc, _, _ in mod.check(r).refusals]
        out.append((vid, agree(scan_codes, refusals, out_) or (None if refusals else 'the scan finds nothing')))
    return out


def spec_rs(rs):
    paths = sorted({os.path.relpath(R.ROOT + cwd + '/' + f, R.ROOT + TESTS).replace(os.sep, '/')
                    for _, _, cwd, files in rs for f in files if not cwd.startswith(TESTS)})
    out = ['//! The spec\'s texts that row 3 of `MILESTONE-M0.md` loads (`m0/loadcheck.refcore`), by their path',
           '//! relative to `engine/tests/`, sorted: generated by `m0/loadcheck.py` (its `--check` holds this file',
           '//! to it), beside `m0_texts.rs`. Never edited by hand.',
           '',
           '/// Each text: its path relative to `engine/tests/` and its content.',
           'pub static SPEC_TEXTS: [(&str, &str); %d] = [' % len(paths)]
    for p in paths:
        out += ['    (', '        "%s",' % p, '        include_str!("../../%s"),' % p, '    ),']
    out.append('];')
    return '\n'.join(out) + '\n'


def main(argv):
    R.need_watchdog()
    mod = scan_module()
    if mod.selftest() != 0:
        sys.exit('loadcheck.py --selftest fails: a STOP-REPORT, not a vector')
    rs = realities()
    res = probe(rs)
    scan_codes = {c for c in re.findall(r'^  (LOAD-[A-Z]+) ', mod.__doc__, re.M)}
    rows, bad, data = [], [], b''
    for (rid, group, cwd, files), e in zip(rs, res):
        refusals = scanned(mod, group, cwd, files)
        why = agree(scan_codes, refusals, e['out'])
        if why:
            bad.append('%s: %s' % (rid, why))
        verdict = ' '.join('%s %s' % r for r in refusals) or 'clean'
        rows.append('%s\t%s\t%s\n' % (rid, verdict, e['out'].split('\n', 1)[0]))
        table = ''.join('%s\t%s\n' % (f, os.path.relpath(R.ROOT + cwd + '/' + f, R.ROOT + TESTS).replace(os.sep, '/'))
                        for f in files)
        data += (V.ns(rid) + V.ns(group) + V.ns(cwd) + V.ns(' '.join(['run'] + files + ['--ask', V.PROBE])) +
                 V.ns(table) + V.ns(str(e['rc'])) + V.ns(e['out']))
    ctl = controls(mod, scan_codes)
    bad += ['control %s: %s' % (vid, why) for vid, why in ctl if why]
    if not ctl:
        bad.append('no control: codes/m0.refcore holds no fixture of a code the scan checks')
    for b in bad:
        print('FAIL ' + b)
    if bad:
        sys.exit('%d of %d realities: the scan and refcore disagree: a STOP-REPORT, not a vector' % (len(bad), len(rs)))
    tsv = 'id\tloadcheck\trefcore\n' + ''.join(rows)
    rs_text = spec_rs(rs)
    clean = sum(r.split('\t')[1] == 'clean' for r in rows)
    if '--check' in argv:
        olds = [open(p, 'rb').read() if os.path.exists(p) else b'' for p in (STORE, VERDICTS, SPEC_RS)]
        if olds != [data, tsv.encode('utf-8'), rs_text.encode('utf-8')]:
            print('m0/loadcheck.refcore, m0/loadcheck.tsv or runner/src/m0_spec_texts.rs is not what the runs give: '
                  'run m0/loadcheck.py')
            return 1
        print('m0/loadcheck.refcore: %d realities and %d controls; the scan (selftest passed) and refcore agree on '
              'each' % (len(rs), len(ctl)))
        return 0
    with open(STORE, 'wb') as f:
        f.write(data)
    with open(VERDICTS, 'w', encoding='utf-8', newline='') as f:
        f.write(tsv)
    with open(SPEC_RS, 'w', encoding='utf-8', newline='') as f:
        f.write(rs_text)
    print('m0/loadcheck.refcore: %d realities written; the scan finds %d clean; refcore loads %d clean; %d controls' % (
        len(rs), clean, sum(e["out"] == V.PROBED for e in res), len(ctl)))
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
