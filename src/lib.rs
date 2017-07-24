//! An implementation of the "stringprep" algorithm defined in [RFC 3454][].
//!
//! [RFC 3454]: https://tools.ietf.org/html/rfc3454
#![doc(html_root_url="https://docs.rs/stringprep/0.1.1")]
#![warn(missing_docs)]
extern crate unicode_bidi;
extern crate unicode_normalization;

use std::ascii::AsciiExt;
use std::borrow::Cow;
use std::error;
use std::fmt;
use unicode_normalization::UnicodeNormalization;

pub mod tables;

/// Describes why a string failed stringprep normalization.
#[derive(Debug)]
pub enum ErrorCause {
    /// Contains stringprep prohibited characters.
    ProhibitedCharacter(char),
    /// Violates stringprep rules for bidirectional text.
    ProhibitedBidirectionalText,
}

/// An error performing the stringprep algorithm.
#[derive(Debug)]
pub struct Error(pub ErrorCause);

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            ErrorCause::ProhibitedCharacter(c) => write!(fmt, "prohibited character `{}`", c),
            ErrorCause::ProhibitedBidirectionalText => write!(fmt, "prohibited bidirectional text"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "error performing stringprep algorithm"
    }
}

/// Prepares a string with the SASLprep profile of the stringprep algorithm.
///
/// SASLprep is defined in [RFC 4013][].
///
/// [RFC 4013]: https://tools.ietf.org/html/rfc4013
pub fn saslprep<'a>(s: &'a str) -> Result<Cow<'a, str>, Error> {
    // fast path for ascii text
    if s.chars()
           .all(|c| c.is_ascii() && !tables::ascii_control_character(c)) {
        return Ok(Cow::Borrowed(s));
    }

    // 2.1 Mapping
    let mapped = s.chars()
        .map(|c| if tables::non_ascii_space_character(c) {
                 ' '
             } else {
                 c
             })
        .filter(|&c| !tables::commonly_mapped_to_nothing(c));

    // 2.2 Normalization
    let normalized = mapped.nfkc().collect::<String>();

    // 2.3 Prohibited Output
    let prohibited = normalized
        .chars()
        .filter(|&c| {
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
        })
        .next();
    if let Some(c) = prohibited {
        return Err(Error(ErrorCause::ProhibitedCharacter(c)));
    }

    // RFC3454, 6. Bidirectional Characters
    if normalized.contains(tables::bidi_r_or_al) {
        // 2) If a string contains any RandALCat character, the string
        // MUST NOT contain any LCat character.
        if normalized.contains(tables::bidi_l) {
            return Err(Error(ErrorCause::ProhibitedBidirectionalText));
        }

        // 3) If a string contains any RandALCat character, a RandALCat
        // character MUST be the first character of the string, and a
        // RandALCat character MUST be the last character of the string.
        if !tables::bidi_r_or_al(normalized.chars().next().unwrap()) ||
           !tables::bidi_r_or_al(normalized.chars().next_back().unwrap()) {
            return Err(Error(ErrorCause::ProhibitedBidirectionalText));
        }
    }

    // 2.5 Unassigned Code Points
    // TODO: Reject unassigned code points.

    Ok(Cow::Owned(normalized))
}

#[cfg(test)]
mod test {
    use super::*;

    // RFC4013, 3. Examples
    #[test]
    fn saslprep_examples() {
        assert_eq!(saslprep("I\u{00AD}X").unwrap(), "IX");
        assert_eq!(saslprep("user").unwrap(), "user");
        assert_eq!(saslprep("USER").unwrap(), "USER");
        assert_eq!(saslprep("\u{00AA}").unwrap(), "a");
        assert_eq!(saslprep("\u{2168}").unwrap(), "IX");
        assert_prohibited_character(saslprep("\u{0007}"));
        assert_prohibited_bidirectional_text(saslprep("\u{0627}\u{0031}"));
    }

    fn assert_prohibited_character<T>(result: Result<T, Error>) {
        match result {
            Err(Error(ErrorCause::ProhibitedCharacter(_))) => (),
            _ => assert!(false)
        }
    }

    fn assert_prohibited_bidirectional_text<T>(result: Result<T, Error>) {
        match result {
            Err(Error(ErrorCause::ProhibitedBidirectionalText)) => (),
            _ => assert!(false)
        }
    }
}
