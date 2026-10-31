"""The sha256 pins of a joint model's use lines, computed by the frozen refcore's own joint.member_texts and
member_hash (read-only import, no run): python3 -I -B pin.py FAMILY.tally [--write]. Prints each use line's written
pin and the member's hash now; --write replaces a pin that differs (the line's other bytes stay)."""
import os, re, sys
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))))
sys.path.insert(0, os.path.join(ROOT, 'engine', 'tools'))
from refcore.joint import member_texts, member_hash
USE = re.compile(r'^(\s*use\s+(\w+)\s*=\s*"([^"]+)"\s+sha256\s+")([0-9a-f]{64})(")', re.M)
p = os.path.abspath(sys.argv[1])
src = open(p, encoding='utf-8').read()
base = os.path.dirname(p)
new, changed = src, 0
for m in USE.finditer(src):
    alias, target, pin = m.group(2), m.group(3), m.group(4)
    files = member_texts(alias, os.path.basename(p), os.path.normpath(os.path.join(base, target)), [], True)
    h = member_hash(files, None, [], True)
    print('%s %s: written %s, now %s %s' % (alias, target, pin[:12], h[:12], 'same' if h == pin else 'DIFFERS'))
    if h != pin:
        new = new.replace(m.group(0), m.group(1) + h + m.group(5))
        changed += 1
if '--write' in sys.argv and changed:
    open(p, 'w', encoding='utf-8').write(new)
    print('re-pinned %d use line(s) in %s' % (changed, sys.argv[1]))
