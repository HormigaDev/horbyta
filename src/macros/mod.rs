#[macro_export]
macro_rules! string_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident { $($variant:ident),* $(,)? }
    ) => {
        #[derive(Debug, Clone, PartialEq, Eq, strum::Display, strum::EnumString, serde::Serialize, utoipa::ToSchema)]
        #[strum(serialize_all = "snake_case")]
        #[serde(rename_all = "snake_case")]
        $(#[$meta])*
        $vis enum $name {
            $($variant),*
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                use std::str::FromStr;
                let s = String::deserialize(d)?;
                Self::from_str(&s).map_err(|_| serde::de::Error::custom(
                    format!("Invalid value for {}: {}", stringify!($name), s)
                ))
            }
        }

        impl TryFrom<String> for $name {
            type Error = String;
            fn try_from(s: String) -> Result<Self, Self::Error> {
                use std::str::FromStr;
                Self::from_str(&s).map_err(|_| {
                    format!("Invalid value for {}: '{}'", stringify!($name), s)
                })
            }
        }

        impl From<$name> for sea_orm::Value {
            fn from(v: $name) -> Self {
                sea_orm::Value::String(Some(Box::new(v.to_string())))
            }
        }

        impl sea_orm::TryGetable for $name {
            fn try_get_by<I: sea_orm::ColIdx>(
                res: &sea_orm::QueryResult,
                idx: I,
            ) -> Result<Self, sea_orm::TryGetError> {
                use std::str::FromStr;
                let raw: String = res.try_get_by(idx).map_err(sea_orm::TryGetError::DbErr)?;
                Self::from_str(&raw).map_err(|_| sea_orm::TryGetError::DbErr(
                    sea_orm::DbErr::Type(format!("DB: invalid value for {}: {}", stringify!($name), raw))
                ))
            }
        }

        impl sea_orm::sea_query::ValueType for $name {
            fn try_from(v: sea_orm::Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
                use std::str::FromStr;
                match v {
                    sea_orm::Value::String(Some(s)) => Self::from_str(&s).map_err(|_| sea_orm::sea_query::ValueTypeErr),
                    _ => Err(sea_orm::sea_query::ValueTypeErr),
                }
            }
            fn type_name() -> String { stringify!($name).to_owned() }
            fn array_type() -> sea_orm::sea_query::ArrayType { sea_orm::sea_query::ArrayType::String }
            fn column_type() -> sea_orm::sea_query::ColumnType { sea_orm::sea_query::ColumnType::String(sea_orm::sea_query::StringLen::None) }
        }
    }
}
