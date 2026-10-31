"""Session 8 step 1: run the chosen checks, counted (m8/refcore) or plain (engine/tools/refcore), in parallel.

    python3 -I -B tmp/session8/tools/shape_runs.py count|plain [ID ...]

Each job runs under /usr/bin/time -l (peak memory footprint) in the check's cwd, on PyPy, as engine/tests/run.py
builds its argv. At most 6 jobs at once (the machine's cores). The 8 GB watchdog must be running (it matches both
command lines). Outputs: tmp/session8/runs/<mode>/<safe id>.{counts,time,err}."""
import os
import subprocess
import sys
import time

ROOT = '/Users/jswob/projects/trop/'
sys.path.insert(0, ROOT + 'engine/tests')
import run as R  # noqa: E402

CHECKS = ['I:A:a05_ania@c05-ania', 'I:E1:tx2_mtg_day', 'I:E2:E2-01~e2_01b', 'I:E2:E2-01~e2_01a',
          'U:set/pairs/PTY-05+OWN-12/ania', 'I:X:replay_september@b_r1']
M8 = ROOT + 'tmp/session8/tools/m8/refcore'


def main():
    mode = sys.argv[1]
    ids = sys.argv[2:] or CHECKS
    if 'memcap.py' not in subprocess.run(['ps', '-axo', 'command='], capture_output=True, text=True).stdout:
        sys.exit('the 8 GB watchdog is not running')
    by = {c['id']: c for c in R.load_checks()}
    out = ROOT + 'tmp/session8/runs/%s/' % mode
    os.makedirs(out, exist_ok=True)
    procs = []
    for i in ids:
        c = by[i]
        safe = i.replace('/', '_').replace(':', '_')
        env = dict(os.environ, COUNT8_OUT=out + safe + '.counts', COUNT8_LIMIT='88')
        interp = M8 if mode == 'count' else R.RC
        cmd = ['/usr/bin/time', '-l', '-o', out + safe + '.time', R.PY['pypy'], '-I', '-B', interp] + R.argv_of(c)
        with open(out + safe + '.err', 'w') as err, open(out + safe + '.out', 'w') as so:
            procs.append((i, time.time(), subprocess.Popen(cmd, cwd=ROOT + c['cwd'], stdout=so, stderr=err,
                                                           env=env)))
        while sum(p.poll() is None for _, _, p in procs) >= 6:
            time.sleep(1)
    for i, t0, p in procs:
        try:
            p.wait(timeout=max(1, 100 - (time.time() - t0)))
        except subprocess.TimeoutExpired:
            p.kill()
            print('%s: killed at 100 s' % i)
        print('%s: rc %s, %.1f s' % (i, p.returncode, time.time() - t0))


if __name__ == '__main__':
    main()
