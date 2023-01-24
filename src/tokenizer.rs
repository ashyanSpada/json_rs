use crate::error::Error;
use crate::token::{MaybeString, ParseNumber, Token};
use core::result;
use std::str;
use std::string::String;

pub type Result<T> = result::Result<T, Error>;

#[derive(Clone)]
pub struct Tokenizer<'a> {
    input: &'a str,
    chars: str::CharIndices<'a>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Tokenizer<'a> {
        let mut t = Tokenizer {
            input,
            chars: input.char_indices(),
        };
        t.eatc('\u{feff}');
        t
    }

    pub fn next(&mut self) -> Result<Token<'a>> {
        self.eat_whitespace();
        match self.next_char() {
            Some((start, '[' | ']' | '{' | '}')) => self.bracket_token(start),
            Some((start, ',' | ':')) => self.operator_token(start),
            Some((start, 't')) => self.bool_token(start, true),
            Some((start, 'f')) => self.bool_token(start, false),
            Some((start, 'n')) => self.null_token(start),
            Some((start, ch @ '0'..='9')) => self.number_token(start),
            Some((start, '-')) => self.number_token(start),
            Some((start, '"')) => {
                let mut s = String::new();
                return self.string_token(start, &mut s);
            }
            None => Ok(Token::EOF),
            Some((start, ch)) => Err(Error::NotSupportedChar(ch, start)),
        }
    }

    pub fn peek(&self) -> Result<Token<'a>> {
        self.clone().next()
    }

    fn bracket_token(&self, start: usize) -> Result<Token<'a>> {
        Ok(Token::Bracket(
            &self.input[start..start + 1],
            (start, start + 1),
        ))
    }

    fn operator_token(&self, start: usize) -> Result<Token<'a>> {
        Ok(Token::Operator(
            &self.input[start..start + 1],
            (start, start + 1),
        ))
    }

    fn bool_token(&mut self, start: usize, val: bool) -> Result<Token<'a>> {
        match val {
            true => self.parse_ident("rue")?,
            false => self.parse_ident("alse")?,
        }
        Ok(Token::Bool(val, (start, self.current())))
    }

    fn null_token(&mut self, start: usize) -> Result<Token<'a>> {
        self.parse_ident("ull")?;
        Ok(Token::Null((start, self.current())))
    }

    fn number_token(&mut self, start: usize) -> Result<Token<'a>> {
        let mut is_scientific = false;
        loop {
            match self.peek_char() {
                Some((_, ch)) => {
                    if ch == 'e' || ch == 'E' {
                        is_scientific = true;
                    }
                    if is_digit_char(ch) {
                        self.next_char();
                    } else {
                        break;
                    }
                }
                None => break,
            }
        }
        let s = &self.input[start..self.current()];
        match s.parse::<i64>() {
            Ok(num) => Ok(Token::Number(
                ParseNumber::I64(num),
                (start, self.current()),
            )),
            Err(e) => match s.parse::<f64>() {
                Ok(num) => Ok(Token::Number(
                    ParseNumber::F64(num),
                    (start, self.current()),
                )),
                Err(e) => Err(Error::InvalidNumber(s.to_string())),
            },
        }
    }

    fn parse_ident(&mut self, expected: &str) -> Result<()> {
        let mut chars = str::char_indices(expected);
        while let Some((_, ex)) = chars.next() {
            match self.next_char() {
                Some((start, ch)) => {
                    if ch != ex {
                        return Err(Error::Wanted {
                            at: start,
                            expected: ex,
                            found: ch,
                        });
                    }
                }
                None => return Err(Error::EofWhileParsingValue(self.current())),
            }
        }
        Ok(())
    }

    fn parse_escape(&mut self, start: usize, scratch: &mut String) -> Result<()> {
        match self.next_char() {
            Some((_, '"')) => scratch.push('"'),
            Some((_, '\\')) => scratch.push('\\'),
            Some((_, '/')) => scratch.push('/'),
            Some((_, 'b')) => scratch.push('\u{8}'),
            Some((_, 'f')) => scratch.push('\u{c}'),
            Some((_, 'n')) => scratch.push('\n'),
            Some((_, 'r')) => scratch.push('\r'),
            Some((_, 't')) => scratch.push('\t'),
            Some((i, c @ 'u')) | Some((i, c @ 'U')) => {
                let len = if c == 'u' { 4 } else { 8 };
                scratch.push(self.hex(start, i, len)?);
            }
            Some((i, ch)) => return Err(Error::InvalidEscape(i, ch)),
            None => return Err(Error::UnterminatedString(start)),
        }
        Ok(())
    }

    fn string_token(&mut self, start: usize, scratch: &mut String) -> Result<Token<'a>> {
        let mut escaped = false;
        loop {
            match self.next_char() {
                Some((cur, '\\')) => {
                    print!("escaped");
                    scratch.push_str(&self.input[start + 1..cur]);
                    escaped = true;
                    self.parse_escape(cur, scratch)?
                }
                Some((cur, '"')) => {
                    return if escaped {
                        Ok(Token::String(
                            MaybeString::Escaped(scratch.to_string()),
                            (start, self.current()),
                        ))
                    } else {
                        Ok(Token::String(
                            MaybeString::NotEscaped(&self.input[start + 1..self.current() - 1]),
                            (start, self.current()),
                        ))
                    }
                }
                Some((_, c)) => {
                    if escaped {
                        scratch.push(c);
                    }
                }
                None => {
                    return Err(Error::UnterminatedString(self.current()));
                }
            }
        }
    }

    fn hex(&mut self, start: usize, i: usize, len: usize) -> Result<char> {
        let mut buf = String::with_capacity(len);
        for _ in 0..len {
            match self.next_char() {
                Some((_, ch)) if ch as u32 <= 0x7F && ch.is_digit(16) => buf.push(ch),
                Some((i, ch)) => return Err(Error::InvalidHexEscape(i, ch)),
                None => return Err(Error::UnterminatedString(start)),
            }
        }
        let val = u32::from_str_radix(&buf, 16).unwrap();
        match char::from_u32(val) {
            Some(ch) => Ok(ch),
            None => Err(Error::InvalidEscapeValue(i, val)),
        }
    }

    fn current(&self) -> usize {
        self.chars
            .clone()
            .next()
            .map(|i| i.0)
            .unwrap_or_else(|| self.input.len())
    }

    pub fn eat_whitespace(&mut self) {
        loop {
            match self.peek_char() {
                Some((_, ch)) => {
                    if is_whitespace_char(ch) {
                        self.next_char();
                    } else {
                        break;
                    }
                }
                None => break,
            }
        }
    }

    fn eatc(&mut self, ch: char) -> bool {
        match self.peek_char() {
            Some((_, c)) if c == ch => {
                self.next_char();
                true
            }
            _ => false,
        }
    }

    /// Take one char whithout consuming it
    fn peek_char(&self) -> Option<(usize, char)> {
        self.chars.clone().next()
    }

    /// Take one char
    pub fn next_char(&mut self) -> Option<(usize, char)> {
        self.chars.next()
    }

    pub fn expect(&mut self, s: String) -> Result<()> {
        match self.next()? {
            Token::Operator(op, _) => {
                if op == s {
                    return Ok(());
                }
            }
            Token::Bracket(b, _) => {
                if b == s {
                    return Ok(());
                }
            }
            _ => return Err(Error::OpNotExist(s.to_string())),
        }
        Err(Error::OpNotExist(s.to_string()))
    }
}

fn is_digit_char(ch: char) -> bool {
    match ch {
        _c @ '0'..='9' => true,
        '.' | 'e' | 'E' | '-' | '+' => true,
        _ => false,
    }
}

fn is_whitespace_char(ch: char) -> bool {
    match ch {
        ' ' | '\t' | '\r' | '\n' => true,
        _ => false,
    }
}

#[test]
fn test_tokenizer() {
    let s = "
    [{
        \"fingerprint\": \"0xF9BA143B95FF6D82,\\\"\",
        \"location\": \"Menlo Park, CA\",
        \"age\": 200.054
   }]";

    let mut tokenizer = Tokenizer::new(s);
    loop {
        match tokenizer.next() {
            Ok(token) => {
                println!("Token {:?}", token);
                match token {
                    Token::EOF => break,
                    _ => {}
                }
            }
            Err(e) => {
                println!("{:?}", e);
                break;
            }
        }
    }
}
