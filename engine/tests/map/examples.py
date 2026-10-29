#!/usr/bin/env python3
"""The examples and their three levels (session 4.5f): TESTING.md §9, the recipe, kept as a record and checked by machine.

    python3 -I -B engine/tests/map/examples.py --build    # back-fill examples.tsv from tags.tsv, checks.tsv, load.tsv
    python3 -I -B engine/tests/map/examples.py            # the lint: exit 1 on any fault

An example is an English description the suite was made from: a household (its story), a law text (its provision in
the case set's acts.md), an old model (its story header), a light twin, a blind text. Its row names:
- unit: the catalogue edges tagged (map/tags.tsv) on its integration checks' questions or texts; for a law text only
  the LAW-axis ones (a household loads its whole law set, so its other edges say nothing of one text);
- integration: its checks in checks.tsv, as ids or id globs (a glob stands for every check of a household);
- load: its rows of load/load.tsv;
each one, when there is none, as `none: <reason>`.

The lint fails on: an example missing one of its three entries; an edge, check id, glob or load row that doesn't
exist; an edge with no unit check; a retired (`not-run:`) check whose example has no load row; an integration check
that belongs to no example; a unit check whose edges no example meets, unless the edge is probe-only by TESTING.md
§2 (PROBE_ONLY).
"""
import csv
import fnmatch
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))) + '/'
T = 'engine/tests/'
OUT = ROOT + T + 'examples.tsv'
COLS = ['example', 'kind', 'story', 'unit', 'integration', 'load']
HOUSES = ('A', 'B', 'D', 'E1')
EDGE = re.compile(r'\b[A-Z]+-\d+\b')
# TESTING.md §2: the SURF writing rules met by a probe card only (map/gaps.md, accepted c)
PROBE_ONLY = {'SURF-01', 'SURF-02', 'SURF-04'}


def tsv(p, skip_hash=False):
    with open(ROOT + p, encoding='utf-8', newline='') as f:
        lines = [l for l in f if not (skip_hash and l.startswith('#'))]
    return list(csv.DictReader(lines, delimiter='\t', quoting=csv.QUOTE_NONE))


checks = tsv(T + 'checks.tsv')
ids = [c['id'] for c in checks]
by_id = {c['id']: c for c in checks}
tags = {r['id']: r['edges'].split() for r in tsv(T + 'map/tags.tsv')}
loads = tsv(T + 'load/load.tsv', skip_hash=True)
catalogue = [m.group(1) for m in re.finditer(r'(?m)^### ([A-Z]+-\d+) · ', open(ROOT + T + 'cases/boundaries.md',
                                                                              encoding='utf-8').read())]


def unit_edges(c):
    """A unit check's catalogue edges: those its id names (U:law/LAW-13/rate25, U:pairs/CUR-01+RAND-01), else its
    tags.tsv row's (a probe outside the card scheme, e.g. U:time/TIME-08-before, tagged TIME-07)."""
    es = [e for e in EDGE.findall(c['id']) if e in catalogue]
    return es or [e for e in tags.get(c['id'], []) if e in catalogue]


unit_of = {}  # edge -> its unit checks
for c in checks:
    if c['layer'] == 'unit':
        for e in unit_edges(c):
            unit_of.setdefault(e, []).append(c['id'])


def tag_id(c):
    """The tags.tsv row a check's edges come from."""
    i = c['id']
    m = re.match(r'^I:(A|B|D|E1):', i)
    if m:
        q = re.sub(r'[a-z]+$', '', c['question'])
        return 'I:%s:%s' % (m.group(1), q) if re.match(r'^[A-Z]\d?-\d+$', q) else None
    if i.startswith('I:M:'):
        return i.split('~')[0]
    if i.startswith('I:light/'):
        return i.split('~')[0]
    if i.startswith('I:hardset/b2fix/'):
        return 'I:hardset/modeller-b2/I-12/division'
    if i.startswith('I:hardset/'):
        return i
    return None


