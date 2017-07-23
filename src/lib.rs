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

#[derive(Debug)]
enum ErrorCause {
    ProhibitedCharacter(char),
    ProhibitedBidirectionalText,
}

/// An error performing the stringprep algorithm.
#[derive(Debug)]
pub struct Error(ErrorCause);

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

    let mapped = s.chars()
        .map(|c| if tables::non_ascii_space_character(c) {
                 ' '
             } else {
                 c
             })
        .filter(|&c| !tables::commonly_mapped_to_nothing(c));

    let normalized = mapped.nfkc().collect::<String>();

    let prohibited = normalized
        .chars()
        .filter(|&c| {
            tables::non_ascii_space_character(c) || tables::ascii_control_character(c) ||
            tables::non_ascii_control_character(c) || tables::private_use(c) ||
            tables::non_character_code_point(c) ||
            tables::surrogate_code(c) || tables::inappropriate_for_plain_text(c) ||
            tables::inappropriate_for_canonical_representation(c) ||
            tables::change_display_properties_or_deprecated(c) ||
            tables::tagging_character(c)
        })
        .next();
    if let Some(c) = prohibited {
        return Err(Error(ErrorCause::ProhibitedCharacter(c)));
    }

    if normalized.contains(tables::bidi_r_or_al) {
        if normalized.contains(tables::bidi_l) {
            return Err(Error(ErrorCause::ProhibitedBidirectionalText));
        }

        if !tables::bidi_r_or_al(normalized.chars().next().unwrap()) ||
           !tables::bidi_r_or_al(normalized.chars().next_back().unwrap()) {
            return Err(Error(ErrorCause::ProhibitedBidirectionalText));
        }
    }

    Ok(Cow::Owned(normalized))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn saslprep_examples() {
        assert_eq!(saslprep("I\u{00AD}X").unwrap(), "IX");
        assert_eq!(saslprep("user").unwrap(), "user");
        assert_eq!(saslprep("USER").unwrap(), "USER");
        assert_eq!(saslprep("\u{00AA}").unwrap(), "a");
        assert_eq!(saslprep("\u{2168}").unwrap(), "IX");
        assert!(saslprep("\u{0007}").is_err());
        assert!(saslprep("\u{0627}\u{0031}").is_err());
    }
}
