//! An implementation of the "stringprep" algorithm defined in [RFC 3454][].
//!
//! [RFC 3454]: https://tools.ietf.org/html/rfc3454
#![warn(missing_docs)]
extern crate unicode_bidi;
extern crate unicode_normalization;
extern crate unicode_properties;

use std::borrow::Cow;
use std::fmt;
use unicode_normalization::UnicodeNormalization;
use unicode_properties::{GeneralCategoryGroup, UnicodeGeneralCategory};

mod rfc3454;
pub mod tables;

/// Describes why a string failed stringprep normalization.
#[derive(Debug)]
enum ErrorCause {
    /// Contains stringprep prohibited characters.
    ProhibitedCharacter(char),
    /// Violates stringprep rules for bidirectional text.
    ProhibitedBidirectionalText,
    /// Starts with a combining character
    StartsWithCombiningCharacter,
    /// Empty String
    EmptyString,
}

/// An error performing the stringprep algorithm.
#[derive(Debug)]
pub struct Error(ErrorCause);

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            ErrorCause::ProhibitedCharacter(c) => write!(fmt, "prohibited character `{}`", c),
            ErrorCause::ProhibitedBidirectionalText => write!(fmt, "prohibited bidirectional text"),
            ErrorCause::StartsWithCombiningCharacter => {
                write!(fmt, "starts with combining character")
            }
            ErrorCause::EmptyString => write!(fmt, "empty string"),
        }
    }
}

impl std::error::Error for Error {}

/// Prepares a string with the SASLprep profile of the stringprep algorithm.
///
/// SASLprep is defined in [RFC 4013][].
///
/// [RFC 4013]: https://tools.ietf.org/html/rfc4013
pub fn saslprep(s: &str) -> Result<Cow<'_, str>, Error> {
    // fast path for ascii text
    if s.chars()
        .all(|c| c.is_ascii() && !tables::ascii_control_character(c))
    {
        return Ok(Cow::Borrowed(s));
    }

    // 2.1 Mapping
    let mapped = s
        .chars()
        .map(|c| {
            if tables::non_ascii_space_character(c) {
                ' '
            } else {
                c
            }
        })
        .filter(|&c| !tables::commonly_mapped_to_nothing(c));

    // 2.2 Normalization
    let normalized = mapped.nfkc().collect::<String>();

    // 2.3 Prohibited Output
    let prohibited = normalized.chars().find(|&c| {
        tables::non_ascii_space_character(c) /* C.1.2 */ ||
            tables::ascii_control_character(c) /* C.2.1 */ ||
            tables::non_ascii_control_character(c) /* C.2.2 */ ||
            tables::private_use(c) /* C.3 */ ||
            tables::non_character_code_point(c) /* C.4 */ ||
            tables::surrogate_code(c) /* C.5 */ ||
            tables::inappropriate_for_plain_text(c) /* C.6 */ ||
            tables::inappropriate_for_canonical_representation(c) /* C.7 */ ||
            tables::change_display_properties_or_deprecated(c) /* C.8 */ ||
            tables::tagging_character(c) /* C.9 */
    });
    if let Some(c) = prohibited {
        return Err(Error(ErrorCause::ProhibitedCharacter(c)));
    }

    // 2.4. Bidirectional Characters
    if is_prohibited_bidirectional_text(&normalized) {
        return Err(Error(ErrorCause::ProhibitedBidirectionalText));
    }

    // 2.5 Unassigned Code Points
    let unassigned = normalized
        .chars()
        .find(|&c| tables::unassigned_code_point(c));
    if let Some(c) = unassigned {
        return Err(Error(ErrorCause::ProhibitedCharacter(c)));
    }

    Ok(Cow::Owned(normalized))
}

// RFC3454, 6. Bidirectional Characters
fn is_prohibited_bidirectional_text(s: &str) -> bool {
    if s.contains(tables::bidi_r_or_al) {
        // 2) If a string contains any RandALCat character, the string
        // MUST NOT contain any LCat character.
        if s.contains(tables::bidi_l) {
            return true;
        }

        // 3) If a string contains any RandALCat character, a RandALCat
        // character MUST be the first character of the string, and a
        // RandALCat character MUST be the last character of the string.
        if !tables::bidi_r_or_al(s.chars().next().unwrap())
            || !tables::bidi_r_or_al(s.chars().next_back().unwrap())
        {
            return true;
        }
    }

    false
}

