use crate::error_codes as codes;
use chrono::{DateTime, FixedOffset, NaiveDate};
use sea_orm::{ColumnTrait, Condition, QueryOrder, QuerySelect, sea_query::SimpleExpr};

use crate::{
    dtos::query::{ColumnType, Filter, FilterOperator, FilterSchema, QueryPayload},
    error::AppError,
};

#[derive(Clone, Copy)]
pub enum FilterField<C>
where
    C: ColumnTrait,
{
    Text(C),
    Keyword(C),
    Uuid(C),
    NumberI16(C),
    NumberI64(C),
    Boolean(C),
    Date(C),
    DateTime(C),
}

impl<C> FilterField<C>
where
    C: ColumnTrait + Copy,
{
    fn to_expression(self, filter: &Filter) -> Result<SimpleExpr, AppError> {
        match self {
            FilterField::Text(column) => text_expression(column, filter, true),
            FilterField::Keyword(column) => text_expression(column, filter, false),
            FilterField::Uuid(column) => uuid_expression(column, filter),
            FilterField::NumberI16(column) => number_i16_expression(column, filter),
            FilterField::NumberI64(column) => number_i64_expression(column, filter),
            FilterField::Boolean(column) => boolean_expression(column, filter),
            FilterField::Date(column) => date_expression(column, filter),
            FilterField::DateTime(column) => datetime_expression(column, filter),
        }
    }
}

pub fn validate_payload_against_schema(schema: &FilterSchema, payload: &QueryPayload) -> Result<(), AppError> {
    if let Some(order_col) = &payload.order_by {
        if !schema.allowed_columns.contains_key(order_col.as_str()) {
            return Err(AppError::BadRequest(
                codes::QUERY_SORT_COLUMN_NOT_ALLOWED.with([order_col.to_string()]),
            ));
        }
    }

    for filter in &payload.filters {
        let col_type = schema.allowed_columns.get(filter.column.as_str()).ok_or_else(|| {
            AppError::BadRequest(codes::QUERY_FILTER_FIELD_NOT_ALLOWED.with([filter.column.to_string()]))
        })?;

        match filter.operator {
            FilterOperator::Contains | FilterOperator::NotContains => {
                if !matches!(col_type, ColumnType::Text) {
                    return Err(AppError::BadRequest(codes::QUERY_OPERATOR_TEXT_ONLY.with([
                        operator_symbol(&filter.operator).to_string(),
                        filter.column.to_string(),
                    ])));
                }
                if !filter.value.is_string() {
                    return Err(AppError::BadRequest(
                        codes::QUERY_BETWEEN_TEXT_VALUE.with([filter.column.to_string()]),
                    ));
                }
            }
            FilterOperator::Between => {
                if !matches!(col_type, ColumnType::Date | ColumnType::Number) {
                    return Err(AppError::BadRequest(
                        codes::QUERY_RANGE_OPERATOR_TYPE.with([filter.column.to_string()]),
                    ));
                }

                match filter.value.as_array() {
                    Some(values) if values.len() == 2 => {}
                    _ => {
                        return Err(AppError::BadRequest(
                            codes::QUERY_RANGE_TWO_VALUES.with([filter.column.to_string()]),
                        ));
                    }
                }
            }
            FilterOperator::Eq | FilterOperator::Neq => {
                if matches!(col_type, ColumnType::Boolean) && !filter.value.is_boolean() {
                    return Err(AppError::BadRequest(
                        codes::QUERY_FIELD_BOOLEAN.with([filter.column.to_string()]),
                    ));
                }
            }
            FilterOperator::Gt | FilterOperator::Lt | FilterOperator::Gte | FilterOperator::Lte => {
                if matches!(col_type, ColumnType::Text | ColumnType::Boolean) {
                    return Err(AppError::BadRequest(codes::QUERY_OPERATOR_INVALID.with([
                        operator_symbol(&filter.operator).to_string(),
                        filter.column.to_string(),
                    ])));
                }
            }
        }
    }

    Ok(())
}

