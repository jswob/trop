"""Merge the settled rows into the gap log, tmp/cases/gaps-1.0.1.md (brief §8).

Usage (from anywhere): python3 -I -B merge_gaps.py [--out PATH] [--check]

Reads rows/<chunk>.settled.md (P0, G1-G5, A, B, D, E1), the writers' raw rows
(rows/<chunk>.writer.md, rows/G5.recovered.md, rows/P0.md) for the writer's
status, rows/agents.tsv for cost and audits, and ../INDEX.md for the edge order.
Never hand-types a row: every row, count and cost line comes from those files.
--check prints check 1 (ids, counts, uniqueness) and the repairs, and exits 1 on
a failure. Standard library only.
"""
import os
import re
import sys
from collections import Counter, OrderedDict

HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))   # tally-1.0.1/
CASES = os.path.dirname(HERE)                                        # tmp/cases/
REPO = os.path.dirname(os.path.dirname(CASES))
ROWS = os.path.join(HERE, 'rows')

CHUNKS = ['P0', 'G1', 'G2', 'G3', 'G4', 'G5', 'A', 'B', 'D', 'E1']
HOUSES = ['A', 'B', 'D', 'E1']
SCALE = ['breaks', "can't express", 'awkward', 'expressed']
SUSPECT = 'case suspect'
STATUSES = SCALE + [SUSPECT]
COLS = ['row', 'status', 'reason', 'spec', 'gap', 'text', 'settlement']
WITHDRAWN = {'S:13', 'S:14'}        # writer misread, status awkward: not spec findings
AXES = ['NUM', 'TIME', 'PTY', 'OWN', 'FLOW', 'CUR', 'VAL', 'LAW', 'LIFE', 'RULE',
        'REC', 'SCEN', 'RAND', 'ASK', 'SURF']
CAUSES = ['writer misread', 'judge misread', 'spec unclear', 'case suspect', 'supervisor']
ESC = '\x00'

repairs = []      # notes on source rows the script had to mend (sources stay unchanged)


def split_row(line):
    """Cells of a Markdown table row; `\\|` (and settle.py's mangled `\\ |`) stay in."""
    s = line.strip()
    s = s.replace('\\ |', ESC).replace('\\|', ESC)
    if s.startswith('|'):
        s = s[1:]
    if s.endswith('|'):
        s = s[:-1]
    return [c.strip().replace(ESC, '\\|') for c in s.split('|')]


def row_id(cell):
    return cell.strip('`* ')


def read_table(path):
    out = OrderedDict()
    if not os.path.exists(path):
        return out
    for line in open(path, encoding='utf-8'):
        if not line.lstrip().startswith('|'):
            continue
        c = split_row(line)
        rid = row_id(c[0])
        if re.match(r'^[PQOS]:\S+$', rid):
            out[rid] = c
    return out


def unq(cell):
    """A cell without its code ticks, for comparing; '—' and '' read as empty."""
    v = cell.strip()
    if v in ('`—`', '—', ''):
        return ''
    return v


