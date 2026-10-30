"""Count RULEBOOK rows per group by their Rec. column (W3 scratch, reads text only)."""
import re, sys
path = sys.argv[1]
group = None
counts = {}
for line in open(path, encoding="utf-8"):
    m = re.match(r"^## (\d+)\. (.*)", line)
    if m:
        group = m.group(1) + ". " + m.group(2).strip()
        continue
    if line.startswith("| R-"):
        cells = [c.strip() for c in line.strip().strip("|").split(" | ")]
        rid, rec = cells[0].split()[0], cells[6]
        if rec.startswith("drop"):
            k = "drop"
        elif rec.startswith("report →") or rec.startswith("keep"):
            k = "keep"
        elif rec.startswith("report"):
            k = "report"
        else:
            k = "?" + rec
        c = counts.setdefault(group, {"rows": [], "keep": 0, "report": 0, "drop": 0})
        c["rows"].append(rid)
        c[k] = c.get(k, 0) + 1
        if len(cells) != 8:
            print("CELLS", rid, len(cells))
tot = {"rows": 0, "keep": 0, "report": 0, "drop": 0}
for g, c in counts.items():
    print(g, len(c["rows"]), "keep", c["keep"], "report", c["report"], "drop", c["drop"], c["rows"][0], c["rows"][-1])
    tot["rows"] += len(c["rows"])
    for k in ("keep", "report", "drop"):
        tot[k] += c[k]
print("TOTAL", tot)
