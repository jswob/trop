"""refcore_fast: the frozen refcore with the memory layer in front (layer.py).

    python -I -B engine/tools/refcore_fast [--verify N] [--plant] <refcore's argv>

--verify N recomputes one in N remembered answers the slow way and stops (exit 70) on any difference; --plant leaves
the read's day out of the read memo's key, the omission the self-test must catch. The switch back is running refcore.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from refcore.cli import main  # noqa: E402
from refcore_fast.layer import install  # noqa: E402

if __name__ == '__main__':
    argv = sys.argv[1:]
    verify, plant = 0, False
    while argv and argv[0] in ('--verify', '--plant'):
        if argv[0] == '--verify':
            verify = int(argv[1])
            argv = argv[2:]
        else:
            plant = True
            argv = argv[1:]
    install(verify, plant)
    sys.exit(main(argv))
