use crate::tokenizer::Result;
use serde::de;

#[derive(Debug)]
pub enum Token<'a> {
    Operator(&'a str, (usize, usize)),
    Bracket(&'a str, (usize, usize)),

    Null((usize, usize)),
    Bool(bool, (usize, usize)),

    String(MaybeString<'a>, (usize, usize)),

    Number(ParseNumber, (usize, usize)),

    EOF,
}

impl<'a> Token<'a> {
    pub fn is_left_bracket(&self) -> bool {
        self.checkOp("[")
    }

    pub fn is_right_bracket(&self) -> bool {
        self.checkOp("]")
    }

    pub fn is_left_curly(&self) -> bool {
        self.checkOp("{")
    }

    pub fn is_right_curly(&self) -> bool {
        self.checkOp("}")
    }

    pub fn is_colon(&self) -> bool {
        self.checkOp(":")
    }

    pub fn is_comma(&self) -> bool {
        self.checkOp(",")
    }

    pub fn checkOp(&self, op: &str) -> bool {
        match self {
            Token::Bracket(s, _) => {
                if *s == op {
                    return true;
                }
            }
            Token::Operator(s, _) => {
                if *s == op {
                    return true;
                }
            }
            _ => return false,
        }
        false
    }
}

#[derive(Debug)]
pub enum MaybeString<'a> {
    NotEscaped(&'a str),
    Escaped(String),
}

#[derive(Debug)]
pub enum ParseNumber {
    I64(i64),
    F64(f64),
}

impl ParseNumber {
    pub fn visit<'de, V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::F64(num) => visitor.visit_f64(num),
            Self::I64(num) => visitor.visit_i64(num),
        }
    }
}
