#!/usr/bin/env python3
"""Write INDEX.md: each edge, its probe card (and any pair card naming it), and the questions that use it (from the keys' `edges:`).

The visible index (tmp/cases/INDEX.md) lists every visible edge with the questions of A, B, D and E1.
With --hidden <holdout root> it also writes that root's INDEX-hidden.md: the holdout's own edges and
every edge its questions use, with those questions. Each table has an empty status column for
session 3's gap log (brief §11). check.py rebuilds both and compares them with the files on disk.

Run: python3 -I tmp/cases/tools/index.py [--hidden tmp/cases-holdout]
"""
import json
import pathlib
import re
import sys

TOOLS = pathlib.Path(__file__).resolve().parent
VIS = TOOLS.parent
EDGE_HEAD = re.compile(r"^### ([A-Z]{3,4}-\d\d) · (.+?)( ★)?$")
AXIS_HEAD = re.compile(r"^## ([A-Z]{3,4}): (.+)$")
VISIBLE_HOUSES = ("A", "B", "D", "E1")


def edges(path):
    """[(axis, axis name, edge id, edge name, star)] in the file's order."""
    out, axis, aname = [], None, None
    for ln in path.read_text(encoding="utf-8").splitlines():
        m = AXIS_HEAD.match(ln)
        if m:
            axis, aname = m.group(1), m.group(2)
            continue
        m = EDGE_HEAD.match(ln)
        if m:
            out.append((axis or m.group(1).split("-")[0], aname or "", m.group(1), m.group(2), bool(m.group(3))))
    return out


def key_edges(keys_root, houses):
    """{edge: [question ids]} from each key's `edges:` line."""
    use = {}
    for h in houses:
        for p in sorted((keys_root / h).glob("*.md")):
            for ln in p.read_text(encoding="utf-8").splitlines():
                if ln.startswith("edges: "):
                    for e in json.loads(ln[7:]):
                        use.setdefault(e, []).append(p.stem)
                    break
    return use


def probe_cell(eid):
    p = VIS / "probes" / eid.split("-")[0].lower() / f"{eid}.md"
    pairs = "".join(f"; pair `probes/pairs/{q.name}`" for q in sorted((VIS / "probes" / "pairs").glob("*.md")) if eid in q.stem.split("+"))
    if not p.exists():
        return "—" + pairs
    keyed = (VIS / "keys" / "probes" / f"{eid}.md").exists()
    return f"`probes/{eid.split('-')[0].lower()}/{eid}.md`" + ("" if keyed else " (a passage, no key)") + pairs


def visible_index():
    rows, last = [], None
    use = key_edges(VIS / "keys", VISIBLE_HOUSES)
    out = ["# Case set index", "",
           "Each visible edge (`boundaries.md`), its probe card, and the questions of A, B, D and E1 that use it, "
           "taken from the keys' `edges:` lines. Session 3 fills the status column with the gap log's worst status "
           "for the edge (breaks, can't express, awkward, expressed; brief §11); session 4 works from here, one edge at a time.",
           "", "Written by `tools/index.py`; don't edit by hand.", ""]
    for axis, aname, eid, name, star in edges(VIS / "boundaries.md"):
        if axis != last:
            if rows:
                out += rows + [""]
            out += [f"## {axis}: {aname}", "", "| Edge | Name | Probe | Questions | Status |", "|---|---|---|---|---|"]
            rows, last = [], axis
        qs = ", ".join(sorted(set(use.get(eid, [])))) or "—"
        rows.append(f"| {eid}{' ★' if star else ''} | {name} | {probe_cell(eid)} | {qs} | |")
    out += rows + [""]
    return "\n".join(out)


def hidden_index(hold):
    hidden = edges(hold / "boundaries-hidden.md")
    vis = {e[2]: e for e in edges(VIS / "boundaries.md")}
    # cases 1.2.0: the holdout's households only (in the merged tree keys/ also holds the visible ones and the probes)
    houses = sorted(p.name for p in (hold / "keys").iterdir() if p.is_dir() and p.name not in ("A", "B", "D", "E1", "probes")
                    and (hold / p.name / "questions.md").exists())
    use = key_edges(hold / "keys", houses)
    out = ["# Holdout index", "",
           "The holdout's own edges (`boundaries-hidden.md`), then every visible edge the holdout's questions use, "
           f"with those questions ({', '.join(houses)}), from the keys' `edges:` lines. The status column is for the "
           "holdout's one-shot scoring (brief §8, §11).", "", "Written by `tmp/cases/tools/index.py --hidden`; don't edit by hand.", "",
           "## Hidden edges", "", "| Edge | Name | Questions | Status |", "|---|---|---|---|"]
    for _, _, eid, name, star in hidden:
        out.append(f"| {eid}{' ★' if star else ''} | {name} | {', '.join(sorted(set(use.get(eid, [])))) or '—'} | |")
    out += ["", "## Visible edges the holdout uses", "", "| Edge | Name | Probe | Questions | Status |", "|---|---|---|---|---|"]
    for eid in sorted(e for e in use if e in vis):
        _, _, _, name, star = vis[eid]
        out.append(f"| {eid}{' ★' if star else ''} | {name} | {probe_cell(eid)} | {', '.join(sorted(set(use[eid])))} | |")
    out.append("")
    return "\n".join(out)


def main(argv):
    (VIS / "INDEX.md").write_text(visible_index(), encoding="utf-8")
    print("INDEX.md written")
    if "--hidden" in argv:
        hold = pathlib.Path(argv[argv.index("--hidden") + 1]).resolve()
        (hold / "INDEX-hidden.md").write_text(hidden_index(hold), encoding="utf-8")
        print("INDEX-hidden.md written")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
