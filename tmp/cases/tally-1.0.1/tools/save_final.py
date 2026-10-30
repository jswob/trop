"""Save a sub-agent's final message, read from its transcript, never retyped.

Usage: python3 -I -B save_final.py <agent-transcript.jsonl> <out.md>
The final message is the SubagentHandback call's `message`; with none, the last
assistant text block.
"""
import json
import sys


def final_message(path):
    handback, last_text = None, None
    with open(path, encoding='utf-8') as f:
        for line in f:
            try:
                entry = json.loads(line)
            except ValueError:
                continue
            if entry.get('type') != 'assistant':
                continue
            for block in (entry.get('message') or {}).get('content') or []:
                if not isinstance(block, dict):
                    continue
                if block.get('type') == 'tool_use' and block.get('name') == 'SubagentHandback':
                    handback = (block.get('input') or {}).get('message')
                elif block.get('type') == 'text' and block.get('text', '').strip():
                    last_text = block['text']
    return handback if handback is not None else last_text


def main():
    src, out = sys.argv[1], sys.argv[2]
    text = final_message(src)
    if not text:
        sys.exit('no final message in ' + src)
    with open(out, 'w', encoding='utf-8') as f:
        f.write(text.rstrip('\n') + '\n')
    print('saved %d lines to %s' % (text.count('\n') + 1, out))


if __name__ == '__main__':
    main()
