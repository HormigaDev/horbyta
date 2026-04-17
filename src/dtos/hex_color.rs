use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HexColor(String);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum HexColorError {
    #[error("color must be hexadecimal")]
    InvalidFormat,
}

impl HexColor {
    pub fn new(value: impl AsRef<str>) -> Result<Self, HexColorError> {
        let v = value.as_ref();
        Self::validate(v)?;
        Ok(Self(v.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn validate(value: &str) -> Result<(), HexColorError> {
        let bytes = value.as_bytes();
        let hex_digits = match bytes {
            [b'#', rest @ ..] if rest.len() == 6 || rest.len() == 3 => rest,
            _ => return Err(HexColorError::InvalidFormat),
        };

        if hex_digits.iter().all(|b| b.is_ascii_hexdigit()) {
            Ok(())
        } else {
            Err(HexColorError::InvalidFormat)
        }
    }
}

impl FromStr for HexColor {
    type Err = HexColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<String> for HexColor {
    type Error = HexColorError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for HexColor {
    type Error = HexColorError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl fmt::Display for HexColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<HexColor> for String {
    fn from(value: HexColor) -> Self {
        value.0
    }
}

impl Serialize for HexColor {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for HexColor {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(d)?;
        HexColor::new(&raw).map_err(serde::de::Error::custom)
    }
}

impl From<HexColor> for sea_orm::Value {
    fn from(color: HexColor) -> Self {
        sea_orm::Value::String(Some(Box::new(color.0)))
    }
}

impl sea_orm::TryGetable for HexColor {
    fn try_get(res: &sea_orm::QueryResult, pre: &str, col: &str) -> Result<Self, sea_orm::TryGetError> {
        let raw: String = res.try_get(pre, col)?;
        HexColor::new(&raw).map_err(|e| sea_orm::TryGetError::DbErr(sea_orm::DbErr::Type(e.to_string())))
    }

    fn try_get_by<I: sea_orm::ColIdx>(res: &sea_orm::QueryResult, index: I) -> Result<Self, sea_orm::TryGetError> {
        let raw: String = res.try_get_by(index)?;
        HexColor::new(&raw).map_err(|e| sea_orm::TryGetError::DbErr(sea_orm::DbErr::Type(e.to_string())))
    }
}

impl sea_orm::sea_query::ValueType for HexColor {
    fn try_from(v: sea_orm::Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
        match v {
            sea_orm::Value::String(Some(s)) => HexColor::new(s.as_str()).map_err(|_| sea_orm::sea_query::ValueTypeErr),
            _ => Err(sea_orm::sea_query::ValueTypeErr),
        }
    }

    fn type_name() -> String {
        "HexColor".to_owned()
    }

    fn array_type() -> sea_orm::sea_query::ArrayType {
        sea_orm::sea_query::ArrayType::String
    }

    fn column_type() -> sea_orm::sea_query::ColumnType {
        sea_orm::sea_query::ColumnType::String(sea_orm::sea_query::StringLen::N(7))
    }
}

impl sea_orm::sea_query::Nullable for HexColor {
    fn null() -> sea_orm::Value {
        sea_orm::Value::String(None)
    }
}
