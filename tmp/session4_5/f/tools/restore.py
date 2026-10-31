"""Write each moved file's bytes from HEAD:<old> to <new>, then check every one."""
import hashlib, subprocess, sys
rows = [l.rstrip('\n').split('\t') for l in open(sys.argv[1], encoding='utf-8')][1:]
p = subprocess.Popen(['git', 'cat-file', '--batch'], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
bad = 0
for o, n in rows:
    p.stdin.write(('HEAD:%s\n' % o).encode()); p.stdin.flush()
    head = p.stdout.readline().split()
    assert head[1] == b'blob', (o, head)
    data = p.stdout.read(int(head[2])); p.stdout.read(1)
    cur = open(n, 'rb').read()
    if cur != data:
        open(n, 'wb').write(data)
        bad += 1
    assert open(n, 'rb').read() == data
print(len(rows), 'checked;', bad, 'restored to their original bytes')
