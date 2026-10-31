"""Fill a prompt template: python3 -I -B fill.py TEMPLATE OUT KEY=@file|KEY=value ...

Each {KEY} in the template is replaced by the value, or by a file's contents with @.
Fails if a {UPPERCASE} placeholder is left.
"""
import re
import sys

tmpl, out = open(sys.argv[1], encoding='utf-8').read(), sys.argv[2]
for arg in sys.argv[3:]:
    k, v = arg.split('=', 1)
    if v.startswith('@'):
        v = open(v[1:], encoding='utf-8').read().rstrip('\n')
    tmpl = tmpl.replace('{' + k + '}', v)
left = sorted(set(re.findall(r'\{[A-Z_]+\}', tmpl)))
if left:
    sys.exit('left: ' + ' '.join(left))
open(out, 'w', encoding='utf-8').write(tmpl)
print('wrote', out, len(tmpl.splitlines()), 'lines')
