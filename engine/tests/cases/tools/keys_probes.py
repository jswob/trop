#!/usr/bin/env python3
"""Compute every probe's key with the key library, compare it with the card's hand-worked answer,
and write keys/probes/<edge>.md. Exit 1 if any card and its key disagree.

A pair card (probes/pairs/<EDGE>+<EDGE>.md, probes/README.md) is keyed the same way: its function in
probefns/pairs.py is named with `_` for `-` and `__` for `+`, and its key is headed by the pair's name.

Run: python3 -I tmp/cases/tools/keys_probes.py
"""
import importlib
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))

from lib.keys import Trace, diff, load_block, refusal, write  # noqa: E402
from lib.num import Refuse  # noqa: E402


def main():
    ok = bad = passages = 0
    for card in sorted((ROOT / "probes").glob("*/*.md")):
        edge = card.stem
        y = load_block(card.read_text(encoding="utf-8"))
        if y is None or "expect" not in y:
            passages += 1
            continue
        mod = importlib.import_module(f"probefns.{card.parent.name}")
        t = Trace()
        try:
            key = getattr(mod, edge.replace("-", "_").replace("+", "__"))(y["inputs"], t)
        except Refuse as e:
            key = refusal(e)
        extra = {k: y[k] for k in ("as_of", "known_as_of") if k in y}
        write(ROOT / "keys" / "probes" / f"{edge}.md", edge, key, t, extra)
        d = diff(y["expect"], key)
        if d:
            bad += 1
            print(f"FAIL {edge}")
            for line in d:
                print(f"     {line}")
        else:
            ok += 1
    print(f"{ok} probes agree, {bad} disagree, {passages} passages without a key")
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
