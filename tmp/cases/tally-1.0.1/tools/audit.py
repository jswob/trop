"""Audit a sub-agent's transcript: tools used, paths read and written, holdout.

Usage: python3 -I -B audit.py <agent-transcript.jsonl> --read GLOB ... --write GLOB ...
GLOBs are repository-relative fnmatch patterns ('*' crosses '/'). Prints the tools
used, any forbidden tool, every path outside its list, and any holdout mention
in a tool call (the two holdout paths, anywhere). Exit 2 on a holdout hit, 1 on
another breach, 0 when clean.
"""
import fnmatch
import json
import os
import re
import shlex
import sys
from collections import Counter

ROOT = '/Users/jswob/projects/trop'
FORBIDDEN = {'Agent', 'Task', 'Workflow', 'Skill', 'WebFetch', 'WebSearch', 'SendMessage'}
HOLDOUT = re.compile(r'cases-holdout|tmp/holdout|(^|[\s/"\'])holdout/')
ALWAYS_OK = ['/dev/null', '/tmp/*', '/private/tmp/*', '/usr/*', '/bin/*', '/opt/*', '/Library/*',
             '/System/*', '/Users/jswob/.claude/projects/*/tool-results/*']
WRITE_TOOLS = {'Write', 'Edit', 'NotebookEdit'}
READ_TOOLS = {'Read', 'Grep', 'Glob'}


def rel(path):
    path = os.path.normpath(os.path.join(ROOT, os.path.expanduser(path)))
    return os.path.relpath(path, ROOT) if path.startswith(ROOT) else path


def ok(path, globs):
    r = rel(path)
    if r.startswith('/'):
        return any(fnmatch.fnmatch(r, g) for g in ALWAYS_OK)
    return any(fnmatch.fnmatch(r, g) or fnmatch.fnmatch(r + '/', g) for g in globs)


def bash_paths(cmd):
    """Path-like tokens of a shell command, resolved against `cd` and simple VAR=,
    and whether each is written (after >). Sed/awk scripts, regexes and globs
    given to -name are skipped."""
    found = []
    cwd = ROOT
    env = {}
    cmd = re.sub(r"<<-?\s*'?\"?(\w+)'?\"?(.*?)\n.*?\n\1(\n|$)", r"\2\n", cmd, flags=re.S)
    for seg in re.split(r'\n|&&|\|\||;|\|', cmd):
        try:
            toks = shlex.split(seg, comments=True, posix=True)
        except ValueError:
            toks = seg.split()
        if not toks:
            continue
        m = re.match(r'^([A-Za-z_]\w*)=(\S+)$', toks[0])
        if m and len(toks) == 1:
            env[m.group(1)] = m.group(2)
            continue
        if toks[0] == 'cd' and len(toks) > 1:
            cwd = os.path.normpath(os.path.join(cwd, os.path.expanduser(sub(toks[1], env))))
            continue
        prog = toks[0]
        prev = ''
        for t in toks[1:]:
            redirect = None
            mm = re.match(r'^(\d?>>?|<)(.*)$', t)
            if mm:
                redirect, t = mm.group(1), mm.group(2)
            if not t:
                prev = redirect
                continue
            t = sub(t, env)
            skip = (prev in ('-name', '-iname', '-path', '-e', '-E') or prog in ('sed', 'awk', 'echo', 'printf')
                    and not redirect and not os.path.exists(os.path.join(cwd, t)))
            if (not skip and re.match(r'^[\w./~-][\w./~@+,-]*$', t) and ('/' in t or re.search(r'\.(md|tally|py|csv|lark|txt|jsonl?)$', t))
                    and not t.startswith('-') and not re.match(r'^s/|^/.*/$', t)):
                write = bool(redirect and '>' in redirect) or prev in ('>', '>>', '1>', '2>')
                full = os.path.normpath(os.path.join(cwd, os.path.expanduser(t)))
                found.append((full, write))
            prev = redirect or t
    return found


def sub(t, env):
    return re.sub(r'\$\{?(\w+)\}?', lambda m: env.get(m.group(1), m.group(0)), t)


def main():
    args = sys.argv[1:]
    path = args.pop(0)
    reads, writes, mode = [], [], None
    for a in args:
        if a in ('--read', '--write'):
            mode = a
        elif mode == '--read':
            reads.append(a)
        else:
            writes.append(a)
    reads = reads + writes
    tools, breaches, holdout = Counter(), [], []
    with open(path, encoding='utf-8') as f:
        for line in f:
            try:
                e = json.loads(line)
            except ValueError:
                continue
            if e.get('type') != 'assistant':
                continue
            for b in (e.get('message') or {}).get('content') or []:
                if not isinstance(b, dict) or b.get('type') != 'tool_use':
                    continue
                name, inp = b.get('name'), b.get('input') or {}
                tools[name] += 1
                blob = json.dumps(inp)
                if name != 'SubagentHandback' and HOLDOUT.search(blob):
                    holdout.append('%s %s' % (name, blob[:200]))
                if name in FORBIDDEN:
                    breaches.append('forbidden tool %s' % name)
                if name in WRITE_TOOLS:
                    p = inp.get('file_path', '')
                    if not ok(p, writes):
                        breaches.append('write outside scope: %s %s' % (name, rel(p)))
                elif name in READ_TOOLS:
                    p = inp.get('file_path') or inp.get('path') or ROOT
                    if not ok(p, reads):
                        breaches.append('read outside scope: %s %s' % (name, rel(p)))
                    if name in ('Grep', 'Glob') and rel(p) in ('.', 'tmp'):
                        breaches.append('search over a broad root: %s %s' % (name, rel(p)))
                elif name == 'Bash':
                    cmd = inp.get('command', '')
                    for p, w in bash_paths(cmd):
                        if not ok(p, writes if w else reads):
                            breaches.append('bash %s outside scope: %s   [%s]' % ('write' if w else 'path', p, cmd[:120].replace('\n', ' ')))
                    if re.search(r'\b(find|grep -r|grep -R|rg|ls -R)\b[^|;]*\s(\.|tmp/?|/Users/jswob/projects/trop/?)(\s|$)', cmd):
                        breaches.append('bash search over a broad root: %s' % cmd[:160].replace('\n', ' '))
    print('tools:', dict(tools))
    for h in holdout:
        print('HOLDOUT', h)
    seen = set()
    for b in breaches:
        if b not in seen:
            seen.add(b)
            print('breach:', b)
    print('summary: %d calls, %d holdout, %d distinct breaches' % (sum(tools.values()), len(holdout), len(seen)))
    sys.exit(2 if holdout else 1 if seen else 0)


if __name__ == '__main__':
    main()