pub fn apply_filters<C, F>(base_condition: Condition, filters: &[Filter], resolver: F) -> Result<Condition, AppError>
where
    C: ColumnTrait + Copy,
    F: Fn(&str) -> Option<FilterField<C>>,
{
    let mut condition = base_condition;

    for filter in filters {
        let field = resolver(filter.column.as_str()).ok_or_else(|| {
            AppError::BadRequest(codes::QUERY_FILTER_FIELD_NOT_SUPPORTED.with([filter.column.to_string()]))
        })?;

        let expression = field.to_expression(filter)?;
        condition = condition.add(expression);
    }

    Ok(condition)
}

pub fn apply_ordering<Q, C, F>(
    query: Q,
    payload: &QueryPayload,
    default_column: C,
    default_asc: bool,
    resolver: F,
) -> Result<Q, AppError>
where
    Q: QueryOrder,
    C: ColumnTrait + Copy,
    F: Fn(&str) -> Option<C>,
{
    if let Some(order_column) = payload.order_by.as_deref() {
        let column = resolver(order_column).ok_or_else(|| {
            AppError::BadRequest(codes::QUERY_SORT_COLUMN_NOT_SUPPORTED.with([order_column.to_string()]))
        })?;

        let order = if payload.order_asc {
            sea_orm::Order::Asc
        } else {
            sea_orm::Order::Desc
        };

        Ok(query.order_by(column, order))
    } else if default_asc {
        Ok(query.order_by_asc(default_column))
    } else {
        Ok(query.order_by_desc(default_column))
    }
}

pub fn apply_pagination<Q>(query: Q, payload: &QueryPayload) -> Q
where
    Q: QuerySelect,
{
    query
        .limit(payload.limit)
        .offset(payload.page.saturating_sub(1).saturating_mul(payload.limit))
}

fn operator_symbol(operator: &FilterOperator) -> &'static str {
    match operator {
        FilterOperator::Eq => "=",
        FilterOperator::Neq => "!=",
        FilterOperator::Gt => ">",
        FilterOperator::Lt => "<",
        FilterOperator::Gte => ">=",
        FilterOperator::Lte => "<=",
        FilterOperator::Contains => "><",
        FilterOperator::NotContains => "<>",
        FilterOperator::Between => "()",
    }
}

fn unsupported_operator_error(filter: &Filter) -> AppError {
    AppError::BadRequest(
        codes::QUERY_OPERATOR_NOT_SUPPORTED
            .with([operator_symbol(&filter.operator).to_string(), filter.column.to_string()]),
    )
}

fn parse_string_value(filter: &Filter) -> Result<String, AppError> {
    filter
        .value
        .as_str()
        .map(ToString::to_string)
        .ok_or_else(|| AppError::BadRequest(codes::QUERY_VALUE_STRING.with([filter.column.to_string()])))
}

fn parse_uuid_value(filter: &Filter) -> Result<uuid::Uuid, AppError> {
    let raw = parse_string_value(filter)?;
    uuid::Uuid::parse_str(&raw)
        .map_err(|_| AppError::BadRequest(codes::QUERY_VALUE_UUID.with([filter.column.to_string()])))
}

fn parse_i16_value(filter: &Filter) -> Result<i16, AppError> {
    let parsed = filter
        .value
        .as_i64()
        .or_else(|| filter.value.as_str().and_then(|value| value.parse::<i64>().ok()))
        .ok_or_else(|| AppError::BadRequest(codes::QUERY_VALUE_NUMBER.with([filter.column.to_string()])))?;

    i16::try_from(parsed)
        .map_err(|_| AppError::BadRequest(codes::QUERY_VALUE_OUT_OF_RANGE.with([filter.column.to_string()])))
}

fn parse_i64_value(filter: &Filter) -> Result<i64, AppError> {
    let parsed = filter
        .value
        .as_i64()
        .or_else(|| filter.value.as_str().and_then(|value| value.parse::<i64>().ok()))
        .ok_or_else(|| AppError::BadRequest(codes::QUERY_VALUE_NUMBER.with([filter.column.to_string()])))?;

    Ok(parsed)
}

fn parse_bool_value(filter: &Filter) -> Result<bool, AppError> {
    filter
        .value
        .as_bool()
        .ok_or_else(|| AppError::BadRequest(codes::QUERY_VALUE_BOOLEAN.with([filter.column.to_string()])))
}

