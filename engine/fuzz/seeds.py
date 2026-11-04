"""Seeds a fuzz target's corpus with the texts M0's tests name (R-91, R-92).

    python3 -I -B engine/fuzz/seeds.py TARGET...
    python3 -I -B engine/fuzz/seeds.py TARGET --into DIR   -> the seeds into DIR instead

The seeds are every text `tests/runner/src/m0_texts.rs` and `m0_spec_texts.rs` embed: the
corpus's realities, the format fixtures under `tests/m0/format/`, and the spec's realities
that row 3 of `tests/MILESTONE-M0.md` loads. The `load` target reads its input as up to three
files split at the byte 0xFF, so it also gets each reality of the load vectors
(`tests/m0/load.refcore`, `loadcheck.refcore`): its file table's first three files, joined by
0xFF. The lists are read, never copied, so a text T adds is a seed on the next run. Each seed
is written to `fuzz/corpus/TARGET/` under the SHA-1 of its bytes, libFuzzer's own naming, so a
second run writes nothing and a seed the fuzzer already holds isn't doubled. The corpus folder
is local (`fuzz/.gitignore`), and the long fuzz job runs this before each target. `check`'s
fuzz step seeds a fresh folder `--into` target/ instead, on every run, so it fuzzes what CI
fuzzes and a corpus grown by long runs can't change it (issue #78).
"""

import hashlib
import os
import re
import sys

FUZZ = os.path.dirname(os.path.realpath(__file__))
TESTS = os.path.join(os.path.dirname(FUZZ), "tests")
RUNNER_SRC = os.path.join(TESTS, "runner", "src")
LISTS = ["m0_texts.rs", "m0_spec_texts.rs"]
INCLUDE = re.compile(r'include_str!\("([^"]+)"\)')
# the load vectors: seven netstrings each, the fifth its file table (`m0/vectors.py`)
VECTORS = ["load.refcore", "loadcheck.refcore"]
FIELDS, TABLE = 7, 4
SEPARATOR = b"\xff"  # the load target's file separator, a byte UTF-8 never holds
FILES = 3  # the most files the load target splits its input into


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


def netstrings(data):
    """A file of netstrings (`<length>:<bytes>,`), newlines between them skipped."""
    fields, i = [], 0
    while i < len(data):
        if data[i:i + 1] == b"\n":
            i += 1
            continue
        colon = data.index(b":", i)
        n = int(data[i:colon])
        end = colon + 1 + n
        if data[end:end + 1] != b",":
            sys.exit("seeds: a load vector file isn't netstrings")
        fields.append(data[colon + 1:end])
        i = end + 1
    return fields


def tables():
    """Each load vector's files, as paths, at most FILES of them, each table once."""
    found = set()
    for name in VECTORS:
        with open(os.path.join(TESTS, "m0", name), "rb") as f:
            fields = netstrings(f.read())
        if not fields or len(fields) % FIELDS:
            sys.exit(f"seeds: m0/{name} doesn't hold {FIELDS} netstrings per vector")
        for k in range(0, len(fields), FIELDS):
            rows = [line.split("\t") for line in fields[k + TABLE].decode("utf-8").splitlines() if line]
            paths = tuple(os.path.normpath(os.path.join(TESTS, row[1])) for row in rows[:FILES] if len(row) == 2)
            if paths:
                found.add(paths)
    return sorted(found)


def read(path):
    with open(path, "rb") as f:
        return f.read()


def seed(corpus, inputs):
    os.makedirs(corpus, exist_ok=True)
    written = 0
    for data in inputs:
        out = os.path.join(corpus, hashlib.sha1(data).hexdigest())
        if not os.path.exists(out):
            with open(out, "wb") as f:
                f.write(data)
            written += 1
    return written


def main(args):
    into = None
    if len(args) == 3 and args[1] == "--into":
        args, into = args[:1], args[2]
    if not args or "--into" in args:
        sys.exit(__doc__)
    paths = texts()
    for target in args:
        inputs = [read(p) for p in paths]
        joined = 0
        if target == "load":
            for table in tables():
                inputs.append(SEPARATOR.join(read(p) for p in table))
                joined += 1
        corpus = into or os.path.join(FUZZ, "corpus", target)
        written = seed(corpus, inputs)
        more = f" and {joined} realities of the load vectors" if joined else ""
        print(f"seeds: {target}: {len(paths)} texts{more}, {written} new in {corpus}")


if __name__ == "__main__":
    main(sys.argv[1:])
