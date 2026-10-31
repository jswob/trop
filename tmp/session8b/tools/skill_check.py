"""Paper checks of the engine-rust skill (session 8b, brief §7).

Usage: python3 -I -B skill_check.py <skill dir> <RULEBOOK.md> <BUILDER.md> [<extra.md> ...]
1. Every R-n cited in the skill (and in each extra file, e.g. the reviewer and engine/CLAUDE.md)
   is a row of the rulebook.
2. Every row of BUILDER §8's two tables appears in SKILL.md: each R-n the row names is cited in
   SKILL.md §3, and the row's first code token (the text in its first backticks) is there too.
3. SKILL.md's size: bytes, words, and token estimates (chars/3.5 and chars/3.2; no tokenizer here).
Exit 1 on any miss.
"""
import os
import re
import sys

def rule_ids(text):
    out = set()
    for a, b in re.findall(r'R-(\d+)(?:\s*(?:to|–|\.\.)\s*R-(\d+))?', text):
        lo = int(a); hi = int(b) if b else lo
        out.update(range(lo, hi + 1))
    return out

skill, rulebook, builder = sys.argv[1:4]
extras = sys.argv[4:]
rows = set(int(m) for m in re.findall(r'^\| R-(\d+)\b', open(rulebook, encoding='utf-8').read(), re.M))
bad = 0
files = [os.path.join(dp, f) for dp, _, fs in os.walk(skill) for f in sorted(fs) if f.endswith('.md')] + extras
print('1. cited rule ids exist in the rulebook (%d rows)' % len(rows))
for path in sorted(files):
    cited = rule_ids(open(path, encoding='utf-8').read())
    missing = sorted(cited - rows)
    print('   %-70s cites %3d ids; missing: %s' % (os.path.relpath(path), len(cited), missing or 'none'))
    bad += bool(missing)

skill_md = open(os.path.join(skill, 'SKILL.md'), encoding='utf-8').read()
sec3 = skill_md.split('## 3.', 1)[1].split('\n## 4.', 1)[0]
sec3_ids = rule_ids(sec3)
b = open(builder, encoding='utf-8').read()
s8 = b.split('## 8.', 1)[1].split('\n## 9.', 1)[0]
print('2. BUILDER §8 rows in SKILL.md §3')
n = 0
for line in s8.splitlines():
    if not line.startswith('| ') or line.startswith('| The workaround') or line.startswith('| What blocks') or set(line) <= set('|- '):
        continue
    n += 1
    cells = [c.strip() for c in line.strip().strip('|').split('|')]
    ids = rule_ids(line)
    tok = re.search(r'`([^`]+)`', cells[0])
    tok = tok.group(1) if tok else None
    miss_ids = sorted(ids - sec3_ids)
    tok_ok = tok is None or tok in sec3
    ok = not miss_ids and tok_ok
    bad += not ok
    print('   %-4s %-34s ids %-28s token %s' % ('ok' if ok else 'MISS', cells[0][:34], ','.join('R-%d' % i for i in sorted(ids)) + (' miss ' + str(miss_ids) if miss_ids else ''), (tok or '-') + ('' if tok_ok else ' MISSING')))
print('   %d rows' % n)
print('3. SKILL.md: %d bytes, %d words, about %d–%d tokens (chars/3.5, chars/3.2); limit 5 000'
      % (len(skill_md.encode()), len(skill_md.split()), len(skill_md) / 3.5, len(skill_md) / 3.2))
bad += len(skill_md) / 3.2 > 5000
sys.exit(1 if bad else 0)
