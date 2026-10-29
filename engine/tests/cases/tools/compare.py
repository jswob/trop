#!/usr/bin/env python3
"""Mark answers against the keys: pass, fail, read or not run, per question.

An answers file is Markdown with one fenced YAML block per answer, in the keys' own flow form:

    ```yaml
    question: A-01
    answer: {"2022": {"joint tax": 20435, ...}}
    without: {...}            # optional: the answer without the twist, where the key has one
    ```

`without:` is the grading side's proof that a twist moves 3 or more keys: no question asks for it, it
isn't marked against a reader unless the reader gives one, and it may rest on facts the dossier doesn't hold.

Numbers are exact decimals (400.00 equals 400.0); a date is YYYY-MM-DD; any other string is prose.

How each kind is marked (brief §6):
- value: the same shape, the same labels (case and spacing ignored), every number and date equal;
- by-case: the key's cases {"kind": "by-case", "cases": {...}}; each key case must be matched by an
  answer case with an equal value (the case labels are prose, so they aren't matched);
- partial: the value as above, and as many named holes as the key lists;
- refusal: the answer is a refusal too, with no value;
- random: {"kind": "random", "r": .., "se_r": .., "p": ..}; the answer gives {"value": .., "se": ..}
  (with or without "kind": "random");
  it passes if se ≤ p and |value − r| ≤ 4 × √(se² + se_r²). With --small (a run cut in paths, marked against a
  key made for it, such as keys-small/), the precision bar se ≤ p is dropped: it passes if |value − r| ≤ 4 × √(se² + se_r²).
Prose (holes, reasons, causes' wording) is never marked by machine: it is listed as **read**, and the
question passes only once those lines are read and agreed (holes are judged both ways).

Run: python3 -I tmp/cases/tools/compare.py <answers.md> [--keys A] [--self-test A] [--keys-root <dir>] [--small]
(--keys-root: the folder holding the keys' house folders, instead of tmp/cases/keys, e.g. tmp/cases/keys-small;
--small: random answers without the precision bar, as above)
"""
import json
import pathlib
import re
import sys
from decimal import Decimal

ROOT = pathlib.Path(__file__).resolve().parents[1]
DATE = re.compile(r"^\d{4}-\d{2}-\d{2}$")
KEYS_ROOT = ROOT / "keys"
SMALL = False  # --small: a run cut in paths; random answers are marked without the precision bar se ≤ p
BLOCK = re.compile(r"```yaml\n(.*?)\n```", re.S)


def parse(text):
    return json.loads(text, parse_float=Decimal, parse_int=Decimal)


def blocks(text):
    """Each YAML block as {field: value}; one field per line, the value in flow form."""
    out = []
    for b in BLOCK.findall(text):
        fields = {}
        for ln in b.splitlines():
            k, _, v = ln.partition(": ")
            v = v.strip()
            try:
                fields[k.strip()] = parse(v)
            except json.JSONDecodeError:
                fields[k.strip()] = v
        out.append(fields)
    return out


def keys(house):
    out = {}
    for p in sorted((KEYS_ROOT / house).glob("*.md")):
        b = blocks(p.read_text(encoding="utf-8"))[0]
        out[b["question"]] = b
    return out


def norm(label):
    return " ".join(str(label).casefold().split())


class Mark:
    def __init__(self):
        self.diffs, self.reads = [], []

    def diff(self, where, msg):
        self.diffs.append(f"{where}: {msg}")

    def read(self, where, key, ans):
        self.reads.append(f"{where}: key says {json.dumps(key, default=str, ensure_ascii=False)}; answer says {json.dumps(ans, default=str, ensure_ascii=False)}")


def is_prose(x):
    return isinstance(x, str) and not DATE.match(x)