fn parse_date_value(filter: &Filter) -> Result<NaiveDate, AppError> {
    let raw = parse_string_value(filter)?;
    NaiveDate::parse_from_str(&raw, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest(codes::QUERY_VALUE_DATE.with([filter.column.to_string()])))
}

fn parse_datetime_value(filter: &Filter) -> Result<DateTime<FixedOffset>, AppError> {
    let raw = parse_string_value(filter)?;
    DateTime::parse_from_rfc3339(&raw)
        .map_err(|_| AppError::BadRequest(codes::QUERY_VALUE_DATETIME.with([filter.column.to_string()])))
}

fn parse_date_between(filter: &Filter) -> Result<(NaiveDate, NaiveDate), AppError> {
    let format = "%Y-%m-%d";
    let values = filter
        .value
        .as_array()
        .ok_or_else(|| AppError::BadRequest(codes::QUERY_RANGE_ARRAY.with([filter.column.to_string()])))?;

    if values.len() != 2 {
        return Err(AppError::BadRequest(
            codes::QUERY_RANGE_TWO_DATES.with([filter.column.to_string()]),
        ));
    }

    let start_raw = values[0]
        .as_str()
        .ok_or_else(|| AppError::BadRequest(codes::QUERY_RANGE_START_DATE_STRING.with([filter.column.to_string()])))?;
    let end_raw = values[1]
        .as_str()
        .ok_or_else(|| AppError::BadRequest(codes::QUERY_RANGE_END_DATE_STRING.with([filter.column.to_string()])))?;

    let start = NaiveDate::parse_from_str(start_raw, &format)
        .map_err(|_| AppError::BadRequest(codes::QUERY_RANGE_START_DATE_INVALID.with([filter.column.to_string()])))?;
    let end = NaiveDate::parse_from_str(end_raw, &format)
        .map_err(|_| AppError::BadRequest(codes::QUERY_RANGE_END_DATE_INVALID.with([filter.column.to_string()])))?;

    Ok((start, end))
}

fn parse_datetime_between(filter: &Filter) -> Result<(DateTime<FixedOffset>, DateTime<FixedOffset>), AppError> {
    let values = filter
        .value
        .as_array()
        .ok_or_else(|| AppError::BadRequest(codes::QUERY_RANGE_ARRAY.with([filter.column.to_string()])))?;

    if values.len() != 2 {
        return Err(AppError::BadRequest(
            codes::QUERY_RANGE_TWO_DATES.with([filter.column.to_string()]),
        ));
    }

    let start_raw = values[0].as_str().ok_or_else(|| {
        AppError::BadRequest(codes::QUERY_RANGE_START_DATETIME_STRING.with([filter.column.to_string()]))
    })?;

    let end_raw = values[1].as_str().ok_or_else(|| {
        AppError::BadRequest(codes::QUERY_RANGE_END_DATETIME_STRING.with([filter.column.to_string()]))
    })?;

    let start = DateTime::parse_from_rfc3339(start_raw)
        .map_err(|_| AppError::BadRequest(codes::QUERY_RANGE_START_DATE_INVALID.with([filter.column.to_string()])))?;
    let end = DateTime::parse_from_rfc3339(end_raw)
        .map_err(|_| AppError::BadRequest(codes::QUERY_RANGE_END_DATE_INVALID.with([filter.column.to_string()])))?;

    Ok((start, end))
}

fn text_expression<C>(column: C, filter: &Filter, allow_partial_match: bool) -> Result<SimpleExpr, AppError>
where
    C: ColumnTrait + Copy,
{
    let value = parse_string_value(filter)?;

    match filter.operator {
        FilterOperator::Eq => Ok(column.eq(value).into()),
        FilterOperator::Neq => Ok(column.ne(value).into()),
        FilterOperator::Contains if allow_partial_match => Ok(column.contains(&value).into()),
        FilterOperator::NotContains if allow_partial_match => Ok(column.not_like(&format!("%{}%", value)).into()),
        _ => Err(unsupported_operator_error(filter)),
    }
}

fn uuid_expression<C>(column: C, filter: &Filter) -> Result<SimpleExpr, AppError>
where
    C: ColumnTrait + Copy,
{
    let value = parse_uuid_value(filter)?;

    match filter.operator {
        FilterOperator::Eq => Ok(column.eq(value).into()),
        FilterOperator::Neq => Ok(column.ne(value).into()),
        _ => Err(unsupported_operator_error(filter)),
    }
}

