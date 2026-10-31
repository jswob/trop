"""Stage the paths commit only: every modified suite file except Part 1/3 work; TESTING.md as rw(HEAD) (its path
hunk alone, the recipe stays in the working tree)."""
import os, subprocess, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import paths
EXCL = ('engine/tests/cases/', 'engine/tests/tools/answers.py', 'engine/tests/tools/memcap.py')
mod = [l[3:] for l in subprocess.run(['git', 'status', '--porcelain', '--', 'engine/tests'], capture_output=True,
                                     text=True, check=True).stdout.splitlines() if l.startswith(' M')]
add = [f for f in mod if not f.startswith(EXCL) and f != 'engine/tests/TESTING.md'] + ['engine/tests/.gitignore']
subprocess.run(['git', 'add', '--'] + add, check=True)
head = subprocess.run(['git', 'show', 'HEAD:engine/tests/TESTING.md'], capture_output=True, text=True, check=True).stdout
blob = subprocess.run(['git', 'hash-object', '-w', '--stdin'], input=paths.rw(head), capture_output=True, text=True,
                      check=True).stdout.strip()
subprocess.run(['git', 'update-index', '--cacheinfo', '100644,%s,engine/tests/TESTING.md' % blob], check=True)
print('staged %d files and TESTING.md\'s path hunk' % len(add))
