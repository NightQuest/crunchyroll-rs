use std::fmt::Formatter;
use std::marker::PhantomData;
use chrono::Duration;
use serde::de::{Error, Visitor};
use serde::Deserializer;

struct DurationMilliVisitor;
impl<'de> Visitor<'de> for DurationMilliVisitor {
    type Value = Duration;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a number representing milliseconds")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> where E: Error {
        Ok(Duration::milliseconds(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> where E: Error {
        Ok(Duration::milliseconds(v as i64))
    }
}

pub(crate) fn millis_to_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where D: Deserializer<'de>
{
    deserializer.deserialize_i64(DurationMilliVisitor)
}

struct StringEnumVisitor<T: TryFrom<String>> { try_from: PhantomData<T> }
impl<'de, T> Visitor<'de> for StringEnumVisitor<T>
where T: TryFrom<String>
{
    type Value = T;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: Error {
        T::try_from(v.to_string()).map_err(|_| E::custom(format!("failed to decode `{}` to enum", v)))
    }
}

pub(crate) fn string_to_enum<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where D: Deserializer<'de>,
          T: TryFrom<String>
{
    deserializer.deserialize_string(StringEnumVisitor { try_from: PhantomData })
}