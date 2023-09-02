//! Character Tables
use unicode_bidi::{bidi_class, BidiClass};
use std::cmp::Ordering;
use std::str::Chars;

use super::rfc3454;

/// A.1 Unassigned code points in Unicode 3.2
pub fn unassigned_code_point(c: char) -> bool {
    rfc3454::A_1
        .binary_search_by(|&(start, end)| if start > c {
            Ordering::Greater
        } else if end < c {
            Ordering::Less
        } else {
            Ordering::Equal
        })
        .is_ok()
}

/// B.1 Commonly mapped to nothing
pub fn commonly_mapped_to_nothing(c: char) -> bool {
    match c {
        '\u{00AD}' | '\u{034F}' | '\u{1806}' | '\u{180B}' | '\u{180C}' | '\u{180D}' |
        '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{2060}' | '\u{FE00}' | '\u{FE01}' |
        '\u{FE02}' | '\u{FE03}' | '\u{FE04}' | '\u{FE05}' | '\u{FE06}' | '\u{FE07}' |
        '\u{FE08}' | '\u{FE09}' | '\u{FE0A}' | '\u{FE0B}' | '\u{FE0C}' | '\u{FE0D}' |
        '\u{FE0E}' | '\u{FE0F}' | '\u{FEFF}' => true,
        _ => false,
    }
}

/// B.2 Mapping for case-folding used with NFKC.
pub fn case_fold_for_nfkc(c: char) -> CaseFoldForNfkc {
    let inner = match rfc3454::B_2.binary_search_by_key(&c, |e| e.0) {
        Ok(idx) => FoldInner::Chars(rfc3454::B_2[idx].1.chars()),
        Err(_) => FoldInner::Char(Some(c)),
    };
    CaseFoldForNfkc(inner)
}

enum FoldInner {
    Chars(Chars<'static>),
    Char(Option<char>),
}

/// The iterator returned by `case_fold_for_nfkc`.
pub struct CaseFoldForNfkc(FoldInner);

impl Iterator for CaseFoldForNfkc {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        match self.0 {
            FoldInner::Chars(ref mut it) => it.next(),
            FoldInner::Char(ref mut ch) => ch.take(),
        }
    }
}

/// C.1.1 ASCII space characters
pub fn ascii_space_character(c: char) -> bool {
    c == ' '
}

/// C.1.2 Non-ASCII space characters
pub fn non_ascii_space_character(c: char) -> bool {
    match c {
        '\u{00A0}' | '\u{1680}' | '\u{2000}' | '\u{2001}' | '\u{2002}' | '\u{2003}' |
        '\u{2004}' | '\u{2005}' | '\u{2006}' | '\u{2007}' | '\u{2008}' | '\u{2009}' |
        '\u{200A}' | '\u{200B}' | '\u{202F}' | '\u{205F}' | '\u{3000}' => true,
        _ => false,
    }
}

/// C.2.1 ASCII control characters
pub fn ascii_control_character(c: char) -> bool {
    match c {
        '\u{0000}'..='\u{001F}' |
        '\u{007F}' => true,
        _ => false,
    }
}

/// C.2.2 Non-ASCII control characters
pub fn non_ascii_control_character(c: char) -> bool {
    match c {
        '\u{0080}'..='\u{009F}' |
        '\u{06DD}' |
        '\u{070F}' |
        '\u{180E}' |
        '\u{200C}' |
        '\u{200D}' |
        '\u{2028}' |
        '\u{2029}' |
        '\u{2060}' |
        '\u{2061}' |
        '\u{2062}' |
        '\u{2063}' |
        '\u{206A}'..='\u{206F}' |
        '\u{FEFF}' |
        '\u{FFF9}'..='\u{FFFC}' |
        '\u{1D173}'..='\u{1D17A}' => true,
        _ => false,
    }
}

/// C.3 Private use
pub fn private_use(c: char) -> bool {
    match c {
        '\u{E000}'..='\u{F8FF}' |
        '\u{F0000}'..='\u{FFFFD}' |
        '\u{100000}'..='\u{10FFFD}' => true,
        _ => false,
    }
}

