"""refcore: a reference interpreter for Tally 2, the core (writer 1).

Built from LANGUAGE.md and ENGINE.md alone, as a spec-checking artefact.
Where the spec is silent it refuses with REFCORE-UNSPECIFIED; constructs left
to later writers refuse with REFCORE-NOT-YET.  See HANDOVER.md.
"""

import sys as _sys

# an exact value may print with more decimal places than Python's default limit on converting integers to text
# (4 300 digits); values themselves are capped at 4 096 digits where they are made (L §7.1, values.Num)
if hasattr(_sys, 'set_int_max_str_digits'):
    _sys.set_int_max_str_digits(0)