/// Prepares a string with the Nameprep profile of the stringprep algorithm.
///
/// Nameprep is defined in [RFC 3491][].
///
/// [RFC 3491]: https://tools.ietf.org/html/rfc3491
pub fn nameprep(s: &str) -> Result<Cow<'_, str>, Error> {
    // fast path for ascii text
    if s.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-')
    {
        return Ok(Cow::Borrowed(s));
    }

    // 3. Mapping
    let mapped = s
        .chars()
        .filter(|&c| !tables::commonly_mapped_to_nothing(c))
        .flat_map(tables::case_fold_for_nfkc);

    // 4. Normalization
    let normalized = mapped.nfkc().collect::<String>();

    // 5. Prohibited Output
    let prohibited = normalized.chars().find(|&c| {
        tables::non_ascii_space_character(c) /* C.1.2 */ ||
            tables::non_ascii_control_character(c) /* C.2.2 */ ||
            tables::private_use(c) /* C.3 */ ||
            tables::non_character_code_point(c) /* C.4 */ ||
            tables::surrogate_code(c) /* C.5 */ ||
            tables::inappropriate_for_plain_text(c) /* C.6 */ ||
            tables::inappropriate_for_canonical_representation(c) /* C.7 */ ||
            tables::change_display_properties_or_deprecated(c) /* C.9 */ ||
            tables::tagging_character(c) /* C.9 */
    });
    if let Some(c) = prohibited {
        return Err(Error(ErrorCause::ProhibitedCharacter(c)));
    }

    // 6. Bidirectional Characters
    if is_prohibited_bidirectional_text(&normalized) {
        return Err(Error(ErrorCause::ProhibitedBidirectionalText));
    }

    // 7 Unassigned Code Points
    let unassigned = normalized
        .chars()
        .find(|&c| tables::unassigned_code_point(c));
    if let Some(c) = unassigned {
        return Err(Error(ErrorCause::ProhibitedCharacter(c)));
    }

    Ok(Cow::Owned(normalized))
}

/// Prepares a string with the Nodeprep profile of the stringprep algorithm.
///
/// Nameprep is defined in [RFC 3920, Appendix A][].
///
/// [RFC 3920, Appendix A]: https://tools.ietf.org/html/rfc3920#appendix-A
pub fn nodeprep(s: &str) -> Result<Cow<'_, str>, Error> {
    // fast path for common ascii text
    if s.chars()
        .all(|c| matches!(c, '['..='~' | '0'..='9' | '('..='.' | '#'..='%'))
    {
        return Ok(Cow::Borrowed(s));
    }

    // A.3. Mapping
    let mapped = s
        .chars()
        .filter(|&c| !tables::commonly_mapped_to_nothing(c))
        .flat_map(tables::case_fold_for_nfkc);

    // A.4. Normalization
    let normalized = mapped.nfkc().collect::<String>();

    // A.5. Prohibited Output
    let prohibited = normalized.chars().find(|&c| {
        tables::ascii_space_character(c) /* C.1.1 */ ||
            tables::non_ascii_space_character(c) /* C.1.2 */ ||
            tables::ascii_control_character(c) /* C.2.1 */ ||
            tables::non_ascii_control_character(c) /* C.2.2 */ ||
            tables::private_use(c) /* C.3 */ ||
            tables::non_character_code_point(c) /* C.4 */ ||
            tables::surrogate_code(c) /* C.5 */ ||
            tables::inappropriate_for_plain_text(c) /* C.6 */ ||
            tables::inappropriate_for_canonical_representation(c) /* C.7 */ ||
            tables::change_display_properties_or_deprecated(c) /* C.9 */ ||
            tables::tagging_character(c) /* C.9 */ ||
            prohibited_node_character(c)
    });
    if let Some(c) = prohibited {
        return Err(Error(ErrorCause::ProhibitedCharacter(c)));
    }

    // A.6. Bidirectional Characters
    if is_prohibited_bidirectional_text(&normalized) {
        return Err(Error(ErrorCause::ProhibitedBidirectionalText));
    }

    let unassigned = normalized
        .chars()
        .find(|&c| tables::unassigned_code_point(c));
    if let Some(c) = unassigned {
        return Err(Error(ErrorCause::ProhibitedCharacter(c)));
    }

    Ok(Cow::Owned(normalized))
}

// Additional characters not allowed in JID nodes, by RFC3920.
fn prohibited_node_character(c: char) -> bool {
    matches!(c, '"' | '&' | '\'' | '/' | ':' | '<' | '>' | '@')
}

