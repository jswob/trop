# Probes

One card per visible edge of `../boundaries.md`, in `<axis>/<EDGE>.md`: a few lines of story, the rules it needs, one question, the answer worked by hand, and a YAML block. Each card is self-contained: its rules and figures are the card's own, not the home law's (`../law/`), and they are made up.

**Currencies:** DUK (the dukat, the home currency) and FLR (the floren, the foreign one); amounts are written with a space between thousands and a point before the minor units.

**The YAML block** has one top-level `name: value` per entry, each value in flow style (JSON), so the standard library reads it: `edge`, `inputs` (what the key script reads), `expect` (the hand-worked answer, as a key). A card with several parts keys each part separately (`a`, `b`).

**Keys:** `python3 -I tmp/cases/tools/keys_probes.py` computes every card's key with the key library (`../tools/lib/`), compares it with `expect`, and writes `../keys/probes/<EDGE>.md`: the key, then its derivation. Key kinds: value, partial (a value and what it leaves out), refusal (what's missing), by-case (one value per reading or branch, with the date that settles it), relation (a statement checked on the answer), random (a reference `r`, its standard error `se_r`, a target `p`; an answer passes if its own error is at most `p` and it lies within 4 combined standard errors of `r`).

**SURF cards** (`surf/`) are passages to write, with no YAML block and no key.

**Pair cards** (`pairs/`, from cases 1.1) test two edges where they meet, beside each edge's own card. The convention:
- **file:** `pairs/<EDGE>+<EDGE>.md`, the two edge ids in the order of `../boundaries.md` (for now `pairs/CUR-01+RAND-01.md`: a drawn exchange rate on the law's leg);
- **its `edge:` line** names both edges, as a list: `edge: ["CUR-01", "RAND-01"]`;
- **its key** goes to `../keys/probes/<EDGE>+<EDGE>.md`, headed by the pair's name; `keys_probes.py` computes it with the function `CUR_01__RAND_01` of `../tools/probefns/pairs.py` (`-` written `_`, `+` written `__`);
- a pair is not an edge: `boundaries.md` doesn't list it, and it counts toward neither edge's probe. `check.py` requires that each pair card name two visible edges and have its key; `INDEX.md` lists it beside both edges' cards.

**LAW-20** (from cases 1.1) is the one keyed edge that no visible question uses: its `where:` says no visible household meets a named solver, and the 79 cases' models (UL-07, UL-08) carry it in integration. `check.py` exempts an edge only on that `where:` wording, and names it.