def law_reads(c):
    """The law texts a check reads, by provision (a file <PROV>.tally in a directory named law)."""
    return {os.path.basename(f)[:-len('.tally')] for f in c['files'].split()
            if os.path.basename(os.path.dirname(f)) == 'law' and f.endswith('.tally')}


def compress(members, scope):
    """Check ids -> ids, with `I:<H>:*` for a household whose every check is a member."""
    out, rest = [], set(members)
    for h in HOUSES + ('X',):
        allh = {i for i in scope if i.startswith('I:%s:' % h)}
        if allh and allh <= rest:
            out.append('I:%s:*' % h)
            rest -= allh
    return out + sorted(rest, key=ids.index)


def load_rows(members, words):
    """load.tsv rows whose capability checks match a member, or whose `what` names the example."""
    out = []
    for r in loads:
        pats = [t.rstrip('.,;') for t in re.findall(r'\b[IU]:\S+', r['capability checks'])]
        hit = any(fnmatch.fnmatchcase(i, p) for p in pats for i in members)
        if hit or any(re.search(w, r['what']) for w in words):
            out.append(r['id'])
    return out


def build():
    integ = [c for c in checks if c['layer'] == 'integration']
    rows = []

    def row(ex, kind, story, members, tag_ids, words, none_integ=''):
        es = []
        for t in tag_ids:
            es += [e for e in tags.get(t, []) if e not in es and (kind != 'law text' or e.startswith('LAW-'))]
        es.sort(key=lambda e: catalogue.index(e) if e in catalogue else len(catalogue))
        ld = load_rows(members, words)
        retired = [i for i in members if by_id[i]['note'].startswith('not-run')]
        rows.append({'example': ex, 'kind': kind, 'story': story,
                     'unit': ' '.join(es) or 'none: no edge is tagged on its checks',
                     'integration': ' '.join(compress(members, ids)) or 'none: ' + none_integ,
                     'load': ' '.join(ld) or ('none: no load.tsv row names it; its checks run at full size'
                                              if not retired else 'none: MISSING (retired checks with no load row)')})

    for h in HOUSES:
        ms = [c['id'] for c in integ if c['id'].startswith('I:%s:' % h)]
        row('household:' + h, 'household', T + 'cases/%s/story.md' % h, ms, {tag_id(by_id[i]) for i in ms} - {None},
            [r'household %s\b' % h, r"\b%s's\b" % h])
    ms = [c['id'] for c in integ if c['id'].startswith('I:X:')]
    row('household:X', 'household', 'engine/spec/amendments/README.md', ms, [t for t in tags if t.startswith('I:X:')],
        [r'amendments household', r'household-bands', r"household's explain_room", r"change 1's"])
    for p in sorted(os.listdir(ROOT + T + 'integration/law')):
        prov = p[:-len('.tally')]
        ms = [c['id'] for c in integ if prov in law_reads(c)]
        row('law:' + prov, 'law text', T + 'cases/law/acts.md#' + prov, ms, {tag_id(by_id[i]) for i in ms} - {None}, [],
            'no check reads it (the households read the law texts their questions need)')
    for p in sorted(os.listdir(ROOT + 'engine/spec/models')):
        mid = p[:-len('.tally')]
        ms = [c['id'] for c in integ if c['id'].split('~')[0] == 'I:M:' + mid]
        row('model:' + mid, 'old model', 'engine/spec/models/' + p, ms, ['I:M:' + mid], [r'\b%s\b' % mid],
            'no check runs it at full size; its light twin is the check (load only at full size)')
    for p in sorted(os.listdir(ROOT + T + 'integration/light')):
        name = p[:-len('.tally')]
        base = re.match(r'^(LH-\d+)', name).group(1)
        ms = [c['id'] for c in integ if c['id'].split('~')[0] == 'I:light/' + name]
        row('twin:' + name, 'light twin', 'engine/spec/models/%s.tally' % base, ms, ['I:light/' + name], [])
    blind = sorted({m.group(1) for m in (re.search(r'/(I-\d+)/', t) for t in tags if t.startswith('I:hardset/')) if m},
                   key=lambda s: int(s[2:]))
    for b in blind:
        ms = [c['id'] for c in integ if c['id'].startswith('I:hardset/') and tag_id(c) and '/%s/' % b in tag_id(c)]
        story = [os.path.join(d, f) for d in ('integration/blind/invented-b1', 'integration/blind/invented-b2')
                 for f in sorted(os.listdir(ROOT + T + d)) if f.startswith(b + '-')]
        row('blind:' + b, 'blind text', T + story[0] if story else 'none: MISSING', ms, {tag_id(by_id[i]) for i in ms}, [])
    with open(OUT, 'w', encoding='utf-8') as f:
        f.write('# The examples and their three levels (TESTING.md §9). Written by map/examples.py --build; '
                'never edit by hand.\n')
        f.write('\t'.join(COLS) + '\n')
        for r in rows:
            f.write('\t'.join(r[c] for c in COLS) + '\n')
    print('%d examples -> %s' % (len(rows), OUT[len(ROOT):]))


