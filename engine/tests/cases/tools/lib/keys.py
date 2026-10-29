"""Keys: read a probe card's YAML block, compare answers, write key files with their derivations.

The YAML blocks use one top-level `name: value` per entry, each value in flow style (JSON), so
the standard library reads them; numbers are read as exact Decimals.
"""
import json
import re
from decimal import Decimal
from fractions import Fraction

from .num import Refuse

_TOP = re.compile(r"^([a-z_]+):\s?(.*)$")


def load_block(text):
    """The first ```yaml block of a Markdown file, as a dict."""
    m = re.search(r"```yaml\n(.*?)```", text, re.S)
    if not m:
        return None
    out, name, buf = {}, None, []
    for line in m.group(1).split("\n"):
        t = _TOP.match(line)
        if t and not line.startswith(" "):
            if name:
                out[name] = _value("\n".join(buf))
            name, buf = t.group(1), [t.group(2)]
        else:
            buf.append(line)
    if name:
        out[name] = _value("\n".join(buf))
    return out


def _value(s):
    s = s.strip()
    try:
        return json.loads(s, parse_float=Decimal)
    except json.JSONDecodeError:
        return s


def enc(x):
    """Flow YAML (JSON with exact numbers)."""
    if isinstance(x, dict):
        return "{" + ", ".join(f"{json.dumps(str(k))}: {enc(v)}" for k, v in x.items()) + "}"
    if isinstance(x, (list, tuple)):
        return "[" + ", ".join(enc(v) for v in x) + "]"
    if isinstance(x, bool) or x is None:
        return json.dumps(x)
    if isinstance(x, Decimal):
        return str(x)
    if isinstance(x, int):
        return str(x)
    if isinstance(x, Fraction):
        return json.dumps(f"{x.numerator}/{x.denominator}")
    return json.dumps(str(x), ensure_ascii=False)


def fmt(x):
    """A value inside a derivation line."""
    if isinstance(x, Fraction):
        return f"{x.numerator}/{x.denominator}"
    if isinstance(x, (dict, list, tuple)):
        return enc(x)
    return str(x)


def norm(x):
    """Answers in comparable form: numbers as Decimals, fractions as 'a/b' strings."""
    if isinstance(x, dict):
        return {str(k): norm(v) for k, v in x.items()}
    if isinstance(x, (list, tuple)):
        return [norm(v) for v in x]
    if isinstance(x, bool) or x is None:
        return x
    if isinstance(x, (int, Decimal)):
        return Decimal(x)
    if isinstance(x, Fraction):
        return f"{x.numerator}/{x.denominator}"
    return str(x)


def diff(expect, got, path="$"):
    """Differences between a hand-worked answer and a key, [] when they agree.

    A random part {"kind": "random", "r", "p"} in `expect` agrees when the key's own standard
    error is at most p/4 and its r lies within 4 of them of the hand-worked r.
    """
    e, g = norm(expect), norm(got)
    if isinstance(e, dict) and e.get("kind") == "random" and isinstance(g, dict):
        out = []
        if g.get("se_r") is None or g["se_r"] > e["p"] / 4:
            out.append(f"{path}: se_r {g.get('se_r')} above p/4")
        elif abs(g["r"] - e["r"]) > 4 * g["se_r"]:
            out.append(f"{path}: r {g['r']} not within 4 se of {e['r']}")
        return out
    if isinstance(e, dict) and isinstance(g, dict):
        out = []
        for k in sorted(set(e) | set(g)):
            if k not in e and k in ("reason", "note", "unit"):
                continue
            if k not in e or k not in g:
                out.append(f"{path}.{k}: only in {'key' if k in g else 'card'}")
            else:
                out += diff(e[k], g[k], f"{path}.{k}")
        return out
    if isinstance(e, list) and isinstance(g, list):
        if len(e) != len(g):
            return [f"{path}: length {len(e)} vs {len(g)}"]
        return [d for i, (a, b) in enumerate(zip(e, g)) for d in diff(a, b, f"{path}[{i}]")]
    return [] if e == g else [f"{path}: card {fmt(e)} vs key {fmt(g)}"]


class Trace:
    """The derivation: numbered steps of what, how and the result."""

    def __init__(self):
        self.steps = []

    def __call__(self, what, value, how=""):
        self.steps.append((what, how, value))
        return value

    def render(self):
        out = []
        for i, (what, how, value) in enumerate(self.steps, 1):
            out.append(f"{i}. {what}: {how} = {fmt(value)}" if how else f"{i}. {what} = {fmt(value)}")
        return "\n".join(out)


def refusal(e: Refuse):
    return {"kind": "refusal", "missing": e.missing, "reason": e.reason}


def write(path, edge, key, trace, extra=None):
    path.parent.mkdir(parents=True, exist_ok=True)
    head = [f"edge: {enc(edge)}"] + [f"{k}: {enc(v)}" for k, v in (extra or {}).items()] + [f"key: {enc(key)}"]
    body = f"# Key · {edge}\n\nWritten by `tools/keys_probes.py`; don't edit.\n\n```yaml\n" + "\n".join(head) + "\n```\n\n## Derivation\n\n" + trace.render() + "\n"
    path.write_text(body, encoding="utf-8")