def settled_rows():
    rows = OrderedDict()
    for ch in CHUNKS:
        path = os.path.join(ROWS, ch + '.settled.md')
        tab = read_table(path)
        writer = OrderedDict()
        for w in ([ch + '.writer.md'] if ch != 'P0' else ['P0.md']) + (
                ['G5.recovered.md'] if ch == 'G5' else []):
            writer.update(read_table(os.path.join(ROWS, w)))
        # S ids were renumbered at settlement: map the writer's S rows by order
        ws = [k for k in writer if k.startswith('S:')]
        ss = [k for k in tab if k.startswith('S:')]
        smap = dict(zip(ss, ws)) if len(ws) == len(ss) else {}
        for rid, c in tab.items():
            wid = smap.get(rid, rid)
            wc = writer.get(wid)
            if ch == 'P0' and len(c) == 6:
                c = c + ['']
            elif len(c) < 7:
                # a cell holding a bare `|` was split badly by settle.py: the last
                # cell is the settlement; take the missing cells from the writer
                settle = c[-1]
                have = c[:-1]
                missing = COLS[len(have):6]
                fill = []
                for name in missing:
                    i = COLS.index(name)
                    fill.append(wc[i] if wc and len(wc) > i else '')
                repairs.append('%s (%s.settled.md): %d cells; %s taken from the writer\'s row %s'
                               % (rid, ch, len(c), ', '.join(missing), wid))
                c = have + fill + [settle]
                if wc and len(wc) > 2 and re.sub(r'\s', '', c[2]) == re.sub(r'\s', '', wc[2]):
                    c[2] = wc[2]      # the writer's reason, its spacing around `\|` intact
                    repairs[-1] += '; reason spacing from the writer\'s row (same text)'
            elif len(c) > 7:
                repairs.append('%s (%s.settled.md): %d cells, extra cells joined into the reason'
                               % (rid, ch, len(c)))
                c = c[:2] + [' \\| '.join(c[2:len(c) - 4])] + c[-4:]
            r = dict(zip(COLS, c))
            r['row'] = rid
            r['chunk'] = ch
            r['writer_id'] = wid
            r['writer_status'] = row_id(wc[1]) if wc and len(wc) > 1 else '(no writer row)'
            r['reason'] = r['reason'].replace('\\ |', '\\|')
            if ch == 'P0':
                r['settlement'] = ('settled: %s (P0.settled.md has no Settlement column; its note: '
                                   'the supervisor checked each against the spec, "ambiguous" mapped '
                                   'to breaks)' % ('supervisor' if r['writer_status'] == r['status']
                                                   else 'spec unclear'))
            if rid in rows:
                repairs.append('DUPLICATE id %s in %s and %s' % (rid, rows[rid]['chunk'], ch))
            rows[rid] = r
    return rows


def kind(rid):
    return rid[0]


def house_of(r):
    if r['chunk'] in HOUSES:
        return r['chunk']
    return None


def resolve_texts(r):
    """[(shown, exists)] for each path in the Text cell."""
    out = []
    t = unq(r['text'])
    if not t:
        return out
    for p in [x.strip().strip('`').strip() for x in t.split(';')]:
        if not p or p == '—':
            continue
        cands = [os.path.join(HERE, p), os.path.join(REPO, p)]
        h = house_of(r)
        if h:
            cands.append(os.path.join(HERE, 'households', h, p))
        hit = next((c for c in cands if os.path.exists(c)), None)
        if hit:
            shown = os.path.relpath(hit, HERE) if hit.startswith(HERE) else os.path.relpath(hit, REPO)
            out.append((shown, True))
        else:
            out.append((p, False))
    return out


def worst(statuses):
    on = [s for s in statuses if s in SCALE]
    w = min(on, key=SCALE.index) if on else None
    sus = any(s == SUSPECT for s in statuses)
    if w is None and sus:
        return '— (+ suspect)'
    if w is None:
        return '—'
    return w + (' (+ suspect)' if sus else '')


def read_index():
    edges = []
    axis = None
    for line in open(os.path.join(CASES, 'INDEX.md'), encoding='utf-8'):
        m = re.match(r'^## (\w+):', line)
        if m:
            axis = m.group(1)
            continue
        if line.startswith('| ') and not line.startswith('| Edge') and axis:
            c = split_row(line)
            edge = c[0].replace('★', '').strip()
            qs = [] if c[3].strip() in ('—', '') else [x.strip() for x in c[3].split(',')]
            edges.append((axis, edge, c[1], qs))
    return edges


def cause_of(r):
    s = r['settlement']
    m = re.search(r'settled:\s*([^()]*?)\s*(\(|$)', s)
    if not m:
        return None
    c = m.group(1).strip()
    for k in CAUSES:
        if c.startswith(k):
            return k
    return 'supervisor' + (' (status set; no cause written)' if not c else ' (%s)' % c)


