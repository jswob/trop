# SURF-06 · A record row with a negative field

**A passage to write; it has no key.** Tally 2.1's writing form (CHANGES.md 2.1-27; LANGUAGE §2.3, the signed literal `slit` in a record's field).

**Story.** The bank's letter of 12 February 2026 corrects an overcharge: the household's account is credited with a correction of −250.00 DUK against the fees, spread over 3 months.

**Write:** the correction as one row of a record input, `(amount money DUK, months int)`, its amount negative as the letter states it; ask its two fields back.

**What the passage shows:** a record row's field holds a signed literal, as the grammar allows (`field = FIELD ":" slit …`); the field reads back as written. Refusing the row is an interpreter's fault, not a rule of the language (CHANGES.md 2.1-27). The text is `unit/probes/surf/SURF-06.tally`.