def lint():
    rows = tsv(T + 'examples.tsv', skip_hash=True)
    bad = []
    load_ids = {r['id'] for r in loads}
    met, member = set(), set()
    for r in rows:
        ex = r['example']
        for col in ('story', 'unit', 'integration', 'load'):
            v = r[col].strip()
            if not v or v.startswith('none: MISSING') or (v.startswith('none:') and len(v) < 8):
                bad.append('%s: no %s entry' % (ex, col))
        story = r['story'].split('#')[0]
        if not r['story'].startswith('none:') and not os.path.isfile(ROOT + story):
            bad.append('%s: story %s does not exist' % (ex, story))
        elif '#' in r['story'] and not re.search(r'(?m)^### %s · ' % re.escape(r['story'].split('#')[1]),
                                                  open(ROOT + story, encoding='utf-8').read()):
            bad.append('%s: %s has no section %s' % (ex, story, r['story'].split('#')[1]))
        if not r['unit'].startswith('none:'):
            for e in r['unit'].split():
                if e not in catalogue:
                    bad.append('%s: edge %s is not in the catalogue' % (ex, e))
                elif e not in unit_of:
                    bad.append('%s: edge %s has no unit check' % (ex, e))
                met.add(e)
        ms = set()
        if not r['integration'].startswith('none:'):
            for i in r['integration'].split():
                got = [x for x in ids if fnmatch.fnmatchcase(x, i)] if '*' in i else ([i] if i in by_id else [])
                if not got:
                    bad.append('%s: check %s does not exist' % (ex, i))
                ms |= set(got)
        member |= ms
        lds = [] if r['load'].startswith('none:') else r['load'].split()
        for l in lds:
            if l not in load_ids:
                bad.append('%s: load row %s does not exist' % (ex, l))
        if not lds and any(by_id[i]['note'].startswith('not-run') for i in ms):
            bad.append('%s: retired checks but no load row' % ex)
    for c in checks:
        if c['layer'] == 'integration' and c['id'] not in member:
            bad.append('check %s belongs to no example' % c['id'])
        if c['layer'] == 'unit':
            es = set(unit_edges(c))
            if not es:
                bad.append('unit check %s names no catalogue edge and has no tags.tsv row' % c['id'])
            elif not es & (met | PROBE_ONLY):
                bad.append('unit check %s: no example meets its edges %s' % (c['id'], ' '.join(sorted(es))))
    for b in bad:
        print('FAIL', b)
    kinds = {}
    for r in rows:
        kinds[r['kind']] = kinds.get(r['kind'], 0) + 1
    print('examples.tsv: %d examples (%s); %d integration checks in them; %d edges met; %s' % (
        len(rows), ', '.join('%d %s' % (n, k) for k, n in kinds.items()), len(member), len(met),
        'lint PASSED' if not bad else 'lint FAILED (%d)' % len(bad)))
    return not bad


if __name__ == '__main__':
    if '--build' in sys.argv:
        build()
    sys.exit(0 if lint() else 1)