fn number_i16_expression<C>(column: C, filter: &Filter) -> Result<SimpleExpr, AppError>
where
    C: ColumnTrait + Copy,
{
    let value = parse_i16_value(filter)?;

    match filter.operator {
        FilterOperator::Eq => Ok(column.eq(value).into()),
        FilterOperator::Neq => Ok(column.ne(value).into()),
        FilterOperator::Gt => Ok(column.gt(value).into()),
        FilterOperator::Lt => Ok(column.lt(value).into()),
        FilterOperator::Gte => Ok(column.gte(value).into()),
        FilterOperator::Lte => Ok(column.lte(value).into()),
        _ => Err(unsupported_operator_error(filter)),
    }
}

fn number_i64_expression<C>(column: C, filter: &Filter) -> Result<SimpleExpr, AppError>
where
    C: ColumnTrait + Copy,
{
    let value = parse_i64_value(filter)?;

    match filter.operator {
        FilterOperator::Eq => Ok(column.eq(value).into()),
        FilterOperator::Neq => Ok(column.ne(value).into()),
        FilterOperator::Gt => Ok(column.gt(value).into()),
        FilterOperator::Lt => Ok(column.lt(value).into()),
        FilterOperator::Gte => Ok(column.gte(value).into()),
        FilterOperator::Lte => Ok(column.lte(value).into()),
        _ => Err(unsupported_operator_error(filter)),
    }
}

fn boolean_expression<C>(column: C, filter: &Filter) -> Result<SimpleExpr, AppError>
where
    C: ColumnTrait + Copy,
{
    let value = parse_bool_value(filter)?;

    match filter.operator {
        FilterOperator::Eq => Ok(column.eq(value).into()),
        FilterOperator::Neq => Ok(column.ne(value).into()),
        _ => Err(unsupported_operator_error(filter)),
    }
}

fn date_expression<C>(column: C, filter: &Filter) -> Result<SimpleExpr, AppError>
where
    C: ColumnTrait + Copy,
{
    match filter.operator {
        FilterOperator::Between => {
            let (start, end) = parse_date_between(filter)?;
            Ok(column.between(start, end).into())
        }
        FilterOperator::Eq => {
            let value = parse_date_value(filter)?;
            Ok(column.eq(value).into())
        }
        FilterOperator::Neq => {
            let value = parse_date_value(filter)?;
            Ok(column.ne(value).into())
        }
        FilterOperator::Gt => {
            let value = parse_date_value(filter)?;
            Ok(column.gt(value).into())
        }
        FilterOperator::Lt => {
            let value = parse_date_value(filter)?;
            Ok(column.lt(value).into())
        }
        FilterOperator::Gte => {
            let value = parse_date_value(filter)?;
            Ok(column.gte(value).into())
        }
        FilterOperator::Lte => {
            let value = parse_date_value(filter)?;
            Ok(column.lte(value).into())
        }
        _ => Err(unsupported_operator_error(filter)),
    }
}

fn datetime_expression<C>(column: C, filter: &Filter) -> Result<SimpleExpr, AppError>
where
    C: ColumnTrait + Copy,
{
    match filter.operator {
        FilterOperator::Between => {
            let (start, end) = parse_datetime_between(filter)?;
            Ok(column.between(start, end).into())
        }
        FilterOperator::Eq => {
            let value = parse_datetime_value(filter)?;
            Ok(column.eq(value).into())
        }
        FilterOperator::Neq => {
            let value = parse_datetime_value(filter)?;
            Ok(column.ne(value).into())
        }
        FilterOperator::Gt => {
            let value = parse_datetime_value(filter)?;
            Ok(column.gt(value).into())
        }
        FilterOperator::Lt => {
            let value = parse_datetime_value(filter)?;
            Ok(column.lt(value).into())
        }
        FilterOperator::Gte => {
            let value = parse_datetime_value(filter)?;
            Ok(column.gte(value).into())
        }
        FilterOperator::Lte => {
            let value = parse_datetime_value(filter)?;
            Ok(column.lte(value).into())
        }
        _ => Err(unsupported_operator_error(filter)),
    }
}
