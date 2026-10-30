import csv, re, os
ORDER=['S','M0','M1','M2','M3','M4','M5a','M5b','M5c','M6','M7','M8','M9']
rk={m:i for i,m in enumerate(ORDER)}
ck={r['id']:r for r in csv.DictReader(open('engine/tests/checks.tsv'),delimiter='\t')}
pl={r['id']:r['milestone'] for r in csv.DictReader(open('tmp/session8b/drafts/BUILD-ORDER-placement.tsv'),delimiter='\t')}
def strip(t):
    t=re.sub(r'"[^"\n]*"','""',t)
    t=re.sub(r'--.*','',t)
    return t
feat={
 'parts': (re.compile(r'^\s*(owned by|batched)\b',re.M),'M5b'),
 'use': (re.compile(r'^\s*use\s',re.M),'M5c'),
 'fill': (re.compile(r'\bfill\b'),'M5a'),
 'tou': (re.compile(r'\bto\s+[A-Za-z0-9_]+\b(?=.*\bexp\b|.*\bln\b|.*\bsqrt\b)|\b(exp|ln|sqrt)\s*\('),'M6'),
 'paths': (re.compile(r'\bpaths\s+\d+'),'M7'),
}
out=[]
for i,r in ck.items():
    if i not in pl: continue
    m=pl[i]
    if m in ('M9',): continue
    if not (i.startswith('U:') or i.startswith('I:M:') or i.startswith('I:X:') or i.startswith('I:hardset') or i.startswith('I:light') or i.startswith('I:S')): continue
    txt=''
    for f in (r['files'] or '').split():
        if os.path.isfile(f) and f.endswith('.tally'):
            txt+=strip(open(f,errors='replace').read())+'\n'
    for name,(rx,floor) in feat.items():
        if rx.search(txt) and rk[m]<rk[floor]:
            out.append((i,m,name,floor))
for o in out: print(o)
print(len(out))