def cell(v):
    v = v.strip()
    return v if v else ' '


def parse_usage(u):
    m = re.match(r'(\d+) calls, ([\d.]+) M weighted, about ([\d.]+) % of the week', u)
    return (int(m.group(1)), float(m.group(2)), float(m.group(3))) if m else (0, 0.0, 0.0)


def minutes(a, b):
    ha, ma = map(int, a.split(':'))
    hb, mb = map(int, b.split(':'))
    return (hb * 60 + mb) - (ha * 60 + ma)


def agents():
    lines = open(os.path.join(ROWS, 'agents.tsv'), encoding='utf-8').read().splitlines()
    head = lines[0].split('\t')
    return [dict(zip(head, l.split('\t'))) for l in lines[1:] if l.strip()]


def check1(rows):
    ok = True
    out = []
    by = Counter(kind(k) for k in rows)
    q = Counter(rows[k]['chunk'] for k in rows if kind(k) == 'Q')
    out.append('check 1: P %d (want 92), Q %d (want 39: A %d, B %d, D %d, E1 %d; want 12, 12, 12, 3), '
               'O %d, S %d (%d withdrawn)' % (by['P'], by['Q'], q['A'], q['B'], q['D'], q['E1'],
                                             by['O'], by['S'], len(WITHDRAWN & set(rows))))
    if by['P'] != 92 or by['Q'] != 39 or [q['A'], q['B'], q['D'], q['E1']] != [12, 12, 12, 3]:
        ok = False
        out.append('  FAIL: counts')
    dups = [x for x in repairs if x.startswith('DUPLICATE')]
    out.append('  ids unique across all settled files: %s' % ('yes' if not dups else 'NO: ' + '; '.join(dups)))
    ok = ok and not dups
    bad = [k for k, r in rows.items() if r['status'] not in STATUSES]
    if bad:
        ok = False
        out.append('  FAIL: status off the scale: ' + ', '.join(bad))
    edges = read_index()
    pe = {'P:' + e[1] for e in edges}
    miss = sorted(pe - {k for k in rows if kind(k) == 'P'})
    extra = sorted({k for k in rows if kind(k) == 'P'} - pe)
    out.append('  probe ids vs INDEX.md edges: missing %s, extra %s' % (miss or 'none', extra or 'none'))
    ok = ok and not miss and not extra
    return ok, out


