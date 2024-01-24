use crate::TokenKind;

static PUNCTUATION_CHARS: [Option<TokenKind>; 256] = punctuation_lut();
static NAMESTART_CHARS: [bool; 256] = namestart_lut();

#[inline]
pub(crate) fn punctuation_kind(c: char) -> Option<TokenKind> {
    if c.is_ascii() {
        PUNCTUATION_CHARS[c as usize]
    } else {
        None
    }
}

#[inline]
pub(crate) fn is_namestart(c: char) -> bool {
    c.is_ascii() && NAMESTART_CHARS[c as usize]
}

const fn punctuation_lut() -> [Option<TokenKind>; 256] {
    let mut lut = [None; 256];
    lut[b'{' as usize] = Some(TokenKind::LCurly);
    lut[b'}' as usize] = Some(TokenKind::RCurly);
    lut[b'!' as usize] = Some(TokenKind::Bang);
    lut[b'$' as usize] = Some(TokenKind::Dollar);
    lut[b'&' as usize] = Some(TokenKind::Amp);
    lut[b'(' as usize] = Some(TokenKind::LParen);
    lut[b')' as usize] = Some(TokenKind::RParen);
    lut[b':' as usize] = Some(TokenKind::Colon);
    lut[b',' as usize] = Some(TokenKind::Comma);
    lut[b'[' as usize] = Some(TokenKind::LBracket);
    lut[b']' as usize] = Some(TokenKind::RBracket);
    lut[b'=' as usize] = Some(TokenKind::Eq);
    lut[b'@' as usize] = Some(TokenKind::At);
    lut[b'|' as usize] = Some(TokenKind::Pipe);

    lut
}

/// <https://spec.graphql.org/October2021/#NameStart>
const fn namestart_lut() -> [bool; 256] {
    let mut lut = [false; 256];
    lut[b'a' as usize] = true;
    lut[b'b' as usize] = true;
    lut[b'c' as usize] = true;
    lut[b'd' as usize] = true;
    lut[b'e' as usize] = true;
    lut[b'f' as usize] = true;
    lut[b'g' as usize] = true;
    lut[b'h' as usize] = true;
    lut[b'i' as usize] = true;
    lut[b'j' as usize] = true;
    lut[b'k' as usize] = true;
    lut[b'l' as usize] = true;
    lut[b'm' as usize] = true;
    lut[b'n' as usize] = true;
    lut[b'o' as usize] = true;
    lut[b'p' as usize] = true;
    lut[b'q' as usize] = true;
    lut[b'r' as usize] = true;
    lut[b's' as usize] = true;
    lut[b't' as usize] = true;
    lut[b'u' as usize] = true;
    lut[b'v' as usize] = true;
    lut[b'w' as usize] = true;
    lut[b'x' as usize] = true;
    lut[b'y' as usize] = true;
    lut[b'z' as usize] = true;

    lut[b'A' as usize] = true;
    lut[b'B' as usize] = true;
    lut[b'C' as usize] = true;
    lut[b'D' as usize] = true;
    lut[b'E' as usize] = true;
    lut[b'F' as usize] = true;
    lut[b'G' as usize] = true;
    lut[b'H' as usize] = true;
    lut[b'I' as usize] = true;
    lut[b'J' as usize] = true;
    lut[b'K' as usize] = true;
    lut[b'L' as usize] = true;
    lut[b'M' as usize] = true;
    lut[b'N' as usize] = true;
    lut[b'O' as usize] = true;
    lut[b'P' as usize] = true;
    lut[b'Q' as usize] = true;
    lut[b'R' as usize] = true;
    lut[b'S' as usize] = true;
    lut[b'T' as usize] = true;
    lut[b'U' as usize] = true;
    lut[b'V' as usize] = true;
    lut[b'W' as usize] = true;
    lut[b'X' as usize] = true;
    lut[b'Y' as usize] = true;
    lut[b'Z' as usize] = true;

    lut[b'_' as usize] = true;

    lut
}
