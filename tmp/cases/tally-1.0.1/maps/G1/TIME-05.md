# Map · TIME-05 · `probes/time/TIME-05.tally` · expressed

step 1 · (a) window end: same-numbered day, clipped = 2023-02-28 · lines 8–9, 16, 18–24, 58–59, 66 · L §4.2 (`add_months`: the same day number, clamped to the month's last day) · same; short-term = sold ≤ end, true (the end day is inside)
step 2 · (b) clock end: end of the year of acquisition + 5 years = 2024-12-31 · lines 10–11, 14, 26–35, 60–61, 64 · L §4.2 (`year_of`, `end_of` is the first day after the period, so − 1 day is 2019-12-31; `add_years`) · same; exempt = sold > end, false; first exempt day = end + 1, 2025-01-01
step 3 · (c) deadline: from 2024-04-30, the last day of the final month = 2024-10-31 · lines 12–13, 15, 41–47, 62–63, 65 · L §4.2 (`month_of`, month + int moves by months, `end_of` − 1 day) · same; in time = reported ≤ deadline, true
step 4 · (c) day-for-day, for contrast = 2024-10-30 · lines 37–39 · L §4.2 (`add_months` from the day of death) · same
ask returns: value ((2023-02-28, true), (2024-12-31, false, 2025-01-01), (2024-10-31, true, 2024-10-30)) · three asks
checks: parsecheck ok · loadcheck ok
