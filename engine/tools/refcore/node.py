"""A generic syntax-tree node: a kind and named fields."""


class Node:
    def __init__(self, k, loc=None, **kw):
        self.k = k
        self.loc = loc
        self.__dict__.update(kw)

    def get(self, name, default=None):
        return self.__dict__.get(name, default)

    def __repr__(self):
        fields = ', '.join('%s=%r' % (a, b) for a, b in self.__dict__.items()
                           if a not in ('k', 'loc') and not a.startswith('_'))
        return '%s(%s)' % (self.k, fields)


def walk(node):
    """Yield every Node reachable from node (pre-order)."""
    stack = [node]
    while stack:
        n = stack.pop()
        if isinstance(n, Node):
            yield n
            for a, b in list(n.__dict__.items()):
                if a in ('loc',) or a.startswith('_'):
                    continue
                if isinstance(b, (Node, list, tuple)):
                    stack.append(b)
        elif isinstance(n, (list, tuple)):
            for x in reversed(n):
                if isinstance(x, (Node, list, tuple)):
                    stack.append(x)
