"""Seeds a fuzz target's corpus with the texts M0's tests name (R-91, R-92).

    python3 -I -B engine/fuzz/seeds.py TARGET...

The seeds are every text `tests/runner/src/m0_texts.rs` and `m0_spec_texts.rs` embed: the
corpus's realities, the format fixtures under `tests/m0/format/`, and the spec's realities
that row 3 of `tests/MILESTONE-M0.md` loads. The lists are read, never copied, so a text T
adds is a seed on the next run. Each seed is written to `fuzz/corpus/TARGET/` under the
SHA-1 of its bytes, libFuzzer's own naming, so a second run writes nothing and a seed the
fuzzer already holds isn't doubled. The corpus folder is local (`fuzz/.gitignore`); `check`'s
fuzz step and the long fuzz job run this before each target.
"""

import hashlib
import os
import re
import sys

FUZZ = os.path.dirname(os.path.realpath(__file__))
RUNNER_SRC = os.path.join(os.path.dirname(FUZZ), "tests", "runner", "src")
LISTS = ["m0_texts.rs", "m0_spec_texts.rs"]
INCLUDE = re.compile(r'include_str!\("([^"]+)"\)')


def texts():
    """The paths each list embeds, resolved, each once, sorted."""
    paths = set()
    for name in LISTS:
        with open(os.path.join(RUNNER_SRC, name), encoding="utf-8") as f:
            found = INCLUDE.findall(f.read())
        if not found:
            sys.exit(f"seeds: {name} embeds no text")
        paths.update(os.path.normpath(os.path.join(RUNNER_SRC, p)) for p in found)
    return sorted(paths)


def seed(target, paths):
    corpus = os.path.join(FUZZ, "corpus", target)
    os.makedirs(corpus, exist_ok=True)
    written = 0
    for path in paths:
        with open(path, "rb") as f:
            data = f.read()
        out = os.path.join(corpus, hashlib.sha1(data).hexdigest())
        if not os.path.exists(out):
            with open(out, "wb") as f:
                f.write(data)
            written += 1
    return written


def main(targets):
    if not targets:
        sys.exit(__doc__)
    paths = texts()
    for target in targets:
        written = seed(target, paths)
        print(f"seeds: {target}: {len(paths)} texts, {written} new in fuzz/corpus/{target}/")


if __name__ == "__main__":
    main(sys.argv[1:])
