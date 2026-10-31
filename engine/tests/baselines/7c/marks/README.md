# The marks behind the 7c baselines (cases 1.4.0)

The 7 outputs in `..` are refcore's runs of the blind cards' checks (`defs/blind7.tsv`), made 2026-10-30 by
`run.py 'I:S*' --interp refcore --jobs 2`: one per card S1..S6, with all its asks, on session 7b's blind texts copied
unchanged (`integration/blind7/S<n>/s<n>.tally`), and S4's pinned twin (`s4.tally` with `S4/pinned.tally`, the judge's
asks `j3_*`, `j4_*`, `j5_*` from `tmp/session7b/blind-run/judge/S4check/s4_pinned.tally`).

Each card's output is byte-equal to the judge's run of the same text (`tmp/session7b/blind-run/S<n>/run.out`) less that
file's last line, `# rc 0, N s`, which `run.py --adhoc` adds and a check never prints. The texts are unchanged and
refcore's 7c fixes don't reach them. The twin's output is new; its figures are those of the judge's `S4check` run
(`causes.md`).

- `S<n>.map.md`: the judge's slot maps (`tmp/session7b/blind-run/judge/`), unchanged; S4's adds the twin's two blocks
  (S4-03P, S4-04P) over the twin's output. S4-03's and S4-04's keys are now refusals (below), and the judge's slots fit
  them as mapped.
- `S<n>.answers.md`: `engine/tests/tools/answers.py` over these outputs (S4: over the card's and the twin's). Each card's
  answers equal the judge's but for the runs' names.
- `S<n>.compare.txt`: `compare.py --keys S<n> --keys-root engine/tests/cases/keys` over them. S1, S2, S3, S5 and S6 are
  byte-equal to the judge's `compare.txt`: S1 1 pass, 2 read; S2 2 pass, 1 read; S3 3 pass; S5 3 pass; S6 2 pass,
  1 read. S4: S4-01 pass, S4-02 read, S4-03 read, S4-04 read, S4-03P pass, S4-04P pass.

So the score stands as `tmp/session7b/blind-run/judge/score.md` recorded it (12 pass, 5 reads agreed, 2 fail of 19):
every pass and read is the same, and the reads keep the judge's verdicts (all agreed). The one change is S4-03 and
S4-04, card faults by the user's G2 Q-1: their keys now name the missing ruling (`keys_s4.py`, ERRATA ADD-10), so the
run's hole, which names the will's reason (`reason_true[edvin_estate, lea]`), is a read line in each of their 8 slots;
these reads are new in cases 1.4.0, outside the recorded score, for the lead to agree. The by-case figures moved to the
pinned twin, S4-03P and S4-04P, which pass on every slot. Ruta on 2027-12-31 is the twin's slot
"Ruta, as known 2028-01-01" (`j5_*`), for the reason in S4-04P's step 13.

A check here compares the whole output with its baseline; a change is a `differ` to judge against these marks.
