# SURF-07 · A day key where a date is expected

**A passage to write; it has no key.** Tally 2.1's writing form (CHANGES.md 2.1-5; LANGUAGE §3.1, "A day key where a date is expected").

**Story.** The fund opens for subscriptions on 10 March 2026. For each of the days 9, 10 and 11 March, the household asks the first date on or after that day when it can subscribe: the later of the opening and the day.

**Write:** the days as an index of days, the opening as a date input, and the figure written both ways: `max(opens, k)`, with the day key standing where a date is expected, and `max(opens, start_of(k))`, the conversion written out.

**What the passage shows:** a key of a day index where a date is expected means `start_of(k)`: a day has one date, so the two forms give the same dates. A key of a month or a year index doesn't convert, since its start or end is a choice, and is written. The text is `unit/probes/surf/SURF-07.tally`.