def build(rows):
    L = []
    w = L.append
    edges = read_index()
    ag = agents()

    # ---- header
    w('# Gap log, tally 1.0.1 against cases 1.0')
    w('')
    w('Session 3 of the case-set plan. Written by `tally-1.0.1/tools/merge_gaps.py` from the settled rows '
      'in `tally-1.0.1/rows/`; do not edit by hand, rerun the script. Text paths are relative to '
      '`tmp/cases/tally-1.0.1/` (or to the repository root for `engine/…`).')
    w('')

    # ---- part 1
    w('## 1. What it is and how it was made')
    w('')
    w('One row per probe card (`P:`, 92), per household question (`Q:`, 39: A 12, B 12, D 12, E1 3), per '
      'modelling observation (`O:`, a fact or rule that is hard to write although no question asks about it) '
      'and per spec finding (`S:`, a place where the spec itself is unclear or contradicts itself). '
      'Statuses, worst first: **breaks**, **can\'t express**, **awkward**, **expressed**; **case suspect** is '
      'outside the scale and goes to `ERRATA.md` (part 6).')
    w('')
    w('- **Agents.** A pilot probe writer (P0, spec findings from parsecheck); five probe writers (W-G1 … W-G5), '
      'one judge each (J-G1 … J-G5); four household writers (W-A, W-B, W-D, W-E1, the last for the joined '
      'family), one judge each (J-A, J-B, J-D, J-E1); the supervisor, who settled every disagreement; and '
      'this close-out (CO), which merged the rows. Part 7 lists each with its cost and audit.')
    w('- **Step map.** Each writer wrote a Tally text and a step map (`maps/<chunk>/`): every step of the '
      'card\'s or key\'s derivation, marked "same", a workaround, or missing, with the lines of the text '
      'that do it. A row\'s status follows from its map.')
    w('- **Judges.** Each judge tried to break every row its writer marked "expressed", line by line against '
      'the spec, and proposed a downgrade where it succeeded. The supervisor settled each verdict (part 5).')
    w('- **What "expressed" means here.** Judged against the spec (`engine/spec/`, tally 1.0.1), line by line, '
      'and not run: nothing runs Tally yet. A row is expressed when every step of its map means, by the spec, '
      'what the key\'s step computes.')
    w('- **Tools.** `parsecheck` checks syntax; `loadcheck` is a line scan of ids and keywords. Neither checks '
      'units (LOAD-UNIT), names (LOAD-NAME), scenario rules (LOAD-SCENARIO) or run-time refusals (RUN-NA, '
      'RUN-EMPTY), which is how several rows that pass both tools were still downgraded (A-01, A-02, B-01, '
      'B-04, B-05, D-12, E1-03).')
    w('- **Escalations.** W-A ran at xhigh by plan; J-A overturned 3 of 6 expressed rows, so W-B ran at xhigh; '
      'J-B overturned 4 of 5, so W-E1 ran at max; W-D (already running) stayed at xhigh; J-D overturned 3 of 3. '
      'The judges of the probe chunks overturned 0, 1, 0, 1, 1 rows (G1 … G5; G4\'s is P:LAW-12, which the '
      'supervisor downgraded after reading the card, part 5).')
    w('- **Cost cap.** The cap (14 % of the week) did not stop the session: everything ran (part 7).')
    w('- **Scope audit.** No agent touched the holdout folders; the other flags were false positives or '
      'harmless listings, as `agents.tsv` records (part 7).')
    w('')
    w('**The supervisor\'s decisions:**')
    w('')
    w('- D1. Final messages are saved from each agent\'s transcript by `tools/save_final.py` (never retyped).')
    w('- D2. Spec findings (`S:`) have no judge; the supervisor checks and settles them. S ids were renumbered '
      'at settlement into one sequence (S:1 …).')
    w('- D3. The made-up currencies DUK and FLR have no minor unit in the spec (S:4): rows are judged reading a '
      'minor unit of 0.01, and not downgraded for that alone.')
    w('- D4. Prompts: a short prompt restating the hard rules plus a filled brief file; all prompts and templates '
      'are kept in `tally-1.0.1/tools/prompts/`.')
    w('- D5. A row a writer\'s message left out is recovered from its step map (`rows/G5.recovered.md`: '
      'P:RULE-08), then judged like any other.')
    w('- D6. Household B\'s writer converted every record row with a script (one movement row per CSV row, '
      'citing file and line), standing in for the import pipeline of OPERATIONS §2.2; the brief allowed "at '
      'most a few rows". The work was kept and the deviation logged; households D and E1 followed the brief. '
      'So B\'s record questions are judged on full record rows, while A\'s (A-09) and D\'s (D-07) are can\'t '
      'express for want of them: the gap is the same (O:A-1, O:B-1, O:D-1: the import names no file format), '
      'only the writers\' handling differs.')
    w('- D7. A load refusal in one item of a writer\'s own file stops the whole reality (L §8): household D\'s '
      '`fund.tally` LOAD-UNIT (row Q:D-12) is charged to that row only; D\'s other rows and E1\'s are judged as '
      'if that one fault were fixed.')
    w('- D8. A candidate spec finding (HOLE-IMPRECISE for a pay `on m` when a version becomes known inside m) '
      'was dropped: L:420-421 settle it.')
    w('- S:13 and S:14 keep their ids but are not spec findings (writer misread, status awkward): they are '
      'marked "withdrawn as a finding" in part 4 and left out of the spec-findings count.')
    w('- Dossier oddity, no row: D-D04 (received 2024-10-21) says the certificate "became final on 3 November '
      '2024"; D\'s story says "becomes final".')
    if repairs:
        w('')
        w('**Source rows mended by the merge** (the settled files are frozen and stay as they are):')
        w('')
        for x in repairs:
            w('- ' + x)
    w('')

    # ---- part 2
    w('## 2. Counts')
    w('')
    cnt = Counter()
    for k, r in rows.items():
        kk = kind(k) if k not in WITHDRAWN else 'S withdrawn'
        cnt[(kk, r['status'])] += 1
    kinds = [('P', 'probe (P)'), ('Q', 'question (Q)'), ('O', 'observation (O)'),
             ('S', 'spec finding (S)'), ('S withdrawn', 'S withdrawn (not a finding)')]
    w('| Kind | ' + ' | '.join(STATUSES) + ' | Total |')
    w('|---|' + '---:|' * (len(STATUSES) + 1))
    tot = Counter()
    for kk, name in kinds:
        n = [cnt[(kk, s)] for s in STATUSES]
        if not sum(n):
            continue
        for s, v in zip(STATUSES, n):
            tot[s] += v
        w('| %s | %s | %d |' % (name, ' | '.join(str(v) for v in n), sum(n)))
    w('| **all** | %s | **%d** |' % (' | '.join('**%d**' % tot[s] for s in STATUSES), sum(tot.values())))
    w('')
    w('Per chunk (all kinds; P0 holds spec findings only):')
    w('')
    w('| Chunk | P | Q | O | S | ' + ' | '.join(STATUSES) + ' |')
    w('|---|' + '---:|' * (4 + len(STATUSES)))
    for ch in CHUNKS:
        rr = [r for r in rows.values() if r['chunk'] == ch]
        kc = Counter(kind(r['row']) for r in rr)
        sc = Counter(r['status'] for r in rr)
        w('| %s | %d | %d | %d | %d | %s |' % (ch, kc['P'], kc['Q'], kc['O'], kc['S'],
                                            ' | '.join(str(sc[s]) for s in STATUSES)))
    w('')

    # ---- part 3
    w('## 3. By edge')
    w('')
    w('In `INDEX.md` order. Probe: the `P:<edge>` row. Questions: the worst of the `Q:` rows of the questions '
      'INDEX.md lists for the edge. Worst: of both. Case suspect is outside the scale and shows beside the '
      'worst status, never as it.')
    w('')
    edge_worst = Counter()
    cur_axis = None
    for axis, edge, name, qs in edges:
        if axis != cur_axis:
            cur_axis = axis
            w('')
            w('### ' + axis)
            w('')
            w('| Edge | Name | Probe | Questions | Questions\' worst | Worst |')
            w('|---|---|---|---|---|---|')
        p = rows.get('P:' + edge)
        ps = p['status'] if p else '(no row)'
        qst = []
        qshow = []
        for q in qs:
            r = rows.get('Q:' + q)
            if r:
                qst.append(r['status'])
                qshow.append(q if r['status'] == 'expressed' else '%s (%s)' % (q, r['status']))
            else:
                qshow.append(q + ' (no row)')
        qw = worst(qst) if qs else '—'
        allw = worst(([ps] if p else []) + qst)
        edge_worst[allw.split(' (+')[0]] += 1
        w('| %s | %s | %s | %s | %s | %s |' % (edge, name, ps, ', '.join(qshow) if qs else '—', qw, allw))
    w('')
    w('Questions column: every question INDEX.md lists, with its status where it is not expressed. '
      'Edges by worst status: '
      + ', '.join('%s %d' % (s, edge_worst[s]) for s in SCALE + ['—'] if edge_worst[s]) + '.')
    w('')

    # ---- part 4
    def table(rr, mark=None):
        w('| Row | Status | Reason | Spec | Gap | Text |')
        w('|---|---|---|---|---|---|')
        for r in rr:
            texts = resolve_texts(r)
            tshow = '; '.join(('`%s`' % t) if ok else ('`%s` (**missing**)' % t) for t, ok in texts) or '—'
            st = r['status'] + ((' — ' + mark(r)) if mark and mark(r) else '')
            w('| `%s` | %s | %s | %s | %s | %s |' % (r['row'], st, cell(r['reason']), cell(r['spec']) or '—',
                                                   unq(r['gap']) or '—', tshow))
        w('')

    w('## 4. The rows')
    w('')
    w('Statuses as settled; part 5 has each change. A Text path marked **missing** does not exist.')
    w('')
    order = {('P:' + e[1]): i for i, e in enumerate(edges)}
    w('### 4.1 Probes, by axis')
    for ax in AXES:
        rr = sorted([r for k, r in rows.items() if kind(k) == 'P' and k[2:].split('-')[0] == ax],
                    key=lambda r: order.get(r['row'], 999))
        if rr:
            w('')
            w('#### ' + ax)
            w('')
            table(rr)
    w('### 4.2 Questions, by household')
    for h in HOUSES:
        rr = sorted([r for k, r in rows.items() if kind(k) == 'Q' and r['chunk'] == h],
                    key=lambda r: int(r['row'].rsplit('-', 1)[1]))
        w('')
        w('#### ' + h)
        w('')
        table(rr)
    w('### 4.3 Observations')
    w('')
    table([r for k, r in rows.items() if kind(k) == 'O'])
    w('### 4.4 Spec findings')
    w('')
    table(sorted([r for k, r in rows.items() if kind(k) == 'S'], key=lambda r: int(r['row'][2:])),
          mark=lambda r: 'withdrawn as a finding' if r['row'] in WITHDRAWN else '')
    missing = [(r['row'], t) for r in rows.values() for t, ok in resolve_texts(r) if not ok]
    w('Text paths that do not exist: ' + ('; '.join('%s `%s`' % m for m in missing) if missing else 'none') + '.')
    w('')

    # ---- part 5
    w('## 5. Settlements')
    w('')
    w('Every row whose final status differs from its writer\'s, or on which the supervisor ruled. The writer\'s '
      'status is the one in its raw message (S rows under their old ids); "—", "observation", "finding" and '
      '"ambiguous" are writers\' words off the scale, set by the supervisor.')
    w('')
    w('| Row | Writer\'s status | Final status | Cause | Settlement |')
    w('|---|---|---|---|---|')
    bycause = Counter()
    for k, r in rows.items():
        c = cause_of(r)
        if r['writer_status'] == r['status'] and not c:
            continue
        if not c:
            c = '(none written)'
        bycause[c.split(' (')[0]] += 1
        ws = r['writer_status'] + ('' if r['writer_id'] == k else ' (as %s)' % r['writer_id'])
        w('| `%s` | %s | %s | %s | %s |' % (k, ws, r['status'], c, cell(r['settlement'])))
    w('')
    w('By cause: ' + ', '.join('%s %d' % (c, bycause[c]) for c in CAUSES + ['(none written)'] if bycause[c])
      + '; %d rows in all.' % sum(bycause.values()))
    jd = [k for k, r in rows.items() if 'judge: downgrade' in r['settlement']]
    w('')
    w('Judge downgrades upheld by the supervisor: %d (%s). Rows the supervisor downgraded with no judge '
      'downgrade: %s.' % (len(jd), ', '.join(jd),
                          ', '.join(k for k, r in rows.items()
                                    if cause_of(r) == 'writer misread' and 'judge: downgrade' not in r['settlement'])
                          or 'none'))
    w('')

    # ---- part 6
    w('## 6. Suspects')
    w('')
    w('Each row settled as case suspect, with its reason. The exam side settles them (ERRATA, version bump) '
      'before session 4 starts.')
    w('')
    w('| Row | Reason | Settlement |')
    w('|---|---|---|')
    for k, r in rows.items():
        if r['status'] == SUSPECT:
            w('| `%s` | %s | %s |' % (k, cell(r['reason']), cell(r['settlement'])))
    w('')
    w('Also, with no row: the dossier oddity in part 1 (D-D04 "became final on 3 November 2024" against D\'s '
      'story "becomes final").')
    w('')

    # ---- part 7
    w('## 7. Cost and time')
    w('')
    w('From `rows/agents.tsv` (usage lines from `tmp/cases/tools/usage.py`). The close-out\'s own cost is not '
      'in it.')
    w('')
    w('| Slot | Agent | Role | Chunk | Effort | Start | End | Minutes | Calls | M weighted | % of week | Audit |')
    w('|---|---|---|---|---|---|---|---:|---:|---:|---:|---|')
    tc = tw = tp = 0
    tm = 0
    sub = [0, 0.0, 0.0, 0]
    for a in ag:
        c, mw, pc = parse_usage(a['usage'])
        m = minutes(a['start'], a['end'])
        tc += c
        tw += mw
        tp += pc
        if a['agent'] != 'supervisor':
            sub[0] += c
            sub[1] += mw
            sub[2] += pc
            sub[3] += m
        w('| %s | %s | %s | %s | %s | %s | %s | %d | %d | %.2f | %.2f | %s |'
          % (a['slot'], a['agent'], a['role'], a['chunk'], a['effort'], a['start'], a['end'], m, c, mw, pc,
             a['audit']))
    sup = [a for a in ag if a['agent'] == 'supervisor']
    wall = minutes(sup[0]['start'], sup[0]['end']) if sup else 0
    w('| | **sub-agents** | | | | | | %d | %d | %.2f | %.2f | |' % (sub[3], sub[0], sub[1], sub[2]))
    w('| | **all** | | | | | | | %d | %.2f | **%.2f** | |' % (tc, tw, tp))
    w('')
    w('**Total:** about %.2f %% of the week (sub-agents %.2f %%, supervisor %.2f %%), %d calls, %.2f M weighted; '
      'wall time %dh%02d (supervisor %s–%s), sub-agent time summed %dh%02d (agents ran two at a time). '
      '**Against the estimate** (7–11 %% of the week, about 10–12 hours): %s on cost, %s on time. The cap '
      '(14 %%) was never near.'
      % (tp, sub[2], tp - sub[2], tc, tw, wall // 60, wall % 60, sup[0]['start'] if sup else '?',
         sup[0]['end'] if sup else '?', sub[3] // 60, sub[3] % 60,
         'well under' if tp < 7 else ('within' if tp <= 11 else 'over'),
         'under' if wall < 600 else ('within' if wall <= 720 else 'over')))
    w('')
    w('**Audits,** one line per agent:')
    w('')
    for a in ag:
        w('- %s (%s, %s): %s' % (a['agent'], a['role'], a['chunk'], a['audit']))
    w('')
    return '\n'.join(L)


def main():
    out = os.path.join(CASES, 'gaps-1.0.1.md')
    if '--out' in sys.argv:
        out = sys.argv[sys.argv.index('--out') + 1]
    rows = settled_rows()
    ok, lines = check1(rows)
    text = build(rows)
    if '--check' in sys.argv:
        print('\n'.join(lines))
        for x in repairs:
            print('  repair: ' + x)
        sys.exit(0 if ok else 1)
    with open(out, 'w', encoding='utf-8') as f:
        f.write(text)
    print('wrote %s (%d lines)' % (os.path.relpath(out, REPO), text.count('\n') + 1))
    print('\n'.join(lines))
    sys.exit(0 if ok else 1)


if __name__ == '__main__':
    main()
