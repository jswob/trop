# Mutation study (experiment, never merged)

Branch `rules/mutation-study` only. Measures a coverage-selected gate 8 on M1's mutants on
GitHub's runners: per-test kernel coverage of T's mutants targets (`cov-wrap.sh`,
`proc_cov.py`), then each mutant built with its selected tests' targets and run on those
tests, killers first (`gate8_select.py`, `prio.json` from run 43056231538). The report is
the study's `REPORT.md`; nothing here is a rule.