def same(key, ans, where, m):
    """Compare a key value with an answer value, recording every difference."""
    if isinstance(key, dict) and key.get("kind") in ("by-case", "partial", "refusal", "random"):
        return kinded(key, ans, where, m)
    if isinstance(key, dict):
        if not isinstance(ans, dict):
            return m.diff(where, f"expected a table, got {ans!r}")
        amap = {norm(k): v for k, v in ans.items()}
        for k, v in key.items():
            if norm(k) not in amap:
                if is_prose(v) or (isinstance(v, list) and all(is_prose(x) for x in v)):
                    m.read(f"{where}/{k}", v, None)
                else:
                    m.diff(where, f"missing {k!r}")
                continue
            same(v, amap[norm(k)], f"{where}/{k}", m)
        extra = set(amap) - {norm(k) for k in key}
        if extra:
            m.diff(where, f"labels the key doesn't have: {sorted(extra)}")
        return
    if isinstance(key, list):
        if not isinstance(ans, list) or len(ans) != len(key):
            if key and all(is_prose(x) for x in key):
                return m.read(where, key, ans)
            return m.diff(where, f"expected {len(key)} items, got {ans!r}")
        for i, (k, a) in enumerate(zip(key, ans)):
            same(k, a, f"{where}[{i}]", m)
        return
    if isinstance(key, Decimal):
        if not isinstance(ans, Decimal) or ans != key:
            m.diff(where, f"key {key}, answer {ans}")
        return
    if is_prose(key):
        return m.read(where, key, ans)
    if key != ans:
        m.diff(where, f"key {key!r}, answer {ans!r}")


def kinded(key, ans, where, m):
    kind = key["kind"]
    if kind == "random" and isinstance(ans, dict) and "kind" not in ans:
        pass  # the documented {value, se} form, with no kind
    elif not isinstance(ans, dict) or ans.get("kind") != kind:
        return m.diff(where, f"the key is a {kind}, the answer isn't")
    if kind == "refusal":
        if "value" in ans:
            m.diff(where, "a refusal gives no value")
        m.read(f"{where}/missing", key.get("missing"), ans.get("missing"))
    elif kind == "partial":
        same(key["value"], ans.get("value"), f"{where}/value", m)
        if len(ans.get("missing") or []) != len(key["missing"]):
            m.diff(where, f"names {len(ans.get('missing') or [])} holes, the key {len(key['missing'])}")
        m.read(f"{where}/missing", key["missing"], ans.get("missing"))
    elif kind == "by-case":
        acases = list((ans.get("cases") or {}).items())
        if len(acases) != len(key["cases"]):
            m.diff(where, f"{len(acases)} cases, the key has {len(key['cases'])}")
        used = set()
        for label, kv in key["cases"].items():
            hit = None
            for i, (al, av) in enumerate(acases):
                if i in used:
                    continue
                t = Mark()
                same(kv, av, "", t)
                if not t.diffs:
                    hit = i
                    m.reads += [f"{where}/{label}{r}" for r in t.reads]
                    m.read(f"{where}/case label", label, al)
                    break
            if hit is None:
                m.diff(where, f"no answer case equals the key's case {label!r}")
            else:
                used.add(hit)
        for f in ("missing", "settled by"):
            if f in key:
                m.read(f"{where}/{f}", key[f], ans.get(f))
    elif kind == "random":
        v, se = ans.get("value"), ans.get("se")
        if not isinstance(v, Decimal) or not isinstance(se, Decimal):
            return m.diff(where, "a random answer gives value and se")
        if se > key["p"] and not SMALL:
            m.diff(where, f"se {se} above the target {key['p']}")
        if (v - key["r"]) ** 2 > 16 * (se ** 2 + key["se_r"] ** 2):
            m.diff(where, f"{v} is more than 4 combined standard errors from {key['r']}")


def mark(house, answers):
    out = {}
    for qid, k in keys(house).items():
        a = answers.get(qid)
        if a is None:
            out[qid] = ("not run", [], [])
            continue
        m = Mark()
        same(k["key"], a.get("answer"), qid, m)
        if "without" in k and "without" in a:
            same(k["without"], a["without"], f"{qid} without the twist", m)
        out[qid] = ("fail" if m.diffs else ("read" if m.reads else "pass"), m.diffs, m.reads)
    return out


def report(res):
    for qid, (verdict, diffs, reads) in res.items():
        print(f"{qid}: {verdict}")
        for d in diffs:
            print(f"  ✗ {d}")
        for r in reads:
            print(f"  ? {r}")
    tally = {}
    for v, _, _ in res.values():
        tally[v] = tally.get(v, 0) + 1
    print("  ".join(f"{k} {n}" for k, n in sorted(tally.items())))
    return all(v in ("pass", "read") for v, _, _ in res.values())


