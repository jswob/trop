"""Memory cap for interpreter jobs (the user's rule, 2026-10-27 12:05): any `refcore_fast ... run` process (4.5e's copy of session 5's memcap.py; only the match and the log differ) whose memory
footprint passes 8 GB is stopped, and logged to tmp/session4_5/e/tools/memcap_fast.log with its command, so the run goes on the
benchmark list for the engine build instead of being run here. The footprint is top's MEM column (resident plus
compressed and swapped pages); RSS alone misses swapped memory.
Usage: python3 -I -B memcap.py [CAP_GB=8] [INTERVAL_S=5]
"""
import re
import subprocess
import sys
import time

CAP = float(sys.argv[1]) if len(sys.argv) > 1 else 8.0
EVERY = float(sys.argv[2]) if len(sys.argv) > 2 else 5.0
LOG = '/Users/jswob/projects/trop/tmp/session4_5/e/tools/memcap_fast.log'
UNIT = {'B': 1 / 1024 ** 3, 'K': 1 / 1024 ** 2, 'M': 1 / 1024, 'G': 1.0, 'T': 1024.0}


def jobs():
    out = subprocess.run(['ps', '-axo', 'pid=,command='], capture_output=True, text=True).stdout
    return {int(l.split(None, 1)[0]): l.split(None, 1)[1] for l in out.splitlines()
            if 'refcore_fast' in l and ' run ' in l and 'memcap' not in l}


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