/// C.4 Non-character code points
pub fn non_character_code_point(c: char) -> bool {
    match c {
        '\u{FDD0}'..='\u{FDEF}' |
        '\u{FFFE}'..='\u{FFFF}' |
        '\u{1FFFE}'..='\u{1FFFF}' |
        '\u{2FFFE}'..='\u{2FFFF}' |
        '\u{3FFFE}'..='\u{3FFFF}' |
        '\u{4FFFE}'..='\u{4FFFF}' |
        '\u{5FFFE}'..='\u{5FFFF}' |
        '\u{6FFFE}'..='\u{6FFFF}' |
        '\u{7FFFE}'..='\u{7FFFF}' |
        '\u{8FFFE}'..='\u{8FFFF}' |
        '\u{9FFFE}'..='\u{9FFFF}' |
        '\u{AFFFE}'..='\u{AFFFF}' |
        '\u{BFFFE}'..='\u{BFFFF}' |
        '\u{CFFFE}'..='\u{CFFFF}' |
        '\u{DFFFE}'..='\u{DFFFF}' |
        '\u{EFFFE}'..='\u{EFFFF}' |
        '\u{FFFFE}'..='\u{FFFFF}' |
        '\u{10FFFE}'..='\u{10FFFF}' => true,
        _ => false,
    }
}

/// C.5 Surrogate codes
pub fn surrogate_code(c: char) -> bool {
    match c {
        // forbidden by rust
        /*'\u{D800}'..='\u{DFFF}' => true,*/
        _ => false,
    }
}

/// C.6 Inappropriate for plain text
pub fn inappropriate_for_plain_text(c: char) -> bool {
    match c {
        '\u{FFF9}' | '\u{FFFA}' | '\u{FFFB}' | '\u{FFFC}' | '\u{FFFD}' => true,
        _ => false,
    }
}

/// C.7 Inappropriate for canonical representation
pub fn inappropriate_for_canonical_representation(c: char) -> bool {
    match c {
        '\u{2FF0}'..='\u{2FFB}' => true,
        _ => false,
    }
}

/// C.8 Change display properties or are deprecated
pub fn change_display_properties_or_deprecated(c: char) -> bool {
    match c {
        '\u{0340}' | '\u{0341}' | '\u{200E}' | '\u{200F}' | '\u{202A}' | '\u{202B}' |
        '\u{202C}' | '\u{202D}' | '\u{202E}' | '\u{206A}' | '\u{206B}' | '\u{206C}' |
        '\u{206D}' | '\u{206E}' | '\u{206F}' => true,
        _ => false,
    }
}

/// C.9 Tagging characters
pub fn tagging_character(c: char) -> bool {
    match c {
        '\u{E0001}' |
        '\u{E0020}'..='\u{E007F}' => true,
        _ => false,
    }
}

/// D.1 Characters with bidirectional property "R" or "AL"
pub fn bidi_r_or_al(c: char) -> bool {
    match bidi_class(c) {
        BidiClass::R | BidiClass::AL => true,
        _ => false,
    }
}

/// D.2 Characters with bidirectional property "L"
pub fn bidi_l(c: char) -> bool {
    match bidi_class(c) {
        BidiClass::L => true,
        _ => false,
    }
}

