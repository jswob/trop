//! The builder's catalogue test: the engine's codes are exactly the rows of the spec's tables.
//! Spec: L §8.1, §8.3, §8.4, §8.5; E §11.
//! Twin: none; refcore keeps its codes as strings at each raise site.
//! Invariants: reads the spec only, never a test, vector or answer key (R-93); a code added
//! to or dropped from a table fails it, as rustc's tidy check does for error codes (R-76).

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally::base::Code;

    /// The language's text, read at compile time.
    const LANGUAGE: &str = include_str!("../../../spec/LANGUAGE.md");
    /// The engine's text, read at compile time.
    const ENGINE: &str = include_str!("../../../spec/ENGINE.md");

    /// The codes of the table under the heading `heading`: the first cell of each row that
    /// starts with a code in backticks, down to the next heading.
    fn codes<'a>(text: &'a str, heading: &str) -> Vec<&'a str> {
        text.lines()
            .skip_while(|line| *line != heading)
            .skip(1)
            .take_while(|line| !line.starts_with('#'))
            .filter_map(|line| line.strip_prefix("| `"))
            .filter_map(|rest| rest.split_once('`'))
            .map(|(code, _)| code)
            .collect()
    }

    #[test]
    fn the_catalogue_is_the_spec_tables() {
        let mut spec = Vec::new();
        for heading in [
            "### 8.1 Holes",
            "### 8.3 Refused at load",
            "### 8.4 Refused per value",
            "### 8.5 Warnings",
        ] {
            let table = codes(LANGUAGE, heading);
            assert!(!table.is_empty(), "L has the table {heading}");
            spec.extend(table);
        }
        let requests = codes(ENGINE, "## 11. What the engine refuses");
        assert!(!requests.is_empty(), "E has the table of §11");
        spec.extend(requests);
        let mine: Vec<&str> = Code::ALL.iter().map(|code| code.as_str()).collect();
        assert_eq!(
            mine, spec,
            "Code::ALL against the tables of L §8.1, §8.3–§8.5 and E §11"
        );
    }
}
