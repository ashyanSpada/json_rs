use crate::error::Error;
use crate::token::{MaybeString, ParseNumber, Token};
use crate::tokenizer::Result;
use crate::tokenizer::Tokenizer;
use serde::{de, Deserialize};

pub struct Deserializer<'a> {
    input: &'a str,
    tokenizer: Tokenizer<'a>,
}

impl<'a> Deserializer<'a> {
    pub fn new(s: &'a str) -> Self {
        Deserializer {
            input: s,
            tokenizer: Tokenizer::new(s),
        }
    }
}

impl<'a> Deserializer<'a> {
    fn next(&mut self) -> Result<Token<'a>> {
        self.tokenizer.next()
    }

    fn peek(&self) -> Result<Token> {
        self.tokenizer.peek()
    }

    pub fn expect(&mut self, op: String) -> Result<()> {
        self.tokenizer.expect(op)
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.next()? {
            Token::Bool(val, _) => visitor.visit_bool(val),
            Token::Null(_) => visitor.visit_unit(),
            Token::Number(val, _) => val.visit(visitor),
            Token::String(val, _) => match val {
                MaybeString::Escaped(s) => visitor.visit_string(s),
                MaybeString::NotEscaped(s) => visitor.visit_borrowed_str(s),
            },
            s => {
                if s.is_left_bracket() {
                    return visitor.visit_seq(SeqAccess::new(self));
                } else if s.is_left_curly() {
                    return visitor.visit_map(MapAccess::new(self));
                }
                Err(Error::InvalidNumber("abc".to_string()))
            }
        }
    }

    serde::forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 i128 char str string seq
        bytes byte_buf map unit newtype_struct
        ignored_any unit_struct tuple_struct tuple option identifier
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.peek()?.is_left_curly() || self.peek()?.is_left_bracket() {
            return self.deserialize_any(visitor);
        }
        Err(Error::InvalidStructString())
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.next()?.is_left_curly() {
            return visitor.visit_enum(VariantAccess::new(self));
        }
        Err(Error::InvalidEnumString())
    }
}

struct MapAccess<'de, 'a> {
    de: &'a mut Deserializer<'de>,
    first: bool,
}

impl<'de, 'a> MapAccess<'de, 'a> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        MapAccess {
            de: de,
            first: true,
        }
    }
}

impl<'de, 'a> de::MapAccess<'de> for MapAccess<'de, 'a> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.de.peek()?.is_right_curly() {
            self.de.next()?;
            return Ok(None);
        }
        if self.first {
            self.first = false
        } else {
            self.de.expect(",".to_string())?;
        }
        seed.deserialize(MapKey::new(&mut *self.de)).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.de.expect(":".to_string())?;
        seed.deserialize(&mut *self.de)
    }
}

struct SeqAccess<'de, 'a> {
    de: &'a mut Deserializer<'de>,
    first: bool,
}

impl<'de, 'a> SeqAccess<'de, 'a> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        SeqAccess {
            de: de,
            first: true,
        }
    }
}

impl<'de, 'a> de::SeqAccess<'de> for SeqAccess<'de, 'a> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.de.peek()?.is_right_bracket() {
            self.de.next()?;
            return Ok(None);
        }
        if self.first {
            self.first = false;
        } else {
            self.de.expect(",".to_string())?;
        }
        Ok(Some(seed.deserialize(&mut *self.de)?))
    }
}

struct MapKey<'de, 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'de, 'a> MapKey<'de, 'a> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        MapKey { de: de }
    }
}

impl<'de, 'a> de::Deserializer<'de> for MapKey<'de, 'a> {
    type Error = Error;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.de.next()? {
            Token::String(s, _) => match s {
                MaybeString::Escaped(s) => visitor.visit_string(s),
                MaybeString::NotEscaped(s) => visitor.visit_borrowed_str(s),
            },
            _ => Err(Error::JSONKeyMustBeString()),
        }
    }

    deserialize_integer_key!(deserialize_i8 => visit_i8);
    deserialize_integer_key!(deserialize_i16 => visit_i16);
    deserialize_integer_key!(deserialize_i32 => visit_i32);
    deserialize_integer_key!(deserialize_i64 => visit_i64);
    deserialize_integer_key!(deserialize_i128 => visit_i128);
    deserialize_integer_key!(deserialize_u8 => visit_u8);
    deserialize_integer_key!(deserialize_u16 => visit_u16);
    deserialize_integer_key!(deserialize_u32 => visit_u32);
    deserialize_integer_key!(deserialize_u64 => visit_u64);
    deserialize_integer_key!(deserialize_u128 => visit_u128);

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        // Map keys cannot be null.
        visitor.visit_some(self)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        #[cfg(feature = "raw_value")]
        {
            if name == crate::raw::TOKEN {
                return self.de.deserialize_raw_value(visitor);
            }
        }

        let _ = name;
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.de.deserialize_enum(name, variants, visitor)
    }

    #[inline]
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.de.deserialize_bytes(visitor)
    }

    #[inline]
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.de.deserialize_bytes(visitor)
    }

    serde::forward_to_deserialize_any! {
        bool f32 f64 char str string unit unit_struct seq tuple tuple_struct map
        struct identifier ignored_any
    }
}

struct VariantAccess<'de, 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'de, 'a> VariantAccess<'de, 'a> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        VariantAccess { de: de }
    }
}

impl<'de, 'a> de::EnumAccess<'de> for VariantAccess<'de, 'a> {
    type Error = Error;

    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        let val = seed.deserialize(&mut *self.de)?;
        self.de.expect(":".to_string())?;
        Ok((val, self))
    }
}

impl<'de, 'a> de::VariantAccess<'de> for VariantAccess<'de, 'a> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        de::Deserialize::deserialize(self.de)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_struct(self.de, "", fields, visitor)
    }
}

macro_rules! deserialize_integer_key {
    ($method:ident => $visit:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: de::Visitor<'de>,
        {
            let token = self.de.next()?;
            let tmp: String;
            match token {
                Token::String(s, _) => match s {
                    MaybeString::Escaped(s) => tmp = s,
                    MaybeString::NotEscaped(s) => tmp = s.to_string(),
                },
                _ => return Err(Error::JSONKeyMustBeString()),
            }
            match (tmp.parse(), tmp) {
                (Ok(integer), _) => visitor.$visit(integer),
                (Err(_), s) => visitor.visit_string(s),
            }
        }
    };
}

use deserialize_integer_key;

fn from_str<'a, T>(input: &'a str) -> Result<T>
where
    T: de::Deserialize<'a>,
{
    let mut deserializer = Deserializer::new(input);
    let value: T = de::Deserialize::deserialize(&mut deserializer)?;
    Ok(value)
}

#[test]
fn test() {
    #[derive(Deserialize, Debug)]
    enum User2<'a> {
        Test {
            fingerprint: &'a str,
            location: &'a str,
            age: i128,
        },
    }
    #[derive(Deserialize, Debug)]
    struct User<'a> {
        fingerprint: &'a str,
        location: &'a str,
        age: i128,
    }
    let j = "
    {
            \"Test\": {
                 \"fingerprint\": \"0xF9BA143B95FF6D82\",
                 \"location\": \"Menlo Park, CA\",
                 \"age\": 280
             }
            }";
    match from_str::<User2>(j) {
        Ok(v) => match v {
            User2::Test {
                fingerprint,
                location,
                age,
            } => println!("user is {}, {}, {}", fingerprint, location, age),
        },
        Err(e) => println!("{}", e),
    }
}