/// Prepares a string with the Resourceprep profile of the stringprep algorithm.
///
/// Nameprep is defined in [RFC 3920, Appendix B][].
///
/// [RFC 3920, Appendix B]: https://tools.ietf.org/html/rfc3920#appendix-B
pub fn resourceprep(s: &str) -> Result<Cow<'_, str>, Error> {
    // fast path for ascii text
    if s.chars().all(|c| matches!(c, ' '..='~')) {
        return Ok(Cow::Borrowed(s));
    }

    // B.3. Mapping
    let mapped = s
        .chars()
        .filter(|&c| !tables::commonly_mapped_to_nothing(c))
        .collect::<String>();

    // B.4. Normalization
    let normalized = mapped.nfkc().collect::<String>();

    // B.5. Prohibited Output
    let prohibited = normalized.chars().find(|&c| {
        tables::non_ascii_space_character(c) /* C.1.2 */ ||
            tables::ascii_control_character(c) /* C.2.1 */ ||
            tables::non_ascii_control_character(c) /* C.2.2 */ ||
            tables::private_use(c) /* C.3 */ ||
            tables::non_character_code_point(c) /* C.4 */ ||
            tables::surrogate_code(c) /* C.5 */ ||
            tables::inappropriate_for_plain_text(c) /* C.6 */ ||
            tables::inappropriate_for_canonical_representation(c) /* C.7 */ ||
            tables::change_display_properties_or_deprecated(c) /* C.9 */ ||
            tables::tagging_character(c) /* C.9 */
    });
    if let Some(c) = prohibited {
        return Err(Error(ErrorCause::ProhibitedCharacter(c)));
    }

    // B.6. Bidirectional Characters
    if is_prohibited_bidirectional_text(&normalized) {
        return Err(Error(ErrorCause::ProhibitedBidirectionalText));
    }

    let unassigned = normalized
        .chars()
        .find(|&c| tables::unassigned_code_point(c));
    if let Some(c) = unassigned {
        return Err(Error(ErrorCause::ProhibitedCharacter(c)));
    }

    Ok(Cow::Owned(normalized))
}

/// Prepares a string according to the procedures described in Section 7 of
/// [ITU-T Recommendation X.520 (2019)](https://www.itu.int/rec/T-REC-X.520-201910-I/en).
///
/// Note that this function does _not_ remove leading, trailing, or inner
/// spaces as described in Section 7.6, because the characters needing removal
/// will vary across the matching rules and ASN.1 syntaxes used.
pub fn x520prep(s: &str, case_fold: bool) -> Result<Cow<'_, str>, Error> {
    if s.is_empty() {
        return Err(Error(ErrorCause::EmptyString));
    }
    if s.chars()
        .all(|c| matches!(c, ' '..='~') && (!case_fold || c.is_ascii_lowercase()))
    {
        return Ok(Cow::Borrowed(s));
    }

    // 1. Transcode
    // Already done because &str is enforced to be Unicode.

    // 2. Map
    let mapped = s
        .chars()
        .filter(|&c| !tables::x520_mapped_to_nothing(c))
        .map(|c| {
            if tables::x520_mapped_to_space(c) {
                ' '
            } else {
                c
            }
        });

    // 3. Normalize
    let normalized = if case_fold {
        mapped
            .flat_map(tables::case_fold_for_nfkc)
            .nfkc()
            .collect::<String>()
    } else {
        mapped.nfkc().collect::<String>()
    };

    // 4. Prohibit
    let prohibited = normalized.chars().find(
        |&c| {
            tables::unassigned_code_point(c)
                || tables::private_use(c)
                || tables::non_character_code_point(c)
                || tables::surrogate_code(c)
                || c == '\u{FFFD}'
        }, // REPLACEMENT CHARACTER
    );
    if let Some(c) = prohibited {
        return Err(Error(ErrorCause::ProhibitedCharacter(c)));
    }
    // From ITU-T Recommendation X.520, Section 7.4:
    // "The first code point of a string is prohibited from being a combining character."
    match normalized.chars().next() {
        Some(c) => {
            if c.general_category_group() == GeneralCategoryGroup::Mark {
                return Err(Error(ErrorCause::StartsWithCombiningCharacter));
            }
        }
        None => return Err(Error(ErrorCause::EmptyString)),
    }

    // 5. Check bidi
    // From ITU-T Recommendation X.520, Section 7.4:
    // "There are no bidirectional restrictions. The output string is the input string."
    // So there is nothing to do for this step.

    // 6. Insignificant Character Removal
    // Done in calling functions.

    Ok(normalized.into())
}

#[cfg(test)]
mod test {
    use super::*;

    fn assert_prohibited_character<T>(result: Result<T, Error>) {
        match result {
            Err(Error(ErrorCause::ProhibitedCharacter(_))) => (),
            _ => panic!(),
        }
    }

    fn assert_starts_with_combining_char<T>(result: Result<T, Error>) {
        match result {
            Err(Error(ErrorCause::StartsWithCombiningCharacter)) => (),
            _ => panic!(),
        }
    }

    // RFC4013, 3. Examples
    #[test]
    fn saslprep_examples() {
        assert_prohibited_character(saslprep("\u{0007}"));
    }

    #[test]
    fn nodeprep_examples() {
        assert_prohibited_character(nodeprep(" "));
        assert_prohibited_character(nodeprep("\u{00a0}"));
        assert_prohibited_character(nodeprep("foo@bar"));
    }

