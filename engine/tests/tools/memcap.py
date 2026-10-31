"""Memory cap for interpreter jobs (the user's rule, 2026-10-27 12:05): any interpreter run, `refcore run` or
`refcore_fast [--verify N] run` (4.5f: one watchdog for both), or the engine's `target/release/tally run` (T1, run.py
--interp engine), whose memory footprint passes 8 GB is stopped, and logged
to engine/tests/out/memcap.log with its command, so the run goes on the benchmark list for the engine build instead of
being run here. The footprint is top's MEM column (resident plus compressed and swapped pages); RSS alone misses
swapped memory. run.py refuses to start a run unless this script is running.
Usage: python3 -I -B engine/tests/tools/memcap.py [CAP_GB=8] [INTERVAL_S=5]
       python3 -I -B engine/tests/tools/memcap.py --self-test     (the match, on command lines)
"""
import os
import re
import subprocess
import sys
import time

LOG = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), 'out', 'memcap.log')
JOB = re.compile(r'(?:/refcore(?:_fast)?(?: --\S+(?: \d+)?)*|/target/release/tally) run ')  # refcore run …, refcore_fast --verify 50 run …, the engine's binary (T1)
UNIT = {'B': 1 / 1024 ** 3, 'K': 1 / 1024 ** 2, 'M': 1 / 1024, 'G': 1.0, 'T': 1024.0}


def jobs():
    out = subprocess.run(['ps', '-axo', 'pid=,command='], capture_output=True, text=True).stdout
    return {int(l.split(None, 1)[0]): l.split(None, 1)[1] for l in out.splitlines()
            if JOB.search(l) and 'memcap' not in l}


def footprints(pids):
    if not pids:
        return {}
    cmd = ['top', '-l', '1', '-stats', 'pid,mem']
    for p in pids:
        cmd += ['-pid', str(p)]
    out = subprocess.run(cmd, capture_output=True, text=True).stdout
    res = {}
    for l in out.splitlines():
        m = re.match(r'^\s*(\d+)\s+([\d.]+)([BKMGT])\+?-?\s*$', l)
        if m:
            res[int(m.group(1))] = float(m.group(2)) * UNIT[m.group(3)]
    return res


def log(line):
    with open(LOG, 'a', encoding='utf-8') as f:
        f.write(time.strftime('%Y-%m-%d %H:%M:%S ') + line + '\n')


def self_test():
    py = '/u/pypy3.11 -I -B '
    yes = [py + '/r/engine/tools/refcore run unit/probes/a.tally', py + '/r/engine/tools/refcore_fast run a.tally --ask x',
           py + '/r/engine/tools/refcore_fast --verify 50 run a.tally', py + '/r/engine/tools/refcore_fast --plant run a.tally',
           '/r/engine/target/release/tally run a.tally --ask x']
    no = [py + '/r/engine/tools/refcore test', 'python3 -I -B engine/tests/run.py --interp fast',
          'python3 -I -B engine/tests/tools/memcap.py 8 5', 'python3 -I -B engine/tests/run.py --interp engine',
          '/r/engine/target/release/tally-runner run a.tally']
    bad = [c for c in yes if not JOB.search(c)] + [c for c in no if JOB.search(c)]
    print('memcap self-test %s%s' % ('PASSED' if not bad else 'FAILED: ', '; '.join(bad)))
    return not bad


if '--self-test' in sys.argv:
    sys.exit(0 if self_test() else 1)
CAP = float(sys.argv[1]) if len(sys.argv) > 1 else 8.0
EVERY = float(sys.argv[2]) if len(sys.argv) > 2 else 5.0
os.makedirs(os.path.dirname(LOG), exist_ok=True)
log('memcap started: cap %.1f GB, every %.0f s' % (CAP, EVERY))
while True:
    js = jobs()
    for pid, gb in footprints(list(js)).items():
        if gb > CAP:
            try:
                subprocess.run(['kill', str(pid)])
                time.sleep(2)
                subprocess.run(['kill', '-9', str(pid)], stderr=subprocess.DEVNULL)
            finally:
                log('stopped pid %d at %.1f GB: %s' % (pid, gb, js.get(pid, '')[:600]))
    time.sleep(EVERY)
