"""pr-body: an engine pull request's description has the convention's sections (R-77, R-100;
the retro after M0, decision 7; engine/CONVENTIONS.md).

    engine/tools/pr-body FILE       the description in FILE (`-` reads stdin)

It fails unless the description's `##` headings before the fold are exactly these six, in
this order, and a folded `<details><summary>For sessions</summary>` block follows them and
is closed. HTML comments and fenced code are skipped; `###` headings are free. CI's
`pr body` job runs it on every engine pull request, with the description GitHub holds
when the job runs, and `engine gate` needs it.
"""

import re
import sys

HEADINGS = (
    "What this delivers",
    "Changes you can see",
    "Decisions taken for you",
    "Risks and follow-ups",
    "How it was checked",
    "Merge notes",
)
FOLD = re.compile(r"<details>\s*<summary>\s*(?:<b>|<strong>)?\s*For sessions\s*(?:</b>|</strong>)?\s*</summary>", re.IGNORECASE)


def problems(text):
    text = text.replace("\r\n", "\n")
    fold = FOLD.search(text)
    head = text[: fold.start()] if fold else text
    head = re.sub(r"<!--.*?-->", "", head, flags=re.S)
    head = re.sub(r"^(```|~~~).*?^\1[ \t]*$", "", head, flags=re.S | re.M)
    found = [h.strip().rstrip("#").strip() for h in re.findall(r"^##[ \t]+(.+)$", head, re.M)]
    out = []
    if found != list(HEADINGS):
        for k, want in enumerate(HEADINGS):
            got = found[k] if k < len(found) else None
            if got != want:
                out.append(f"heading {k + 1} is {('## ' + got)!r} where `## {want}` belongs" if got else f"no `## {want}` (heading {k + 1})")
                break
        else:
            out.append(f"an extra heading `## {found[len(HEADINGS)]}` before the fold")
    if not fold:
        out.append("no folded `<details><summary>For sessions</summary>` block after the six sections")
    elif "</details>" not in text[fold.end() :]:
        out.append("the For sessions block isn't closed with </details>")
    return out


def main(argv):
    if len(argv) != 1:
        print("usage: engine/tools/pr-body FILE   (`-` reads stdin)", file=sys.stderr)
        return 64
    try:
        text = sys.stdin.read() if argv[0] == "-" else open(argv[0], encoding="utf-8").read()
    except OSError as e:
        print(f"FAIL pr-body: can't read {argv[0]}: {e}")
        return 1
    found = problems(text)
    for p in found:
        print(f"FAIL pr-body: {p} (engine/CONVENTIONS.md, engine/templates/pr.md)")
    if found:
        return 1
    print("pr-body: the six sections in order, and the folded For sessions block")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
