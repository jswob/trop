//! Milestone M4's constructs on the engine (issue #252; `MILESTONE-M4.md` row 2): small
//! invented texts that pin the corners of each construct BUILD-ORDER § M4 builds, beyond the
//! 77 checks, each ask refcore's whole output in `m4/vectors.refcore`, the engine's held to
//! it byte for byte. One test per construct (`constructs-<construct>` in `m4/vectors.tsv`),
//! so a construct's corners turn green with the group that builds it; every failure of a
//! group is collected before it fails.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use tally_runner::engine::run_args;
    use tally_runner::held::is_held;
    use tally_runner::kernel::{difference, vectors};
    use tally_runner::m4::VECTORS;

    /// The groups this target tests, one `#[test]` each.
    const GROUPS: [&str; 14] = [
        "constructs-pins",
        "constructs-accrue",
        "constructs-round",
        "constructs-logic",
        "constructs-aggregates",
        "constructs-argm",
        "constructs-beats",
        "constructs-fx",
        "constructs-cal",
        "constructs-paper",
        "constructs-keydiff",
        "constructs-ratepin",
        "constructs-tuple",
        "constructs-calrow",
    ];

    /// Runs every vector of `group` on the engine and holds it to refcore's run.
    fn group_holds(group: &str) {
        let all = vectors(VECTORS).expect("m4/vectors.refcore reads");
        let mine: Vec<_> = all.iter().filter(|v| v.group == group).collect();
        assert!(!mine.is_empty(), "the group {group} has vectors");
        let failed: Vec<String> = mine
            .iter()
            .filter(|v| !is_held("m4_constructs", &v.id))
            .filter_map(|v| {
                difference(&v.refcore, &run_args(&v.args, &v.files))
                    .map(|d| format!("{}: {d}", v.id))
            })
            .collect();
        assert!(
            failed.is_empty(),
            "{} of {} vectors of {group} differ from refcore:\n{}",
            failed.len(),
            mine.len(),
            failed.join("\n")
        );
    }

    #[test]
    fn m4_constructs_every_group_is_tested() {
        let named: BTreeSet<String> = vectors(VECTORS)
            .expect("m4/vectors.refcore reads")
            .into_iter()
            .map(|v| v.group)
            .filter(|g| g.starts_with("constructs-"))
            .collect();
        let tested: BTreeSet<String> = GROUPS.iter().map(|g| (*g).to_owned()).collect();
        assert_eq!(
            named, tested,
            "each constructs- group of m4/vectors.tsv has its test here"
        );
    }

    /// `m4/constructs/pins/pins.tally`: an `as if` pin keyed by a def's own key, by an ask's
    /// `for` variable and by an aggregate's bound variable; a pin at another key than the one
    /// read; the least t such that c with the pin's value the bound t, and its empty key
    /// (`RUN-EMPTY`); one def under two pin sets in one line; a pin key that is a hole; a `with`
    /// pin over a def's own pin; `explain` under the pin; and in a def of two keys, a pin at
    /// both keys or one, its value fixed or read at the def's keys, the form UL-14 and C-04
    /// stop on today (`b_def`, `b_lit`, `b_mix`) (L §5.11 `as if`, §8.2 `pinned:`).
    #[test]
    fn m4_constructs_pins() {
        group_holds("constructs-pins");
    }

    /// `m4/constructs/accrue/accrue.tally`: `accrue` under act365 for a day and a month,
    /// `simple`, `effective` over a whole year, `nominal 365` over a day; b before a; a rate
    /// that is a hole; `dcf` both ways; a rate read at its day across a change, and their sum
    /// (L §3.3).
    #[test]
    fn m4_constructs_accrue() {
        group_holds("constructs-accrue");
    }

    /// `m4/constructs/round/round.tally`: each of the seven modes (`half-down` among them) at a
    /// tie and off it, both signs; a step of 0.05 PLN, a percent step, `minor(EUR)` and a
    /// whole count; the mode as an axis's key, pinned, and on a negative amount; and
    /// `round_step_unit.tally`, a num PLN rounded by a num step and stored as money, refused
    /// whole (L §7.2, §8.3 `LOAD-UNIT`).
    #[test]
    fn m4_constructs_round() {
        group_holds("constructs-round");
    }

    /// `m4/constructs/logic/logic.tally`: `and`, `or` and `not` over true, false and a hole
    /// (Kleene's tables); `if` on a hole and on a branch not taken; `stop`, and `stop` under
    /// `not`; `none` under `and` and `or` (`RUN-NONE`, refcore's line, pick P3) and its test
    /// (L §5.11 Logic).
    #[test]
    fn m4_constructs_logic() {
        group_holds("constructs-logic");
    }

    /// `m4/constructs/aggregates/agg.tally`: `any`, `all` and `count` over a hole; `sum` over a
    /// hole and over a term known to be 0; `min`, `max`, `first` and `last` with a hole; each
    /// over no member; `mean` and `share`, with a hole (L §5.11 Aggregates).
    #[test]
    fn m4_constructs_aggregates() {
        group_holds("constructs-aggregates");
    }

    /// `m4/constructs/argm/argm.tally`: `argmax` and `argmin` by position and by a rank, of a
    /// tuple, over booleans, dates and keys, a tie with no `ties by` (`RUN-TIE`) and over
    /// things; and `first_things.tally`, `first` over things with no `ties by`, refused whole
    /// (L §5.11 Aggregates, §5.7).
    #[test]
    fn m4_constructs_argm() {
        group_holds("constructs-argm");
    }

    /// `m4/constructs/beats/beats.tally`: a `beats` list, a rule left off it, no rule
    /// (`RUN-NO-RULE`), two rules equal (`RUN-CONFLICT`); a wide `given` against a narrower
    /// one at a later law date, which wins; `explain` through them (L §4.6, §5.11).
    #[test]
    fn m4_constructs_beats() {
        group_holds("constructs-beats");
    }

    /// `m4/constructs/fx/fx.tally`: a rate PLN/EUR and its inverse applied each way and divided
    /// back, a rate plus a margin and times a number, a rate pinned by a bare literal both ways,
    /// rates multiplied; and the two refusals whole, a rate applied to the wrong currency and
    /// two currencies added (L §3.2, §8.3 `LOAD-UNIT`).
    #[test]
    fn m4_constructs_fx() {
        group_holds("constructs-fx");
    }

    /// `m4/constructs/cal/cal.tally`: calendars per country, weekends Saturday–Sunday,
    /// Friday–Saturday and Sunday alone; `add_business` forward and back, through a key's
    /// country, `last_business`, a holiday `from` a year, an `added` day, the coverage hole;
    /// `schedule` under `following`, `preceding`, `modified_following` and `unadjusted`, under
    /// the other weekend, and in a month without the day (L §4.2).
    #[test]
    fn m4_constructs_cal() {
        group_holds("constructs-cal");
    }

    /// `m4/paper/`: the paper check's probes (`tmp/m4/paper-check.md`), one vector per ask, a
    /// file refused whole as one; its decided cells D-1 to D-26 among them; #253's silent
    /// items on refcore's bytes (pick P1), `cal__register` and the row meeting #250 row 6 too
    /// (picks P2, P4).
    #[test]
    fn m4_constructs_paper() {
        group_holds("constructs-paper");
    }

    /// The vectors that wait for a `check` of two keys or periods to print its `difference`
    /// (`m4/constructs` and `m4/paper`, `for` = `keydiff`).
    #[test]
    fn m4_constructs_keydiff() {
        group_holds("constructs-keydiff");
    }

    /// The vectors that wait for a bare-literal pin of a rate to take the rate's unit
    /// (`for` = `ratepin`).
    #[test]
    fn m4_constructs_ratepin() {
        group_holds("constructs-ratepin");
    }

    /// The vectors that wait for `argmax`'s and `argmin`'s tuple result (`for` = `tuple`).
    #[test]
    fn m4_constructs_tuple() {
        group_holds("constructs-tuple");
    }

    /// #253 item 3's vectors, a business-day count under the row governing d (pick P1: the
    /// engine follows the text): held for #253 in `held.tsv`, their spec answers filled from
    /// the engine's run after the builder's group (`for` = `calrow`).
    #[test]
    fn m4_constructs_calrow() {
        group_holds("constructs-calrow");
    }
}
