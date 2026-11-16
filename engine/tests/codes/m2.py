"""codes/m2.py: M2's fixtures for the codes and message forms of M2's constructs that no output at S, M0, M1 or M2
prints (DONE gate 5; BUILD-ORDER § M2; tools/xref_check.py --gate M2; issue #167, MILESTONE-M2.md row 4). Standard
library only.

codes/m1.py with the store m2: the families are the folders of codes/m2/, the fixtures the rows of codes/m2/edits.tsv
in m0's columns, the store codes/m2.refcore in m0's seven netstrings, each family's base and the answers of its asks
stored beside its fixtures, all as codes/m1.py runs them on the frozen refcore. What M2 adds:

- **A `HOLE-SAME` fixture prints `WARN-LEGS`.** Two rows stated `same` that disagree print the warning and each end is
  booked as its own statement says (E §6, "two rows stated same that disagree print WARN-LEGS"); the hole is the read
  of the movement away from its places. So a `HOLE-SAME` fixture may print that one warning; any other is refused.
- **No planted fault.** M1's `ENGINE-INVARIANT` store isn't M2's.

codes/m2/held.tsv holds, in m0's held columns, the fixtures refcore can't print, each with the issue its ruling is
on (#169: refcore 2.1.7's misses, fixed at 2.1.8); this script doesn't read it.

    python3 -I -B engine/tests/codes/m2.py              -> writes codes/m2.refcore
    python3 -I -B engine/tests/codes/m2.py --check      -> exit 1 if codes/m2.refcore isn't what refcore's runs give
    python3 -I -B engine/tests/codes/m2.py --show GLOB  -> refcore's run of the vectors whose id matches; writes nothing
"""
import importlib.util
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
_spec = importlib.util.spec_from_file_location('codes_m1', os.path.join(HERE, 'm1.py'))
M1 = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(M1)
M = M1.M
M.configure('m2')
LEGS = re.compile(r'^warning: WARN-LEGS .*\n?', re.M)
_failures = M.failures


def failures(vs, res):
    """codes/m1.py's checks, a HOLE-SAME fixture's one WARN-LEGS line set aside (E §6)."""
    seen = [dict(e, out=LEGS.sub('', e['out'], count=1)) if v[1] == 'HOLE-SAME' else e for v, e in zip(vs, res)]
    return _failures(vs, seen)


M.failures = failures


if __name__ == '__main__':
    sys.exit(M.main(sys.argv[1:]))
