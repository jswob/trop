# Keys for runs cut short

Keys made by a key script's knob for a smaller run than the question's, so that a run cut in paths or in time can be marked. Nothing here replaces a key in `../keys/`; no script writes `../keys/` with a knob set.

Mark an answer against them with `python3 -I tmp/cases/tools/compare.py <answers.md> --keys B --keys-root tmp/cases/keys-small --small`. With `--small`, a random answer passes if it lies within 4 combined standard errors of the key's reference (`|value − r| ≤ 4 × √(se² + se_r²)`); the precision bar `se ≤ p` is dropped.

| File | Made by | What it is |
|---|---|---|
| `B/B-12.md` | `python3 -I tmp/cases/tools/keys_b.py 30 --end 2027-01-31` | B-12 at 30 paths (seed 20260102 + i, the first 30 of B-12's), with B's forecast ending on 2027-01-31 instead of 2028-12-31. It keeps the 2027 law draw, both 2026 mortgage resets and the house rules B-12 reads. B-12's labels are kept: each figure B-12 takes on 2028-12-31 is taken on 2027-01-31, and each sum or share runs over 2026-01 .. 2027-01. |
