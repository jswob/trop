"""Milestone M3's purity check of the run function at the binary (ENGINE.md §1 "Pure", LANGUAGE.md §7.5; MILESTONE-M3.md
row 5; issue #226). Standard library only.

The library's own purity is held statically (clippy's disallowed types: no clock, no environment, no file system, no
process, no network outside the CLI's one I/O file, R-38, R-39, R-108) and by every byte-for-byte vector. What this
adds is the binary under a hostile environment: a sample of the stored vectors, the first of each group of each store
(S's kernel, M1's, M2's, M3's check and run stores), each written as a fresh file tree in a temporary folder and run by
the release binary from there (another cwd, deep enough that a `../` key stays inside it), with

    TZ=Pacific/Kiritimati  LC_ALL=LANG=pl_PL.UTF-8  HOME=(an empty folder)  TALLY_DEBUG=1  TALLY_ON=1999-01-01

Each run's exit code and output must equal the stored ones, and the tree must be unchanged after it: the same files,
the same bytes and modification times (the run stores nothing). A vector whose table holds a generated source
(`@keys`, kernel/vectors.py) or whose run is a usage error (its words are the engine's) isn't sampled.

    python3 -I -B engine/tests/m3/purity.py --engine [release | quick | PATH]
                                       -> one line per vector that differs or writes; exit 1 if any (default: release,
                                          engine/target/release/tally; quick: engine/target/quick/tally)
    python3 -I -B engine/tests/m3/purity.py --check   -> the sample's size only (no run; run.py's self-test)

Run at M3's gate, on the release binary (about a minute; memcap.py's watchdog caps each run at 8 GB).
"""
import os
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
T = os.path.dirname(HERE) + '/'
STORES = (('kernel', T + 'kernel/vectors.refcore', 6), ('m1', T + 'm1/vectors.refcore', 6),
          ('m2', T + 'm2/vectors.refcore', 6), ('m3-check', T + 'm3/check.vectors', 8),
          ('m3-runs', T + 'm3/runs.vectors', 8))
ENV = {'TZ': 'Pacific/Kiritimati', 'LC_ALL': 'pl_PL.UTF-8', 'LANG': 'pl_PL.UTF-8', 'TALLY_DEBUG': '1',
       'TALLY_ON': '1999-01-01', 'PATH': '/usr/bin:/bin'}
PROFILES = {p: os.path.dirname(os.path.dirname(T)) + '/target/%s/tally' % p for p in ('release', 'quick')}
DEPTH = 8  # the cwd's depth inside the temporary folder: the deepest `../` key of any store is shallower
BUDGET = 90


def netstrings(b):
    out, i = [], 0
    while i < len(b):
        j = b.index(b':', i)
        n = int(b[i:j])
        out.append(b[j + 1:j + 1 + n])
        i = j + 2 + n
    return out


def files_of(store, table):
    """[(key, bytes)]: a kernel-form table is netstring pairs of key and text; an M3 table is lines of key, a tab, the
    path under engine/tests/."""
    if store.startswith('m3'):
        out = []
        for line in table.decode('utf-8').splitlines():
            key, path = line.split('\t')
            out.append((key, open(T + path, 'rb').read()))
        return out
    ns = netstrings(table)
    return [(ns[k].decode('utf-8'), ns[k + 1]) for k in range(0, len(ns), 2)]


def sample():
    """The first vector of each group of each store: {store, id, args, files, code, out}."""
    out, seen = [], set()
    for store, path, width in STORES:
        parts = netstrings(open(path, 'rb').read())
        for k in range(0, len(parts), width):
            f = [p for p in parts[k:k + width]]
            if width == 6:
                vid, group, args, table, code, whole = f
                source = b'run'
            else:
                vid, group, source, _, args, table, code, whole = f
            if (store, group) in seen or source == b'rule':
                continue
            files = files_of(store, table)
            if any(b'@keys ' in text for _, text in files):
                continue
            seen.add((store, group))
            out.append({'store': store, 'id': vid.decode('utf-8'), 'args': args.decode('utf-8').split(),
                        'files': files, 'code': int(code), 'out': whole})
    return out


def tree(root):
    """{path: (size, mtime_ns, bytes)} of every file under root."""
    out = {}
    for d, _, fs in os.walk(root):
        for f in fs:
            p = os.path.join(d, f)
            st = os.stat(p)
            out[os.path.relpath(p, root)] = (st.st_size, st.st_mtime_ns, open(p, 'rb').read())
    return out


def one(engine, v):
    """Why vector v fails at the binary under the hostile environment, or None."""
    top = tempfile.mkdtemp(prefix='m3-purity-')
    try:
        cwd = os.path.join(top, *['d%d' % i for i in range(DEPTH)])
        os.makedirs(cwd)
        home = os.path.join(top, 'home')
        os.makedirs(home)
        for key, text in v['files']:
            p = os.path.normpath(os.path.join(cwd, key))
            if not p.startswith(top + os.sep):
                return 'a key leaves the temporary folder: %s' % key
            os.makedirs(os.path.dirname(p), exist_ok=True)
            with open(p, 'wb') as fh:
                fh.write(text)
        before = tree(top)
        p = subprocess.run([engine] + v['args'], cwd=cwd, capture_output=True, timeout=BUDGET,
                           env=dict(ENV, HOME=home))
        if p.returncode != v['code'] or p.stdout + p.stderr != v['out']:
            return 'exit %s (stored %s); the output differs from the stored at byte %d' % (
                p.returncode, v['code'], next((i for i, (a, b) in enumerate(zip(p.stdout + p.stderr, v['out']))
                                              if a != b), min(len(p.stdout + p.stderr), len(v['out']))))
        if tree(top) != before:
            return 'the run changed its file tree (it must store nothing)'
        return None
    except subprocess.TimeoutExpired:
        return 'over %d s' % BUDGET
    finally:
        shutil.rmtree(top, ignore_errors=True)


def main(argv):
    vs = sample()
    if '--check' in argv:
        print('m3/purity.py: %d vectors sampled from %d stores (no run without --engine)' % (len(vs), len(STORES)))
        return 0
    if '--engine' not in argv:
        sys.exit('usage: purity.py --engine [release | quick | PATH] | --check')
    word = argv[argv.index('--engine') + 1] if argv.index('--engine') + 1 < len(argv) else 'release'
    engine = PROFILES[word] if word in PROFILES else os.path.abspath(word)
    bad = 0
    for v in vs:
        why = one(engine, v)
        if why:
            bad += 1
            print('DIFFER %s %s: %s' % (v['store'], v['id'], why))
    print('%d of %d vectors differ or write under the hostile environment' % (bad, len(vs)))
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
