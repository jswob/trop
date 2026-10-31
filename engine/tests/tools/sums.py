"""TESTS.sha256: the checksums of the suite, which check-frozen verifies (T1, issue #5). Standard library only.

Lists every file under engine/tests/ as `<sha256>  <path>`, sorted, except TESTS.sha256 itself and the run output
check-frozen skips (cache/, out/, results/, and its SKIP_DIRS such as __pycache__ and target/). The file list and the
skips are check-frozen's own (engine/tools/check_frozen.py, imported read-only), so the two can't disagree.
T writes it last, after every other change to the suite, in the same commit (DONE rule 2: the guard and check-frozen
then hold the tests to it).

    python3 -I -B engine/tests/tools/sums.py            -> writes engine/tests/TESTS.sha256
    python3 -I -B engine/tests/tools/sums.py --check    -> what check-frozen would say, one line per problem
"""
import os
import sys

TESTS = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(TESTS), 'tools'))
import check_frozen as F  # noqa: E402

NAME = 'TESTS.sha256'


def text():
    names = [n for n in F.files_under(TESTS) if n != NAME and not n.startswith(F.TESTS_OUTPUT)]
    return ''.join('%s  %s\n' % (F.sha256(os.path.join(TESTS, n)), n) for n in names)


def main(argv):
    if '--check' in argv:
        problems = F.verify_sums(TESTS, NAME, 'tests', skip=F.TESTS_OUTPUT)
        for p in problems:
            print(p)
        print('%s: %d problems' % (NAME, len(problems)))
        return 1 if problems else 0
    t = text()
    with open(os.path.join(TESTS, NAME), 'w', encoding='utf-8') as f:
        f.write(t)
    print('wrote %s: %d files' % (NAME, t.count('\n')))
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