    #[test]
    fn resourceprep_examples() {
        assert_eq!("foo@bar", resourceprep("foo@bar").unwrap());
    }

    #[test]
    fn x520prep_examples() {
        assert_eq!(x520prep("foo@bar", true).unwrap(), "foo@bar");
        assert_eq!(
            x520prep("J.\u{FE00} \u{9}W. \u{B}wuz h\u{0115}re", false).unwrap(),
            "J.  W.  wuz h\u{0115}re"
        );
        assert_eq!(
            x520prep("J.\u{FE00} \u{9}W. \u{B}wuz h\u{0115}re", true).unwrap(),
            "j.  w.  wuz h\u{0115}re"
        );
        assert_eq!(x520prep("UPPERCASED", true).unwrap(), "uppercased");
        assert_starts_with_combining_char(x520prep("\u{0306}hello", true));
    }

    #[test]
    fn x520prep_normalizes_after_case_folding() {
        // X.520 (2019), 7.2 then 7.3: B.2 case-folding precedes Form KC.
        for (input, expected) in [
            ("\u{2168}", "ix"),
            ("\u{ff21}", "a"),
            ("A\u{030a}", "\u{00e5}"),
        ] {
            assert_eq!(x520prep(input, true).unwrap(), expected);
        }
        assert_eq!(x520prep("\u{2168}", false).unwrap(), "IX");
        assert_eq!(x520prep("A\u{030a}", false).unwrap(), "\u{00c5}");
    }

    #[test]
    fn x520prep_prohibits_empty_after_mapping() {
        // X.520 (2019), 7.4 judges the string produced by mapping/normalization.
        for case_fold in [false, true] {
            for input in ["", "\u{00ad}", "\u{034f}", "\u{0001}"] {
                assert!(
                    matches!(
                        x520prep(input, case_fold),
                        Err(Error(ErrorCause::EmptyString))
                    ),
                    "{:?} must not prepare to an empty string",
                    input
                );
            }
        }
    }

    #[test]
    fn x520prep_prohibits_initial_marks_after_mapping() {
        for case_fold in [false, true] {
            assert_starts_with_combining_char(x520prep("\u{00ad}\u{0306}hello", case_fold));
        }
    }

    #[test]
    fn x520prep_accepts_a_removed_initial_mark() {
        // The first remaining character, rather than the removed Mark, is judged.
        for case_fold in [false, true] {
            assert_eq!(x520prep("\u{034f}hello", case_fold).unwrap(), "hello");
        }
    }

    #[test]
    fn x520prep_leaves_insignificant_space_removal_to_the_matching_rule() {
        assert_eq!(x520prep("\u{00ad}  X  ", true).unwrap(), "  x  ");
        assert_eq!(x520prep("  X  ", false).unwrap(), "  X  ");
    }

    #[test]
    fn x520prep_maps_control_function_characters_to_nothing() {
        // X.520 (2019), 7.2 includes control-function characters (Cf), not only Cc.
        for case_fold in [false, true] {
            for control in ['\u{200e}', '\u{200f}', '\u{202e}', '\u{2060}', '\u{feff}'] {
                let input = format!("a{}b", control);
                assert_eq!(x520prep(&input, case_fold).unwrap(), "ab");
            }
            assert!(matches!(
                x520prep("\u{200e}", case_fold),
                Err(Error(ErrorCause::EmptyString))
            ));
            assert_starts_with_combining_char(x520prep("\u{200e}\u{0306}hello", case_fold));
        }
    }

    #[test]
    fn x520prep_preserves_the_control_whitespace_exceptions() {
        for case_fold in [false, true] {
            for space in [
                '\u{0009}', '\u{000a}', '\u{000b}', '\u{000c}', '\u{000d}', '\u{0085}',
            ] {
                let input = format!("a{}b", space);
                assert_eq!(x520prep(&input, case_fold).unwrap(), "a b");
            }
            // Other categories are not broadly removed along with control-function characters.
            assert_prohibited_character(x520prep("a\u{e000}b", case_fold));
            assert_prohibited_character(x520prep("a\u{0221}b", case_fold));
            assert_eq!(x520prep("a\u{2000}b", case_fold).unwrap(), "a b");
        }
    }

    #[test]
    fn ascii_optimisations() {
        if let Cow::Owned(_) = nodeprep("nodepart").unwrap() {
            panic!("“nodepart” should get optimised as ASCII");
        }
        if let Cow::Owned(_) = nameprep("domainpart.example").unwrap() {
            panic!("“domainpart.example” should get optimised as ASCII");
        }
        if let Cow::Owned(_) = resourceprep("resourcepart").unwrap() {
            panic!("“resourcepart” should get optimised as ASCII");
        }
    }
}