def self_test(house):
    """The keys as answers must pass; one number moved by an ort must fail."""
    ks = keys(house)

    def as_answer(x):
        """A random key answered by its own estimate and standard error."""
        if isinstance(x, dict) and x.get("kind") == "random":
            return {"kind": "random", "value": x["r"], "se": x["se_r"]}
        if isinstance(x, dict):
            return {k: as_answer(v) for k, v in x.items()}
        if isinstance(x, list):
            return [as_answer(v) for v in x]
        return x

    ans = {q: {"answer": as_answer(k["key"]), **({"without": as_answer(k["without"])} if "without" in k else {})} for q, k in ks.items()}
    res = mark(house, ans)
    bad = [q for q, (v, _, _) in res.items() if v not in ("pass", "read")]
    assert not bad, f"keys don't pass as answers: {bad}"

    def nudge(x):
        if isinstance(x, dict):
            for k, v in x.items():
                r = nudge(v)
                if r is not None:
                    x[k] = r
                    return x
        if isinstance(x, list):
            for i, v in enumerate(x):
                r = nudge(v)
                if r is not None:
                    x[i] = r
                    return x
        if isinstance(x, Decimal):
            return x + Decimal("0.01")
        return None

    moved = 0
    for q, k in ks.items():
        a = as_answer(blocks("```yaml\nanswer: " + _enc(k["key"]) + "\n```")[0]["answer"])
        if nudge(a) is None:
            continue
        v = mark(house, {q: {"answer": a}})[q][0]
        assert v == "fail", f"{q}: a moved number still passes"
        moved += 1
    print(f"self-test {house}: {len(ks)} keys pass as answers; {moved} moved by 0.01 fail")
    self_test_small(house, ks)


def self_test_small(house, ks):
    """--small: a random answer whose se is above p fails as usual and passes under --small; one 5 combined standard
    errors or more from r fails under both."""
    global SMALL
    found = []

    def walk(x, path):
        if isinstance(x, dict) and x.get("kind") == "random":
            found.append((path, x))
        elif isinstance(x, dict):
            for k, v in x.items():
                walk(v, path + [k])
        elif isinstance(x, list):
            for i, v in enumerate(x):
                walk(v, path + [i])

    for q, k in ks.items():
        walk(k["key"], [q])
    for path, rk in found:
        se = 2 * rk["p"]
        far = rk["r"] + 5 * (se + rk["se_r"])  # at least 5 combined standard errors off, since √(a² + b²) ≤ a + b
        for value, want_full, want_small in ((rk["r"], "fail", "pass"), (far, "fail", "fail")):
            got = []
            for small in (False, True):
                SMALL = small
                m = Mark()
                same(rk, {"value": value, "se": se}, "/".join(map(str, path)), m)
                got.append("fail" if m.diffs else "pass")
            SMALL = False
            assert got == [want_full, want_small], f"{path}: value {value}, se {se}: {got}, expected {[want_full, want_small]}"
    if found:
        print(f"self-test {house} --small: {len(found)} random figures: se above p fails, and passes with --small; 5 combined se or more off fails both ways")


def _enc(x):
    sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
    from lib.keys import enc
    return enc(x)


def main(argv):
    global KEYS_ROOT, SMALL
    if "--small" in argv:
        SMALL = True
        argv = [a for a in argv if a != "--small"]
    if "--keys-root" in argv:
        i = argv.index("--keys-root")
        KEYS_ROOT = pathlib.Path(argv[i + 1]).resolve()
        argv = argv[:i] + argv[i + 2:]
    if "--self-test" in argv:
        self_test(argv[argv.index("--self-test") + 1])
        return 0
    path = pathlib.Path(argv[0])
    house = argv[argv.index("--keys") + 1] if "--keys" in argv else None
    answers = {}
    for b in blocks(path.read_text(encoding="utf-8")):
        answers[b["question"]] = b
    houses = [house] if house else sorted({q.split("-")[0] for q in answers})
    ok = True
    for h in houses:
        print(f"# {h}")
        ok &= report(mark(h, {q: a for q, a in answers.items() if q.startswith(h + "-")}))
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
