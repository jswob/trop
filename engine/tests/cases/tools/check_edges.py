"""Check the edge list: every H need, LC concern and mechanic sits in an edge or is named outside the set.

Usage: python3 -I tmp/cases/tools/check_edges.py [--hidden <boundaries-hidden.md>]

Reads tmp/design/inventory.md (rows marked H), tmp/learning/CONCERNS.md (LC-n headings),
tmp/design/hard-set.md (MX-nn rows) and tmp/cases/boundaries.md. Without --hidden, ids that sit
in no visible edge are only counted: the holdout covers them. With --hidden, every id must be found.
Exit 0 when the list passes, 1 otherwise.
"""
import os
import re
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", "..", ".."))  # the case set is at engine/tests/cases (4.5f)
INVENTORY = os.path.join(ROOT, "tmp/design/inventory.md")
CONCERNS = os.path.join(ROOT, "tmp/learning/CONCERNS.md")
HARD_SET = os.path.join(ROOT, "tmp/design/hard-set.md")
VISIBLE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "boundaries.md")

ID = re.compile(r"\b(N-\d{3,4}|LC-\d{1,2}|MX-\d\d)\b")
EDGE_HEAD = re.compile(r"^### ([A-Z]{3,4}-\d\d) · (.+?)( ★)?$")
FIELD = re.compile(r"^- (hard|where|key|probe|ids|counts)(?: \(emphasis\))?: (.*)$")
REQUIRED = ("hard", "where", "key", "probe", "ids")
TAG = re.compile(r"\b([LMQR]\d{1,2})\b")


def read(path):
    with open(path, encoding="utf-8") as f:
        return f.read().splitlines()


def universe():
    h = [l.split("|")[1].strip() for l in read(INVENTORY)
         if l.startswith("| N-") and l.rstrip().endswith("| H |")]
    lc = ["LC-" + m.group(1) for l in read(CONCERNS)
          for m in [re.match(r"^## LC-(\d+)\. ", l)] if m]
    mx = sorted({m.group(1) for l in read(HARD_SET)
                 for m in [re.match(r"^\| (MX-\d\d) \|", l)] if m})
    return h, lc, mx


def parse(path, errors):
    """Edges {id: {field: text, 'star': bool, 'ids': [...]}}, outside ids, count tags, count-table edges."""
    edges, outside, tags, table_edges = {}, {}, set(), set()
    section, current, fenced = None, None, False
    for n, line in enumerate(read(path), 1):
        if line.startswith("```"):
            fenced = not fenced
            continue
        if fenced:
            continue
        if line.startswith("## "):
            section, current = line[3:].strip(), None
            continue
        head = EDGE_HEAD.match(line)
        if head:
            eid = head.group(1)
            if eid in edges:
                errors.append(f"{path}:{n}: edge {eid} defined twice")
            current = edges[eid] = {"name": head.group(2), "star": bool(head.group(3)),
                                    "line": n, "idlist": [], "tags": []}
            continue
        if line.startswith("### "):
            errors.append(f"{path}:{n}: malformed edge heading: {line}")
            current = None
            continue
        field = FIELD.match(line)
        if current is not None and field:
            name, text = field.groups()
            current[name] = text
            if name == "ids":
                current["idlist"] = ID.findall(text)
                if not current["idlist"] and text.strip() != "none":
                    errors.append(f"{path}:{n}: ids must list ids or say none")
            if name == "counts":
                current["tags"] = TAG.findall(text)
            continue
        if section and section.startswith("Outside the set") and line.startswith("| "):
            cell = line.split("|")[1].strip()
            if ID.fullmatch(cell):
                if cell in outside:
                    errors.append(f"{path}:{n}: {cell} listed outside twice")
                outside[cell] = n
        if section and section.startswith("Counts") and line.startswith("| "):
            cells = [c.strip() for c in line.split("|")]
            if TAG.fullmatch(cells[1]):
                tags.add(cells[1])
                table_edges.update(re.findall(r"\b[A-Z]{3,4}-\d\d\b", cells[3]))
    for eid, e in edges.items():
        for f in REQUIRED:
            if not e.get(f, "").strip():
                errors.append(f"{path}:{e['line']}: edge {eid} has no '{f}'")
    return edges, outside, tags, table_edges


def main(argv):
    hidden = None
    if argv[:1] == ["--hidden"] and len(argv) == 2:
        hidden = argv[1]
    elif argv:
        print(__doc__)
        return 2
    errors = []
    h, lc, mx = universe()
    known = set(h) | set(lc) | set(mx)
    if len(h) != 217 or len(lc) != 29 or len(mx) != 40:
        errors.append(f"universe changed: {len(h)} H needs, {len(lc)} LC, {len(mx)} MX (expected 217, 29, 40)")

    v_edges, outside, v_tags, v_table = parse(VISIBLE, errors)
    h_edges, h_tags, h_table = {}, set(), set()
    if hidden:
        h_edges, h_out, h_tags, h_table = parse(hidden, errors)
        if h_out:
            errors.append("the hidden list may not name ids outside the set")
        for eid in set(v_edges) & set(h_edges):
            errors.append(f"edge {eid} is both visible and hidden")
        visible_text = "\n".join(read(VISIBLE))
        for eid in h_edges:
            if eid in visible_text:
                errors.append(f"hidden edge {eid} is named in the visible list")

    edges = {**v_edges, **h_edges}
    in_visible = {i for e in v_edges.values() for i in e["idlist"]}
    in_hidden = {i for e in h_edges.values() for i in e["idlist"]}
    in_edges = in_visible | in_hidden

    for i in sorted(in_edges | set(outside)):
        if i not in known:
            errors.append(f"{i} is not an H need, an LC concern or a mechanic")
    for i in sorted(in_edges & set(outside)):
        errors.append(f"{i} is both in an edge and outside the set")

    missing = [i for i in h + lc + mx if i not in in_edges and i not in outside]
    if hidden:
        for i in missing:
            errors.append(f"{i} sits in no edge")

    all_tags = v_tags | h_tags
    for eid, e in edges.items():
        for t in e["tags"]:
            if t not in all_tags:
                errors.append(f"edge {eid} cites count {t}, which no counts table has")
    for eid in sorted(v_table | h_table):
        if eid not in edges:
            errors.append(f"a counts table names {eid}, which is no edge")

    axes = {}
    for eid in v_edges:
        axes.setdefault(eid.split("-")[0], []).append(eid)
    for axis, ids in sorted(axes.items()):
        stars = [e for e in ids if v_edges[e]["star"]]
        if axis != "SURF" and len(stars) != 1:
            errors.append(f"axis {axis} has {len(stars)} edges marked hardest (expected 1)")

    print(f"universe: {len(h)} H needs, {len(lc)} LC concerns, {len(mx)} mechanics")
    print(f"visible: {len(v_edges)} edges on {len(axes)} axes; "
          + ", ".join(f"{a} {len(ids)}" for a, ids in sorted(axes.items())))
    print(f"ids in visible edges: {len(in_visible & known)}; outside the set: {len(outside)}")
    if hidden:
        print(f"hidden: {len(h_edges)} edges; ids only there: {len(in_hidden - in_visible)}")
    else:
        print(f"ids in no visible edge: {len(missing)} (run with --hidden to check them)")
    for e in errors:
        print("FAIL:", e)
    print("pass" if not errors else f"{len(errors)} failures")
    return 0 if not errors else 1


sys.exit(main(sys.argv[1:]))
