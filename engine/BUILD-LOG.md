# The build log

The build session's log: one line per step, newest last, and a STOP-REPORT when the
builder stops. The guard and the Stop hook append their own lines here (a blocked
settings change, "STOPPED RED"). The builder may write this file (R-79); nobody edits
a line once it is committed.

## How to stop: the STOP-REPORT

A STOP-REPORT is the builder's legal exit when a test, the spec or a rule seems wrong
(R-83). A turn can't end while `engine/tools/check fast` is red, unless a STOP-REPORT
heading was added to this file since the last commit, or by it. A rule change starts
here and goes through R-112's procedure: the lead reproduces it, the user approves it
in plain words, dated in the rulebook, and the `rules` role commits it.

Copy the template below to the end of this file, without the four-space indent (the
indent keeps the template itself from counting as a report):

    ## STOP-REPORT 2026-MM-DD: <one line: what blocks the build>

    - **Kind:** a test seems wrong | the spec seems wrong | a rule blocks correct code | other
    - **Where:** <file:line of the code, the test, the vector or the spec line>
    - **Rule or gate:** <R-n, or DONE gate n, or the spec's section>
    - **What happens:** <the command, quoted, with its exit code and the first error line>
    - **What I tried:** <the strict alternative tried, and why it doesn't work>
    - **What I ask:** <the change asked for, or the question for the lead>
    - **State:** <the last green commit; what is uncommitted>

## Log
