"""codes/m1_fault.py: the frozen refcore with ENGINE.md §9 invariant 1's planted fault, the twin of the engine's
`run_faulted` (the coordinator's pick P8; INTERFACE.md §4). Standard library only; it changes no refcore file.

The plant: on the first day a walk books a leg whole (its day's conservation sum is not empty), the day's close adds one
minor unit of the fixture's currency (0.01) to that day's sum before invariant 1 is checked, once per run. refcore then
refuses as it would on an engine bug, `ENGINE-INVARIANT conservation: the legs booked on <day> in <unit> sum to <x>,
not 0 (E §9 invariant 1)` (walk.py's close_day), and codes/m1.py stores that whole output as the ENGINE-INVARIANT
fixture's expected bytes.

    <pypy> engine/tests/codes/m1_fault.py run <files> --ask <asks>    (run by codes/m1.py, never by hand)
"""
import os
import sys
from fractions import Fraction

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))) + os.sep
sys.path.insert(0, ROOT + 'engine/tools')
from refcore import walk as W  # noqa: E402
from refcore.cli import main  # noqa: E402

MINOR = Fraction(1, 100)
_close = W.Walk.close_day
_planted = []


def close_day(self, d):
    if self.cons and not self.proj and not _planted:
        u = sorted(self.cons, key=str)[0]
        self.cons[u] = self.cons[u] + MINOR
        _planted.append((d, u))
    return _close(self, d)


W.Walk.close_day = close_day

if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
