"""Errors of the reference interpreter.

Every failure carries a code of the spec (L §8.3, §8.4) or one of the two
codes of the reference interpreter itself:

- REFCORE-UNSPECIFIED <section>: <what>  the spec is unclear or silent here;
- REFCORE-NOT-YET <construct>            a construct left to a later writer.

At load any TallyError stops the reality.  At run a TallyError is a
per-value refusal: it propagates like a hole and outranks it (L §8.4).
"""


class TallyError(Exception):
    def __init__(self, code, msg, loc=None):
        Exception.__init__(self, code, msg)
        self.code = code
        self.msg = msg
        self.loc = loc

    def text(self):
        s = '%s %s' % (self.code, self.msg)
        if self.loc:
            s += ' (%s)' % self.loc
        return s

    def key(self):
        return (self.code, self.msg)

    def __str__(self):
        return self.text()


class LoadError(TallyError):
    """A refusal at load (L §8.3)."""


class Refusal(TallyError):
    """A refusal per value (L §8.4)."""


class Unspecified(TallyError):
    def __init__(self, section, what, loc=None):
        TallyError.__init__(self, 'REFCORE-UNSPECIFIED', '%s: %s' % (section, what), loc)
        self.section = section
        self.what = what


class NotYet(TallyError):
    def __init__(self, construct, loc=None):
        TallyError.__init__(self, 'REFCORE-NOT-YET', construct, loc)


class RandomLine(Refusal):
    """LOAD-RANDOM that refcore finds only at a read (a random value with no query and no path, reached through a
    balance or a projection: E §3.1 step 10, L §5.16). Inside a walk it is a value's refusal, like any other, so a
    movement no ask line reads stops nothing; an ask line whose value it is stops the reality as a load refusal does
    (ask.is_load)."""

    def __init__(self, msg):
        Refusal.__init__(self, 'LOAD-RANDOM', msg)


def load_now(e):
    """Whether an error met while evaluating is a load refusal that stops the reality at once."""
    return e.code.startswith('LOAD-') and not isinstance(e, RandomLine)


def load(code, msg, loc=None):
    return LoadError(code, msg, loc)


def run(code, msg):
    return Refusal(code, msg)
