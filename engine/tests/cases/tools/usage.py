"""Cost of a case-set session, read from its Claude Code transcript.

Usage: python3 -I tmp/cases/tools/usage.py [session-id | transcript.jsonl]
With no argument it reads the newest transcript of this project, a session's or a sub-agent's,
whose opening messages hold one of the paste lines in MARKERS (a step of the case-set plan, or
the blind reader), so another open session isn't counted. A session counts with its sub-agents;
a sub-agent, or a transcript passed by path, counts alone. Tokens are weighted by Anthropic's relative prices: fresh input 1, cache
write 1.25 (5 min) or 2 (1 h), cache read 0.1, output 5. Calibration (2026-10-24): session 2's
129 M weighted was about 15 % of the user's weekly limit, so 1 % is about 8.6 M.
"""
import glob
import json
import os
import sys

PER_PERCENT = 8.6e6
WEIGHTS = {'input_tokens': 1, 'cache_read_input_tokens': 0.1, 'output_tokens': 5}
MARKERS = ('Run the next step of the case-set plan', 'You are the blind reader of the case set')


def entries(path):
    with open(path, encoding='utf-8') as f:
        for line in f:
            try:
                yield json.loads(line)
            except ValueError:
                continue


def opening(path, count=20):
    """The texts of a transcript's first user messages."""
    texts = []
    for entry in entries(path):
        if entry.get('type') != 'user':
            continue
        content = (entry.get('message') or {}).get('content')
        if isinstance(content, str):
            texts.append(content)
        elif isinstance(content, list):
            texts += [block.get('text') or '' for block in content
                      if isinstance(block, dict) and block.get('type') == 'text']
        count -= 1
        if count == 0:
            break
    return texts


def weighted(path):
    calls = {}
    for entry in entries(path):
        message = entry.get('message') or {}
        if entry.get('type') == 'assistant' and message.get('usage'):
            calls[message.get('id') or entry.get('uuid')] = message['usage']
    total = 0.0
    for usage in calls.values():
        total += sum((usage.get(key) or 0) * weight for key, weight in WEIGHTS.items())
        split = usage.get('cache_creation') or {}
        if split:
            total += 1.25 * (split.get('ephemeral_5m_input_tokens') or 0)
            total += 2 * (split.get('ephemeral_1h_input_tokens') or 0)
        else:
            total += 1.25 * (usage.get('cache_creation_input_tokens') or 0)
    return len(calls), total


def main():
    root = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..', '..'))
    base = os.path.expanduser('~/.claude/projects/' + root.replace('/', '-'))
    found = None
    if len(sys.argv) > 1 and sys.argv[1].endswith('.jsonl'):
        found = sys.argv[1]
    elif len(sys.argv) > 1:
        session = sys.argv[1]
    else:
        transcripts = glob.glob(os.path.join(base, '*.jsonl')) + glob.glob(os.path.join(base, '*', 'subagents', '*.jsonl'))
        newest_first = sorted(transcripts, key=os.path.getmtime, reverse=True)
        # a message must start with the paste line: a compacted session's summary only quotes it
        found = next((path for path in newest_first
                      if any(text.lstrip().startswith(MARKERS) for text in opening(path))), None)
        if found is None:
            sys.exit('usage.py: no case-set session found; pass a session id')
        if os.path.basename(os.path.dirname(found)) != 'subagents':
            session, found = os.path.basename(found)[:-len('.jsonl')], None
    if found:
        session = os.path.basename(found)[:-len('.jsonl')]
        paths = [found]
    else:
        paths = [os.path.join(base, session + '.jsonl')]
        paths += sorted(glob.glob(os.path.join(base, session, 'subagents', '*.jsonl')))
    calls = total = 0
    for path in paths:
        count, cost = weighted(path)
        calls += count
        total += cost
    print(f'session {session}: {calls} calls, {total / 1e6:.2f} M weighted, '
          f'about {total / PER_PERCENT:.2f} % of the week')


if __name__ == '__main__':
    main()
