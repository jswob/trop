"""Part 3.1 of the 4.5f brief: one canonical visible copy. Standard library only; run once from the repository root.

1. The judged texts no check reads, still untracked under tmp/design2/cases/, join the suite's copy
   (engine/tests/integration/households, law): plain moves, bytes unchanged; nothing is overwritten.
2. The six files of TC-1..TC-4 and 1.0.2 take session 5's visible bytes (tmp/session5/visible/cases/), after a check
   that the suite's file is still the judged one; TIME-08 (TC-5) joins unit/probes/time/.
3. Every cut-down copy holding the same lines: a copy byte-equal to the judged file takes the visible file; a copy that
   differs in its knob lines (the B-12 path and run-end cuts) gets TC-2's one change, `windows each` before the paths
   clause of the five ask lines, by pattern; anything else stops the script.
4. Every copy session 5 also made (tmp/session5/visible/mc-*, w-e1) must now equal session 5's bytes."""
import filecmp, os, re, shutil, sys
R = '/Users/jswob/projects/trop/'
OLD, V, I = R + 'tmp/design2/cases/', R + 'tmp/session5/visible/', R + 'engine/tests/integration/'
done = []
# 1
for sub in ('households', 'law'):
    for d, _, fs in os.walk(OLD + sub):
        for f in fs:
            if f == '.DS_Store' or '__pycache__' in d:
                continue
            src = os.path.join(d, f)
            dst = I + sub + src[len(OLD + sub):]
            assert not os.path.exists(dst), dst
            os.makedirs(os.path.dirname(dst), exist_ok=True)
            shutil.move(src, dst)
            done.append(('joined', dst))
# 2, 3
SIX = ['households/B/flat.tally', 'households/B/scenarios/no_overpay.tally', 'households/D/ogrodowa.tally',
       'households/E1/asks/E1-03.tally', 'households/E1/family.tally', 'households/E1/scenarios/e1_not_reported.tally']
COPIES = {'households/B/flat.tally': ['cuts/B/b12_11/flat.tally', 'cuts/B/cd7f11/flat.tally', 'cuts/B/cs11/flat.tally',
                                      'split/k3-b/flat.tally', 'split/k4-b07/flat.tally', 'split/k5-b12/flat.tally'],
          'households/B/scenarios/no_overpay.tally': ['cuts/B/b12_11/scenarios/no_overpay.tally',
                                                      'split/k3-b/scenarios/no_overpay.tally',
                                                      'split/k4-b07/scenarios/no_overpay.tally',
                                                      'split/k5-b12/scenarios/no_overpay.tally'],
          'households/D/ogrodowa.tally': ['cuts/D/cut11/ogrodowa.tally', 'cuts/D/p200_11/ogrodowa.tally',
                                          'cuts/E1/cdB/households/D/ogrodowa.tally'],
          'households/E1/asks/E1-03.tally': ['cuts/E1/cdB/households/E1/asks/E1-03.tally'],
          'households/E1/scenarios/e1_not_reported.tally': ['cuts/E1/cdB/households/E1/scenarios/e1_not_reported.tally']}
TC2 = re.compile(r'^(ask b12_(?:share|share_without|shortfall|shortfall_without|mortgage): .* as known 2026-01-02 )((?:with uncertain )?paths )', re.M)
for f in SIX:
    judged = open(I + f, encoding='utf-8').read()
    for c in COPIES.get(f, []):
        cur = open(I + c, encoding='utf-8').read()
        if cur == judged:
            shutil.copyfile(V + 'cases/' + f, I + c)
            done.append(('visible bytes', I + c))
        elif f.endswith('no_overpay.tally'):
            new, n = TC2.subn(r'\1windows each \2', cur)
            assert n == 5, (c, n)
            open(I + c, 'w', encoding='utf-8').write(new)
            done.append(('TC-2 by pattern', I + c))
        else:
            sys.exit('stop: %s differs from the judged %s in more than its knob lines' % (c, f))
    shutil.copyfile(V + 'cases/' + f, I + f)
    done.append(('visible bytes', I + f))
t8 = R + 'engine/tests/unit/probes/time/TIME-08-before.tally'
assert not os.path.exists(t8)
shutil.copyfile(V + 'cases/probes/time/TIME-08-before.tally', t8)
done.append(('TC-5', t8))
# 4
W = {'mc-b/b12_11/flat.tally': 'cuts/B/b12_11/flat.tally', 'mc-b/b12_11/scenarios/no_overpay.tally': 'cuts/B/b12_11/scenarios/no_overpay.tally',
     'mc-b/cs11/flat.tally': 'cuts/B/cs11/flat.tally', 'mc-d/cut11/ogrodowa.tally': 'cuts/D/cut11/ogrodowa.tally',
     'mc-d/p200_11/ogrodowa.tally': 'cuts/D/p200_11/ogrodowa.tally', 'w-e1/cdB/households/D/ogrodowa.tally': 'cuts/E1/cdB/households/D/ogrodowa.tally',
     'w-e1/cdB/households/E1/asks/E1-03.tally': 'cuts/E1/cdB/households/E1/asks/E1-03.tally',
     'w-e1/cdB/households/E1/scenarios/e1_not_reported.tally': 'cuts/E1/cdB/households/E1/scenarios/e1_not_reported.tally'}
bad = [k for k, v in W.items() if not filecmp.cmp(V + k, I + v, shallow=False)]
for d, _, fs in os.walk(V + 'cases'):
    for f in fs:
        p = os.path.join(d, f)
        q = (I if '/probes/' not in p else R + 'engine/tests/unit/') + os.path.relpath(p, V + 'cases')
        if not os.path.exists(q) or not filecmp.cmp(p, q, shallow=False):
            bad.append('cases/' + os.path.relpath(p, V + 'cases'))
print('%d joined, %d set to the visible bytes, %d by pattern, TIME-08 added' % (
    sum(1 for a, _ in done if a == 'joined'), sum(1 for a, _ in done if a == 'visible bytes'),
    sum(1 for a, _ in done if a == 'TC-2 by pattern')))
print('session 5\'s copies: %s' % ('every one equal' if not bad else 'DIFFER: %s' % bad[:10]))
