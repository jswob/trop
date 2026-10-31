"""Session 8c's cost: tmp/design2/scratch/lead/usage2.py, but over every project directory.

Usage: python3 -I -B usage.py <session id> [<session id> ...]
The 8c lead runs in the scaffold worktree, and the eval's runs in throwaway worktrees, so
their transcripts sit under project directories other than -Users-jswob-projects-trop.
Each id's transcript and its subagents are found under ~/.claude*/projects/*/; calls are
counted once by message id across all of them. Weights and calibration are
engine/tests/cases/tools/usage.py's (main checkout), as in usage2.py.
"""
import glob, os, runpy, sys
ROOT = '/Users/jswob/projects/trop'
u = runpy.run_path(os.path.join(ROOT, 'engine', 'tests', 'cases', 'tools', 'usage.py'), run_name='usage')
grand = 0.0
for session in sys.argv[1:]:
    calls = {}
    for base in ('~/.claude', '~/.claude-personal'):
        for b in sorted(glob.glob(os.path.expanduser(base + '/projects/*'))):
            for path in [os.path.join(b, session + '.jsonl')] + sorted(glob.glob(os.path.join(b, session, 'subagents', '*.jsonl'))):
                if not os.path.exists(path):
                    continue
                for entry in u['entries'](path):
                    m = entry.get('message') or {}
                    if entry.get('type') == 'assistant' and m.get('usage'):
                        calls[m.get('id') or entry.get('uuid')] = m['usage']
    total = 0.0
    for usage in calls.values():
        total += sum(w * (usage.get(k) or 0) for k, w in u['WEIGHTS'].items())
        split = usage.get('cache_creation') or {}
        if split:
            total += 1.25 * (split.get('ephemeral_5m_input_tokens') or 0)
            total += 2 * (split.get('ephemeral_1h_input_tokens') or 0)
        else:
            total += 1.25 * (usage.get('cache_creation_input_tokens') or 0)
    grand += total
    print(f'session {session}: {len(calls)} calls, {total / 1e6:.2f} M weighted, about {total / u["PER_PERCENT"]:.2f} % of the week')
if len(sys.argv) > 2:
    print(f'all: about {grand / u["PER_PERCENT"]:.2f} % of the week')
