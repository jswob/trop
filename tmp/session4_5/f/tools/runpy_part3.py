"""Part 3 edits to engine/tests/run.py (applied after the paths commit): U2 (fast by default, refcore where a check
has no stored baseline), the watchdog requirement, and self-test items for both, for examples.py's lint and for
memcap's match."""
p = '/Users/jswob/projects/trop/engine/tests/run.py'
s = open(p, encoding='utf-8').read()
REP = [
("""- --interp (4.5e): `refcore` (default), `fast` (engine/tools/refcore_fast, the memory layer in front of the same
  refcore) or `both`, which runs each check on the two and requires equal outputs, as --runtime both does.""",
"""- --interp (4.5e): `fast` (engine/tools/refcore_fast, the memory layer in front of the same refcore; the default
  since 4.5f), `refcore`, or `both`, which runs each check on the two and requires equal outputs, as --runtime both
  does. Under `fast`, a check with no stored baseline (verdict `ran`) runs on refcore: a new output comes from refcore
  only (TESTING.md §5).
- The watchdog (4.5f): a run, --self-test and --adhoc refuse to start unless engine/tests/tools/memcap.py is running
  (it caps every refcore and refcore_fast run at 8 GB)."""),
("""def save_results(rt, rows):""", """WATCHDOG = re.compile(r'(^|[\\s/])engine/tests/tools/memcap\\.py(\\s|$)')


def watchdog(ps=None):
    \"\"\"The pid of a running engine/tests/tools/memcap.py, or None.\"\"\"
    if ps is None:
        ps = subprocess.run(['ps', '-axo', 'pid=,command='], capture_output=True, text=True).stdout
    for l in ps.splitlines():
        pid, _, cmd = l.strip().partition(' ')
        if WATCHDOG.search(cmd):
            return int(pid)
    return None


def need_watchdog():
    if watchdog() is None:
        sys.exit('the memory watchdog is not running: start it with\\n'
                 '  nohup python3 -I -B engine/tests/tools/memcap.py 8 5 > /dev/null 2>&1 &\\nand run again')


def own_variant(c, v):
    \"\"\"U2 (4.5f): a check with no stored baseline runs on refcore, whatever variant was asked.\"\"\"
    return v.split('+')[0] if '+fast' in v and not c['baseline'] and not script(c) else v


def save_results(rt, rows):"""),
("""        work = [(c, rt) for c in todo for rt in runtimes]
        with ThreadPoolExecutor(jobs) as ex:
            res = list(ex.map(lambda w: (w[0], w[1], run_one(w[0], w[1], budget, force, rc_hash, cache_dir)), work))""",
"""        work = [(c, rt) for c in todo for rt in runtimes]
        with ThreadPoolExecutor(jobs) as ex:
            res = list(ex.map(lambda w: (w[0], w[1], run_one(w[0], own_variant(w[0], w[1]), budget, force, rc_hash,
                                                               cache_dir)), work))"""),
("""        _, r2, _ = session([law], checks, **dict(kw, runtimes=['pypy+fast+v1']))
        say(e['verdict'] == 'differ' and e['rc'] == 70 and r2['pypy+fast+v1'][0]['verdict'] == 'same',
            'a planted key omission is caught by --verify 1 on LAW-03 (exit 70), and LAW-03 passes without it')""",
"""        _, r2, _ = session([law], checks, **dict(kw, runtimes=['pypy+fast+v1']))
        say(e['verdict'] == 'differ' and e['rc'] == 70 and r2['pypy+fast+v1'][0]['verdict'] == 'same',
            'a planted key omission is caught by --verify 1 on LAW-03 (exit 70), and LAW-03 passes without it')
        # 8. the examples' lint (4.5f, TESTING.md §9)
        p = subprocess.run([sys.executable, '-I', '-B', T + 'map/examples.py'], capture_output=True, text=True)
        say(p.returncode == 0, 'examples.tsv: ' + (p.stdout.strip().split('\\n')[-1] if p.stdout.strip() else p.stderr[-200:]))
        # 9. U2 (4.5f): under fast, a check with no baseline runs on refcore (its cache entry is refcore's)
        nb = dict(fast, baseline='')
        _, r, _ = session([nb], checks, **dict(kw, runtimes=['pypy+fast']))
        say(r['pypy+fast'][0]['verdict'] == 'ran' and os.path.exists(os.path.join(
            cache, cache_key(nb, 'pypy', refcore_hash())[:2], cache_key(nb, 'pypy', refcore_hash()) + '.json')),
            'under --interp fast a check with no baseline runs on refcore (U2)')
        say(main_variants([]) == ['pypy+fast'] and main_variants(['--interp', 'refcore']) == ['pypy'],
            'the default interpreter is fast; --interp refcore still selects refcore')
        # 10. the watchdog: found when its command line names engine/tests/tools/memcap.py, not an old copy
        say(watchdog('  12 python3 -I -B engine/tests/tools/memcap.py 8 5\\n') == 12 and
            watchdog('  13 python3 -I -B tmp/session5/tools/memcap.py 8 5\\n') is None and watchdog() is not None,
            'the watchdog is recognised (engine/tests/tools/memcap.py), an old copy is not, and one is running')
        p = subprocess.run([sys.executable, '-I', '-B', T + 'tools/memcap.py', '--self-test'], capture_output=True, text=True)
        say(p.returncode == 0, 'memcap.py matches refcore and refcore_fast runs: ' + p.stdout.strip())"""),
("""def main(argv):
    if '--adhoc' in argv:
        return adhoc(argv)
    checks = load_checks()
    if '--self-test' in argv:
        return 0 if self_test(checks) else 1""",
"""def main_variants(argv):
    \"\"\"The variants a command line asks for (the interpreter's default is fast, 4.5f U2).\"\"\"
    rt = argv[argv.index('--runtime') + 1] if '--runtime' in argv else 'pypy'
    interp = argv[argv.index('--interp') + 1] if '--interp' in argv else 'fast'
    verify = int(argv[argv.index('--verify') + 1]) if '--verify' in argv else 0
    rts = ['pypy', 'cpython'] if rt == 'both' else [rt]
    fast = lambda r: r + '+fast' + ('+v%d' % verify if verify else '')
    return rts if interp == 'refcore' else [fast(r) for r in rts] if interp == 'fast' else [rts[0], fast(rts[0])]


def main(argv):
    if '--adhoc' in argv:
        need_watchdog()
        return adhoc(argv)
    checks = load_checks()
    if '--self-test' in argv:
        need_watchdog()
        return 0 if self_test(checks) else 1"""),
("""    globs, layers, rt, jobs, budget, interp, verify = [], [], 'pypy', MAX_JOBS, 60.0, 'refcore', 0""",
 """    globs, layers, rt, jobs, budget, interp, verify = [], [], 'pypy', MAX_JOBS, 60.0, 'fast', 0"""),
("""    rts = ['pypy', 'cpython'] if rt == 'both' else [rt]
    fast = lambda r: r + '+fast' + ('+v%d' % verify if verify else '')
    variants = rts if interp == 'refcore' else [fast(r) for r in rts] if interp == 'fast' else [rts[0], fast(rts[0])]
    session(sel, checks, variants, jobs, budget, '--force' in argv)""",
"""    need_watchdog()
    session(sel, checks, main_variants(argv), jobs, budget, '--force' in argv)"""),
]
for a, b in REP:
    assert s.count(a) == 1, a[:80]
    s = s.replace(a, b)
open(p, 'w', encoding='utf-8').write(s)
print('run.py: Part 3 edits applied')
