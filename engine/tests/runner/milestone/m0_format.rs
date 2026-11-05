//! Milestone M0's format (DONE gate 6; `BUILD-ORDER.md` § M0: `format(parse(x)) = x` for every
//! text `checks.tsv` names, and `format` is idempotent; `MILESTONE-M0.md` row 4): each vector of
//! `m0/format.refcore` run in process. The engine's `reprint` must print refcore 2.1.3's output
//! (each file's canonical text, L §2.4, by its length and digest; the reality's `sha256` line,
//! E §3.1 step 1), and its reprint of each canonical text in place of its file must print the
//! same again (the round trip, whose expected value is the text itself). One test per corpus
//! group, and one per format fixture (`m0/format/`), each written off its canonical form in the
//! one way a rule of L §2.4 fixes. Red until M0's formatter is built (`reprint` exits 69), so
//! this target has `test = false`; it gates once M0 is tagged (`engine/tools/milestones.toml`).

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally_runner::m0::group_failures;

    /// Holds every vector of `group` to refcore's reprint and to the round trip.
    fn formats_as_refcore(group: &str) {
        let (failed, of) = group_failures(group).expect("m0/format.refcore reads");
        assert!(
            failed.is_empty(),
            "{} of {of} sets of {group} fail:\n{}",
            failed.len(),
            failed.join("\n")
        );
    }

    /// The unit probes' texts (`unit/probes/`), one set per probe.
    #[test]
    fn m0_format_probes() {
        formats_as_refcore("probes");
    }

    /// The probe sets (`U:set/…`): a probe with its scenario files.
    #[test]
    fn m0_format_probe_sets() {
        formats_as_refcore("probe-sets");
    }

    /// The 77 models of `integration/models/` (tally 2's carry of the 79; LH-15 and LH-16 by
    /// their light twins), UL-09 and UL-14 included (tally 2.1.3, #59).
    #[test]
    fn m0_format_models() {
        formats_as_refcore("models");
    }

    /// The light twins of the heavy models.
    #[test]
    fn m0_format_light() {
        formats_as_refcore("light");
    }

    /// The blind texts (`I:hardset/…`) and session 7's blind cards S1–S6.
    #[test]
    fn m0_format_blind() {
        formats_as_refcore("blind");
    }

    /// Household A's runs, with their scenarios and law files.
    #[test]
    fn m0_format_household_a() {
        formats_as_refcore("household-A");
    }

    /// Household B's runs.
    #[test]
    fn m0_format_household_b() {
        formats_as_refcore("household-B");
    }

    /// Household C's runs.
    #[test]
    fn m0_format_household_c() {
        formats_as_refcore("household-C");
    }

    /// Household D's runs.
    #[test]
    fn m0_format_household_d() {
        formats_as_refcore("household-D");
    }

    /// Household E1's joint model, with its members A, B and D as its `use` lines name them.
    #[test]
    fn m0_format_household_e1() {
        formats_as_refcore("household-E1");
    }

    /// Household E2's joint model, with its members A and D.
    #[test]
    fn m0_format_household_e2() {
        formats_as_refcore("household-E2");
    }

    /// The amendments household X: its base, its changes (`real11/c1..c7`, `v3..v5`), its cut
    /// copies and minimal texts. Loose files print by their paths as given, in full-path order,
    /// and hash in that order (tally 2.1.3, #60).
    #[test]
    fn m0_format_household_x() {
        formats_as_refcore("household-X");
    }

    /// L §2.4 Header first: `model`, `tally`, `on`, `start`, `end`, then `use` lines by name,
    /// `same` lines by left side, notes.
    #[test]
    fn m0_format_rule_header_order() {
        formats_as_refcore("format/header_order");
    }

    /// L §2.4 Sections in the kind order `source`, `unit`, `kind`, `axis`, `index`, `place`,
    /// `input`, `band`, `def`, `fill`, `move`, `ask`, `version`.
    #[test]
    fn m0_format_rule_section_kind_order() {
        formats_as_refcore("format/section_kind_order");
    }

    /// L §2.4 Within a kind, sections are sorted by id (ASCII).
    #[test]
    fn m0_format_rule_sections_by_id() {
        formats_as_refcore("format/sections_by_id");
    }

    /// L §2.4 `kind` sections print in tree order: each kind followed by its subkinds, depth
    /// first, siblings by id.
    #[test]
    fn m0_format_rule_kinds_tree_order() {
        formats_as_refcore("format/kinds_tree_order");
    }

    /// L §2.4 Versions come last, sorted by `known` date (a window by its first day), then id.
    #[test]
    fn m0_format_rule_versions_last_by_known() {
        formats_as_refcore("format/versions_last_by_known");
    }

    /// L §2.4 One blank line between sections, always, one-line sections included.
    #[test]
    fn m0_format_rule_blank_lines() {
        formats_as_refcore("format/blank_lines");
    }

    /// L §2.4 Inside a section, its clause lines and notes first, in the grammar's order
    /// (§2.3): a move's `on`, `when`, `leg`, `note`; an index's `assumed` before its notes.
    #[test]
    fn m0_format_rule_clause_order() {
        formats_as_refcore("format/clause_order");
    }

    /// L §2.4 Rules sorted by label, then by the `known` date of their `v`, a rule without `v`
    /// first.
    #[test]
    fn m0_format_rule_rules_sorted() {
        formats_as_refcore("format/rules_sorted");
    }

    /// L §2.4 Version items: entries into kinds and indexes first, then by target id, a keyless
    /// row first, then keys in index order (a list index's own order).
    #[test]
    fn m0_format_rule_version_items_sorted() {
        formats_as_refcore("format/version_items_sorted");
    }

    /// L §2.4 At one key position the wider key comes first: a key left out (tally 2.1.3, #59),
    /// then `*`, then a range before the keys it holds; a range otherwise at its first key.
    #[test]
    fn m0_format_rule_version_items_wider_key_first() {
        formats_as_refcore("format/version_items_wider_key_first");
    }

    /// L §2.4 Two items that every key leaves tied sort by their own canonical line, compared as
    /// ASCII (GD-28).
    #[test]
    fn m0_format_rule_version_items_ties() {
        formats_as_refcore("format/version_items_ties");
    }

    /// L §2.4 A `beats` list is sorted by label, with ", ".
    #[test]
    fn m0_format_rule_beats_sorted() {
        formats_as_refcore("format/beats_sorted");
    }

    /// L §2.4 Short form: a def with one rule labelled `base`, no `v`, no `when`, no own
    /// `governs`, no notes prints as `def x: T = expr at E`.
    #[test]
    fn m0_format_rule_short_form() {
        formats_as_refcore("format/short_form");
    }

    /// L §2.4 A one-line def followed by `note` lines means the long form, its one rule
    /// labelled `base`, and prints as it (tally 2.1, 2.1-4).
    #[test]
    fn m0_format_rule_short_with_notes() {
        formats_as_refcore("format/short_with_notes");
    }

    /// L §2.4 Numbers print as the shortest exact decimal, a leading `0` before the point,
    /// thousands grouped by a space, or as a reduced `a/b` when the value doesn't terminate.
    #[test]
    fn m0_format_rule_numbers() {
        formats_as_refcore("format/numbers");
    }

    /// L §2.4 A value of the unit `%` prints with it; a number before a unit is the shortest
    /// exact decimal too.
    #[test]
    fn m0_format_rule_percent_units() {
        formats_as_refcore("format/percent_units");
    }

    /// L §2.4 Expressions: one space around binary operators and after commas, none inside
    /// brackets.
    #[test]
    fn m0_format_rule_expr_spacing() {
        formats_as_refcore("format/expr_spacing");
    }

    /// L §2.4 Parentheses appear only where precedence needs them.
    #[test]
    fn m0_format_rule_parens_redundant() {
        formats_as_refcore("format/parens_redundant");
    }

    /// L §2.4 A `… to u` call used as an operand, and a converted `flows` inside a comparison
    /// (parse rule 4), are always parenthesised.
    #[test]
    fn m0_format_rule_parens_forced() {
        formats_as_refcore("format/parens_forced");
    }

    /// L §2.4 Numbers of any size print as every number does: a reduced `a/b` whose
    /// numerator, denominator or both pass 64 bits, and a decimal past 64 bits grouped by a
    /// space (G11's B4).
    #[test]
    fn m0_format_rule_numbers_of_any_size() {
        formats_as_refcore("format/numbers_of_any_size");
    }

    /// L §2.3 `sref`, §2.4 A `same` side of every `sref` form prints as written, its keys and
    /// fields kept: three names deep, a field after its keys, keys after a field (G11's B3).
    /// Each is the only `same` item of its list, so no order between sides is read: refcore
    /// sorts a side deeper than `a.x` by its parse tree (#88, `m0/format/held.tsv`).
    #[test]
    fn m0_format_rule_same_sides() {
        formats_as_refcore("format/same_sides");
    }

    /// L §2.3 `sref`, §2.4 A `same` side keyed by a keyed place (`lim[konto[ana]]`) prints as
    /// written, its items sorted by their left sides' place keys (G12's N3). Beside a row of
    /// its target and keys refcore sorts it by its parse tree (#88, `m0/format/held.tsv`).
    #[test]
    fn m0_format_rule_same_place_keys() {
        formats_as_refcore("format/same_place_keys");
    }
}