/// The table of all Unicode combining characters, meaning all characters under
/// the Unicode "Mark" (M) category.
pub fn unicode_mark_category(c: char) -> bool {
    match c {
        '\u{300}'..='\u{36F}'
        | '\u{483}'..='\u{489}'
        | '\u{591}'..='\u{5BD}'
        | '\u{5BF}'
        | '\u{5C1}'..='\u{5C2}'
        | '\u{5C4}'..='\u{5C5}'
        | '\u{5C7}'
        | '\u{610}'..='\u{61A}'
        | '\u{64B}'..='\u{65F}'
        | '\u{670}'
        | '\u{6D6}'..='\u{6DC}'
        | '\u{6DF}'..='\u{6E4}'
        | '\u{6E7}'..='\u{6E8}'
        | '\u{6EA}'..='\u{6ED}'
        | '\u{711}'
        | '\u{730}'..='\u{74A}'
        | '\u{7A6}'..='\u{7B0}'
        | '\u{7EB}'..='\u{7F3}'
        | '\u{7FD}'
        | '\u{816}'..='\u{819}'
        | '\u{81B}'..='\u{823}'
        | '\u{825}'..='\u{827}'
        | '\u{829}'..='\u{82D}'
        | '\u{859}'..='\u{85B}'
        | '\u{898}'..='\u{89F}'
        | '\u{8CA}'..='\u{8E1}'
        | '\u{8E3}'..='\u{903}'
        | '\u{93A}'..='\u{93C}'
        | '\u{93E}'..='\u{94F}'
        | '\u{951}'..='\u{957}'
        | '\u{962}'..='\u{963}'
        | '\u{981}'..='\u{983}'
        | '\u{9BC}'
        | '\u{9BE}'..='\u{9C4}'
        | '\u{9C7}'..='\u{9C8}'
        | '\u{9CB}'..='\u{9CD}'
        | '\u{9D7}'
        | '\u{9E2}'..='\u{9E3}'
        | '\u{9FE}'
        | '\u{A01}'..='\u{A03}'
        | '\u{A3C}'
        | '\u{A3E}'..='\u{A42}'
        | '\u{A47}'..='\u{A48}'
        | '\u{A4B}'..='\u{A4D}'
        | '\u{A51}'
        | '\u{A70}'..='\u{A71}'
        | '\u{A75}'
        | '\u{A81}'..='\u{A83}'
        | '\u{ABC}'
        | '\u{ABE}'..='\u{AC5}'
        | '\u{AC7}'..='\u{AC9}'
        | '\u{ACB}'..='\u{ACD}'
        | '\u{AE2}'..='\u{AE3}'
        | '\u{AFA}'..='\u{AFF}'
        | '\u{B01}'..='\u{B03}'
        | '\u{B3C}'
        | '\u{B3E}'..='\u{B44}'
        | '\u{B47}'..='\u{B48}'
        | '\u{B4B}'..='\u{B4D}'
        | '\u{B55}'..='\u{B57}'
        | '\u{B62}'..='\u{B63}'
        | '\u{B82}'
        | '\u{BBE}'..='\u{BC2}'
        | '\u{BC6}'..='\u{BC8}'
        | '\u{BCA}'..='\u{BCD}'
        | '\u{BD7}'
        | '\u{C00}'..='\u{C04}'
        | '\u{C3C}'
        | '\u{C3E}'..='\u{C44}'
        | '\u{C46}'..='\u{C48}'
        | '\u{C4A}'..='\u{C4D}'
        | '\u{C55}'..='\u{C56}'
        | '\u{C62}'..='\u{C63}'
        | '\u{C81}'..='\u{C83}'
        | '\u{CBC}'
        | '\u{CBE}'..='\u{CC4}'
        | '\u{CC6}'..='\u{CC8}'
        | '\u{CCA}'..='\u{CCD}'
        | '\u{CD5}'..='\u{CD6}'
        | '\u{CE2}'..='\u{CE3}'
        | '\u{CF3}'
        | '\u{D00}'..='\u{D03}'
        | '\u{D3B}'..='\u{D3C}'
        | '\u{D3E}'..='\u{D44}'
        | '\u{D46}'..='\u{D48}'
        | '\u{D4A}'..='\u{D4D}'
        | '\u{D57}'
        | '\u{D62}'..='\u{D63}'
        | '\u{D81}'..='\u{D83}'
        | '\u{DCA}'
        | '\u{DCF}'..='\u{DD4}'
        | '\u{DD6}'
        | '\u{DD8}'..='\u{DDF}'
        | '\u{DF2}'..='\u{DF3}'
        | '\u{E31}'
        | '\u{E34}'..='\u{E3A}'
        | '\u{E47}'..='\u{E4E}'
        | '\u{EB1}'
        | '\u{EB4}'..='\u{EBC}'
        | '\u{EC8}'..='\u{ECE}'
        | '\u{F18}'..='\u{F19}'
        | '\u{F35}'
        | '\u{F37}'
        | '\u{F39}'
        | '\u{F3E}'..='\u{F3F}'
        | '\u{F71}'..='\u{F84}'
        | '\u{F86}'..='\u{F87}'
        | '\u{F8D}'..='\u{F97}'
        | '\u{F99}'..='\u{FBC}'
        | '\u{FC6}'
        | '\u{102B}'..='\u{103E}'
        | '\u{1056}'..='\u{1059}'
        | '\u{105E}'..='\u{1060}'
        | '\u{1062}'..='\u{1064}'
        | '\u{1067}'..='\u{106D}'
        | '\u{1071}'..='\u{1074}'
        | '\u{1082}'..='\u{108D}'
        | '\u{108F}'
        | '\u{109A}'..='\u{109D}'
        | '\u{135D}'..='\u{135F}'
        | '\u{1712}'..='\u{1715}'
        | '\u{1732}'..='\u{1734}'
        | '\u{1752}'..='\u{1753}'
        | '\u{1772}'..='\u{1773}'
        | '\u{17B4}'..='\u{17D3}'
        | '\u{17DD}'
        | '\u{180B}'..='\u{180D}'
        | '\u{180F}'
        | '\u{1885}'..='\u{1886}'
        | '\u{18A9}'
        | '\u{1920}'..='\u{192B}'
        | '\u{1930}'..='\u{193B}'
        | '\u{1A17}'..='\u{1A1B}'
        | '\u{1A55}'..='\u{1A5E}'
        | '\u{1A60}'..='\u{1A7C}'
        | '\u{1A7F}'
        | '\u{1AB0}'..='\u{1ACE}'
        | '\u{1B00}'..='\u{1B04}'
        | '\u{1B34}'..='\u{1B44}'
        | '\u{1B6B}'..='\u{1B73}'
        | '\u{1B80}'..='\u{1B82}'
        | '\u{1BA1}'..='\u{1BAD}'
        | '\u{1BE6}'..='\u{1BF3}'
        | '\u{1C24}'..='\u{1C37}'
        | '\u{1CD0}'..='\u{1CD2}'
        | '\u{1CD4}'..='\u{1CE8}'
        | '\u{1CED}'
        | '\u{1CF4}'
        | '\u{1CF7}'..='\u{1CF9}'
        | '\u{1DC0}'..='\u{1DFF}'
        | '\u{20D0}'..='\u{20F0}'
        | '\u{2CEF}'..='\u{2CF1}'
        | '\u{2D7F}'
        | '\u{2DE0}'..='\u{2DFF}'
        | '\u{302A}'..='\u{302F}'
        | '\u{3099}'..='\u{309A}'
        | '\u{A66F}'..='\u{A672}'
        | '\u{A674}'..='\u{A67D}'
        | '\u{A69E}'..='\u{A69F}'
        | '\u{A6F0}'..='\u{A6F1}'
        | '\u{A802}'
        | '\u{A806}'
        | '\u{A80B}'
        | '\u{A823}'..='\u{A827}'
        | '\u{A82C}'
        | '\u{A880}'..='\u{A881}'
        | '\u{A8B4}'..='\u{A8C5}'
        | '\u{A8E0}'..='\u{A8F1}'
        | '\u{A8FF}'
        | '\u{A926}'..='\u{A92D}'
        | '\u{A947}'..='\u{A953}'
        | '\u{A980}'..='\u{A983}'
        | '\u{A9B3}'..='\u{A9C0}'
        | '\u{A9E5}'
        | '\u{AA29}'..='\u{AA36}'
        | '\u{AA43}'
        | '\u{AA4C}'..='\u{AA4D}'
        | '\u{AA7B}'..='\u{AA7D}'
        | '\u{AAB0}'
        | '\u{AAB2}'..='\u{AAB4}'
        | '\u{AAB7}'..='\u{AAB8}'
        | '\u{AABE}'..='\u{AABF}'
        | '\u{AAC1}'
        | '\u{AAEB}'..='\u{AAEF}'
        | '\u{AAF5}'..='\u{AAF6}'
        | '\u{ABE3}'..='\u{ABEA}'
        | '\u{ABEC}'..='\u{ABED}'
        | '\u{FB1E}'
        | '\u{FE00}'..='\u{FE0F}'
        | '\u{FE20}'..='\u{FE2F}'
        | '\u{101FD}'
        | '\u{102E0}'
        | '\u{10376}'..='\u{1037A}'
        | '\u{10A01}'..='\u{10A03}'
        | '\u{10A05}'..='\u{10A06}'
        | '\u{10A0C}'..='\u{10A0F}'
        | '\u{10A38}'..='\u{10A3A}'
        | '\u{10A3F}'
        | '\u{10AE5}'..='\u{10AE6}'
        | '\u{10D24}'..='\u{10D27}'
        | '\u{10EAB}'..='\u{10EAC}'
        | '\u{10EFD}'..='\u{10EFF}'
        | '\u{10F46}'..='\u{10F50}'
        | '\u{10F82}'..='\u{10F85}'
        | '\u{11000}'..='\u{11002}'
        | '\u{11038}'..='\u{11046}'
        | '\u{11070}'
        | '\u{11073}'..='\u{11074}'
        | '\u{1107F}'..='\u{11082}'
        | '\u{110B0}'..='\u{110BA}'
        | '\u{110C2}'
        | '\u{11100}'..='\u{11102}'
        | '\u{11127}'..='\u{11134}'
        | '\u{11145}'..='\u{11146}'
        | '\u{11173}'
        | '\u{11180}'..='\u{11182}'
        | '\u{111B3}'..='\u{111C0}'
        | '\u{111C9}'..='\u{111CC}'
        | '\u{111CE}'..='\u{111CF}'
        | '\u{1122C}'..='\u{11237}'
        | '\u{1123E}'
        | '\u{11241}'
        | '\u{112DF}'..='\u{112EA}'
        | '\u{11300}'..='\u{11303}'
        | '\u{1133B}'..='\u{1133C}'
        | '\u{1133E}'..='\u{11344}'
        | '\u{11347}'..='\u{11348}'
        | '\u{1134B}'..='\u{1134D}'
        | '\u{11357}'
        | '\u{11362}'..='\u{11363}'
        | '\u{11366}'..='\u{1136C}'
        | '\u{11370}'..='\u{11374}'
        | '\u{11435}'..='\u{11446}'
        | '\u{1145E}'
        | '\u{114B0}'..='\u{114C3}'
        | '\u{115AF}'..='\u{115B5}'
        | '\u{115B8}'..='\u{115C0}'
        | '\u{115DC}'..='\u{115DD}'
        | '\u{11630}'..='\u{11640}'
        | '\u{116AB}'..='\u{116B7}'
        | '\u{1171D}'..='\u{1172B}'
        | '\u{1182C}'..='\u{1183A}'
        | '\u{11930}'..='\u{11935}'
        | '\u{11937}'..='\u{11938}'
        | '\u{1193B}'..='\u{1193E}'
        | '\u{11940}'
        | '\u{11942}'..='\u{11943}'
        | '\u{119D1}'..='\u{119D7}'
        | '\u{119DA}'..='\u{119E0}'
        | '\u{119E4}'
        | '\u{11A01}'..='\u{11A0A}'
        | '\u{11A33}'..='\u{11A39}'
        | '\u{11A3B}'..='\u{11A3E}'
        | '\u{11A47}'
        | '\u{11A51}'..='\u{11A5B}'
        | '\u{11A8A}'..='\u{11A99}'
        | '\u{11C2F}'..='\u{11C36}'
        | '\u{11C38}'..='\u{11C3F}'
        | '\u{11C92}'..='\u{11CA7}'
        | '\u{11CA9}'..='\u{11CB6}'
        | '\u{11D31}'..='\u{11D36}'
        | '\u{11D3A}'
        | '\u{11D3C}'..='\u{11D3D}'
        | '\u{11D3F}'..='\u{11D45}'
        | '\u{11D47}'
        | '\u{11D8A}'..='\u{11D8E}'
        | '\u{11D90}'..='\u{11D91}'
        | '\u{11D93}'..='\u{11D97}'
        | '\u{11EF3}'..='\u{11EF6}'
        | '\u{11F00}'..='\u{11F01}'
        | '\u{11F03}'
        | '\u{11F34}'..='\u{11F3A}'
        | '\u{11F3E}'..='\u{11F42}'
        | '\u{13440}'
        | '\u{13447}'..='\u{13455}'
        | '\u{16AF0}'..='\u{16AF4}'
        | '\u{16B30}'..='\u{16B36}'
        | '\u{16F4F}'
        | '\u{16F51}'..='\u{16F87}'
        | '\u{16F8F}'..='\u{16F92}'
        | '\u{16FE4}'
        | '\u{16FF0}'..='\u{16FF1}'
        | '\u{1BC9D}'..='\u{1BC9E}'
        | '\u{1CF00}'..='\u{1CF2D}'
        | '\u{1CF30}'..='\u{1CF46}'
        | '\u{1D165}'..='\u{1D169}'
        | '\u{1D16D}'..='\u{1D172}'
        | '\u{1D17B}'..='\u{1D182}'
        | '\u{1D185}'..='\u{1D18B}'
        | '\u{1D1AA}'..='\u{1D1AD}'
        | '\u{1D242}'..='\u{1D244}'
        | '\u{1DA00}'..='\u{1DA36}'
        | '\u{1DA3B}'..='\u{1DA6C}'
        | '\u{1DA75}'
        | '\u{1DA84}'
        | '\u{1DA9B}'..='\u{1DA9F}'
        | '\u{1DAA1}'..='\u{1DAAF}'
        | '\u{1E000}'..='\u{1E006}'
        | '\u{1E008}'..='\u{1E018}'
        | '\u{1E01B}'..='\u{1E021}'
        | '\u{1E023}'..='\u{1E024}'
        | '\u{1E026}'..='\u{1E02A}'
        | '\u{1E08F}'
        | '\u{1E130}'..='\u{1E136}'
        | '\u{1E2AE}'
        | '\u{1E2EC}'..='\u{1E2EF}'
        | '\u{1E4EC}'..='\u{1E4EF}'
        | '\u{1E8D0}'..='\u{1E8D6}'
        | '\u{1E944}'..='\u{1E94A}'
        | '\u{E0100}'..='\u{E01EF}' => true,
        _ => false,
    }
}