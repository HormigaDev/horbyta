use chrono::{DateTime, FixedOffset};
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::IntoEnumIterator;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::{
    dtos::query::{ColumnType, FilterSchema},
    entities::users,
    string_enum,
    utils::search::FilterField,
};

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct UserDto {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

string_enum!(
    pub enum UserRole {
        User,
        Admin,
    }
);

string_enum!(
    pub enum UserStatus {
        Active,
        Inactive,
        Blocked,
    }
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum UserSearchField {
    Id,
    Name,
    Email,
    Role,
    Status,
    CreatedAt,
    UpdatedAt,
}

impl UserSearchField {
    pub const fn column_name(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Name => "name",
            Self::Email => "email",
            Self::Role => "role",
            Self::Status => "status",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }

    pub const fn column_type(self) -> ColumnType {
        match self {
            Self::Id | Self::Name | Self::Email | Self::Role | Self::Status => ColumnType::Text,
            Self::CreatedAt | Self::UpdatedAt => ColumnType::Date,
        }
    }

    pub const fn filter_field(self) -> FilterField<users::Column> {
        match self {
            Self::Id => FilterField::Uuid(users::Column::Id),
            Self::Name => FilterField::Text(users::Column::Name),
            Self::Email => FilterField::Text(users::Column::Email),
            Self::Role => FilterField::Keyword(users::Column::Role),
            Self::Status => FilterField::Keyword(users::Column::Status),
            Self::CreatedAt => FilterField::DateTime(users::Column::CreatedAt),
            Self::UpdatedAt => FilterField::DateTime(users::Column::UpdatedAt),
        }
    }

    pub const fn order_column(self) -> users::Column {
        match self {
            Self::Id => users::Column::Id,
            Self::Name => users::Column::Name,
            Self::Email => users::Column::Email,
            Self::Role => users::Column::Role,
            Self::Status => users::Column::Status,
            Self::CreatedAt => users::Column::CreatedAt,
            Self::UpdatedAt => users::Column::UpdatedAt,
        }
    }
}

pub fn user_filter_schema() -> FilterSchema {
    UserSearchField::iter().fold(FilterSchema::new(), |schema, field| {
        schema.add_column(field.column_name(), field.column_type())
    })
}

fn validate_timezone(timezone: &str) -> Result<(), ValidationError> {
    if chrono_tz::Tz::from_str(timezone).is_err() {
        return Err(ValidationError::new("invalid_timezone"));
    }

    Ok(())
}

fn validate_password_confirmation(payload: &UpdateUserPasswordDto) -> Result<(), ValidationError> {
    if payload.new_password != payload.confirm_password {
        return Err(ValidationError::new("password_mismatch"));
    }

    if payload.current_password == payload.new_password {
        return Err(ValidationError::new("new_password_must_change"));
    }

    Ok(())
}

fn validate_password_two_factor_payload(payload: &UpdateUserPasswordDto) -> Result<(), ValidationError> {
    if payload.two_factor_auth_code.is_some() && payload.two_factor_auth_backup_code.is_some() {
        return Err(ValidationError::new("multiple_two_factor_auth_codes"));
    }

    Ok(())
}

fn validate_non_empty_preferences_patch(payload: &UpdateUserPreferencesDto) -> Result<(), ValidationError> {
    if payload.theme.is_none()
        && payload.timezone.is_none()
        && payload.date_format.is_none()
        && payload.number_format.is_none()
        && payload.language.is_none()
        && payload.max_daily_hours.is_none()
    {
        return Err(ValidationError::new("empty_preferences_patch"));
    }

    Ok(())
}

fn validate_non_empty_notifications_patch(payload: &UpdateNotificationPreferencesDto) -> Result<(), ValidationError> {
    if payload.group_notifications.is_none()
        && payload.client_portal.is_none()
        && payload.billing.is_none()
        && payload.work.is_none()
    {
        return Err(ValidationError::new("empty_notifications_patch"));
    }

    Ok(())
}

fn validate_non_empty_admin_user_patch(payload: &AdminUpdateUserDto) -> Result<(), ValidationError> {
    if payload.name.is_none()
        && payload.email.is_none()
        && payload.password.is_none()
        && payload.role.is_none()
        && payload.status.is_none()
    {
        return Err(ValidationError::new("empty_admin_user_patch"));
    }

    Ok(())
}

fn validate_admin_user_password_patch(payload: &AdminUpdateUserDto) -> Result<(), ValidationError> {
    if let Some(password) = payload.password.as_ref() {
        validate_admin_managed_password(password)?;
    }

    Ok(())
}

fn validate_number_format_preference(payload: &UserNumberFormatPreference) -> Result<(), ValidationError> {
    let thousand_separator = payload.thousand_separator.as_str();
    let decimal_separator = payload.decimal_separator.as_str();

    let uses_supported_separators = matches!(thousand_separator, "," | ".") && matches!(decimal_separator, "," | ".");
    if !uses_supported_separators {
        return Err(ValidationError::new("invalid_number_separator"));
    }

    if thousand_separator == decimal_separator {
        return Err(ValidationError::new("number_separators_must_differ"));
    }

    if !(1..=4).contains(&payload.decimal_quantity) {
        return Err(ValidationError::new("invalid_decimal_quantity"));
    }

    Ok(())
}

fn validate_admin_managed_password(password: &str) -> Result<(), ValidationError> {
    let (mut has_upper, mut has_lower, mut has_number) = (false, false, false);

    for char in password.chars() {
        if char.is_uppercase() {
            has_upper = true;
        } else if char.is_lowercase() {
            has_lower = true;
        } else if char.is_numeric() {
            has_number = true;
        }
    }

    if password.len() >= 8 && has_upper && has_lower && has_number {
        return Ok(());
    }

    Err(ValidationError::new("admin_managed_password_policy"))
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum UserThemePreference {
    System,
    Light,
    Dark,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum UserLanguagePreference {
    En,
    Es,
    Pt,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, ToSchema)]
pub enum UserDateFormatPreference {
    #[serde(rename = "DD/MM/YYYY")]
    DdMmYyyy,
    #[serde(rename = "MM/DD/YYYY")]
    MmDdYyyy,
    #[serde(rename = "YYYY/MM/DD")]
    YyyyMmDd,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema, Validate)]
#[serde(default)]
#[validate(schema(function = "validate_number_format_preference"))]
pub struct UserNumberFormatPreference {
    pub thousand_separator: String,
    pub decimal_separator: String,
    pub decimal_quantity: u8,
}

impl Default for UserNumberFormatPreference {
    fn default() -> Self {
        Self {
            thousand_separator: ",".to_string(),
            decimal_separator: ".".to_string(),
            decimal_quantity: 2,
        }
    }
}

impl Default for UserDateFormatPreference {
    fn default() -> Self {
        Self::DdMmYyyy
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(default)]
pub struct ClientPortalNotifications {
    pub change_request_created: bool,
    pub change_request_updated: bool,
    pub change_request_approved: bool,
    pub change_request_declined: bool,
    pub change_request_cancelled: bool,
}

impl Default for ClientPortalNotifications {
    fn default() -> Self {
        Self {
            change_request_created: true,
            change_request_updated: true,
            change_request_approved: true,
            change_request_declined: true,
            change_request_cancelled: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(default)]
pub struct BillingNotifications {
    pub invoice_opened: bool,
    pub invoice_paid: bool,
    pub payment_received: bool,
}

impl Default for BillingNotifications {
    fn default() -> Self {
        Self {
            invoice_opened: true,
            invoice_paid: true,
            payment_received: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(default)]
pub struct WorkNotifications {
    pub milestone_released: bool,
}

impl Default for WorkNotifications {
    fn default() -> Self {
        Self {
            milestone_released: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(default)]
pub struct NotificationPreferences {
    pub group_notifications: bool,
    pub client_portal: ClientPortalNotifications,
    pub billing: BillingNotifications,
    pub work: WorkNotifications,
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            group_notifications: false,
            client_portal: ClientPortalNotifications::default(),
            billing: BillingNotifications::default(),
            work: WorkNotifications::default(),
        }
    }
}

impl NotificationPreferences {
    pub fn is_enabled(&self, preference_key: &str) -> bool {
        match preference_key {
            "client_portal.change_request_created" => self.client_portal.change_request_created,
            "client_portal.change_request_updated" => self.client_portal.change_request_updated,
            "client_portal.change_request_approved" => self.client_portal.change_request_approved,
            "client_portal.change_request_declined" => self.client_portal.change_request_declined,
            "client_portal.change_request_cancelled" => self.client_portal.change_request_cancelled,
            "billing.invoice_opened" => self.billing.invoice_opened,
            "billing.invoice_paid" => self.billing.invoice_paid,
            "billing.payment_received" => self.billing.payment_received,
            "work.milestone_released" => self.work.milestone_released,
            _ => false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(default)]
#[allow(clippy::struct_excessive_bools)]
pub struct UserPreferences {
    pub theme: UserThemePreference,
    pub timezone: String,
    pub date_format: UserDateFormatPreference,
    pub number_format: UserNumberFormatPreference,
    pub language: UserLanguagePreference,
    #[serde(default = "default_max_daily_hours")]
    pub max_daily_hours: i32,
    pub notifications: NotificationPreferences,
}

fn default_max_daily_hours() -> i32 {
    8
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            theme: UserThemePreference::System,
            timezone: "UTC".to_string(),
            date_format: UserDateFormatPreference::DdMmYyyy,
            number_format: UserNumberFormatPreference::default(),
            language: UserLanguagePreference::Es,
            max_daily_hours: default_max_daily_hours(),
            notifications: NotificationPreferences::default(),
        }
    }
}

impl UserPreferences {
    pub fn from_json_value(raw: Option<&serde_json::Value>) -> Self {
        raw.and_then(|value| serde_json::from_value::<Self>(value.clone()).ok())
            .map(|mut preferences| {
                if preferences.number_format.validate().is_err() {
                    preferences.number_format = UserNumberFormatPreference::default();
                }
                preferences
            })
            .unwrap_or_default()
    }

    pub fn apply_general_patch(&mut self, patch: &UpdateUserPreferencesDto) {
        if let Some(theme) = patch.theme {
            self.theme = theme;
        }

        if let Some(timezone) = patch.timezone.as_ref() {
            self.timezone = timezone.clone();
        }

        if let Some(date_format) = patch.date_format {
            self.date_format = date_format;
        }

        if let Some(number_format) = patch.number_format.as_ref() {
            self.number_format = number_format.clone();
        }

        if let Some(language) = patch.language {
            self.language = language;
        }

        if let Some(max_daily_hours) = patch.max_daily_hours {
            self.max_daily_hours = max_daily_hours;
        }
    }

    pub fn apply_notifications_patch(&mut self, patch: UpdateNotificationPreferencesDto) {
        if let Some(value) = patch.group_notifications {
            self.notifications.group_notifications = value;
        }

        if let Some(group) = patch.client_portal {
            if let Some(value) = group.change_request_created {
                self.notifications.client_portal.change_request_created = value;
            }
            if let Some(value) = group.change_request_updated {
                self.notifications.client_portal.change_request_updated = value;
            }
            if let Some(value) = group.change_request_approved {
                self.notifications.client_portal.change_request_approved = value;
            }
            if let Some(value) = group.change_request_declined {
                self.notifications.client_portal.change_request_declined = value;
            }
            if let Some(value) = group.change_request_cancelled {
                self.notifications.client_portal.change_request_cancelled = value;
            }
        }

        if let Some(group) = patch.billing {
            if let Some(value) = group.invoice_opened {
                self.notifications.billing.invoice_opened = value;
            }
            if let Some(value) = group.invoice_paid {
                self.notifications.billing.invoice_paid = value;
            }
            if let Some(value) = group.payment_received {
                self.notifications.billing.payment_received = value;
            }
        }

        if let Some(group) = patch.work {
            if let Some(value) = group.milestone_released {
                self.notifications.work.milestone_released = value;
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateUserProfileDto {
    #[validate(length(min = 2, max = 100, message = "name must be between 2 and 100 characters"))]
    pub name: String,

    #[validate(email(message = "email must be a valid email address"))]
    #[validate(length(max = 255, message = "email must be at most 255 characters"))]
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "validate_password_confirmation"))]
#[validate(schema(function = "validate_password_two_factor_payload"))]
pub struct UpdateUserPasswordDto {
    #[validate(length(
        min = 1,
        max = 128,
        message = "current password must be between 1 and 128 characters"
    ))]
    pub current_password: String,

    #[validate(length(min = 12, max = 128, message = "new password must be between 12 and 128 characters"))]
    pub new_password: String,

    #[validate(length(
        min = 12,
        max = 128,
        message = "confirm password must be between 12 and 128 characters"
    ))]
    pub confirm_password: String,

    #[validate(length(
        min = 6,
        max = 8,
        message = "two-factor authentication code must be between 6 and 8 digits"
    ))]
    pub two_factor_auth_code: Option<String>,

    #[validate(length(
        min = 8,
        max = 32,
        message = "two-factor authentication backup code must be between 8 and 32 characters"
    ))]
    pub two_factor_auth_backup_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "validate_non_empty_preferences_patch"))]
pub struct UpdateUserPreferencesDto {
    pub theme: Option<UserThemePreference>,

    #[validate(length(min = 1, max = 64, message = "timezone must be between 1 and 64 characters"))]
    #[validate(custom(function = "validate_timezone"))]
    pub timezone: Option<String>,

    pub date_format: Option<UserDateFormatPreference>,

    #[validate(nested)]
    pub number_format: Option<UserNumberFormatPreference>,

    pub language: Option<UserLanguagePreference>,

    #[validate(range(min = 1, max = 14, message = "max daily hours must be between 1 and 14"))]
    pub max_daily_hours: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Default, ToSchema)]
#[serde(default, deny_unknown_fields)]
pub struct ClientPortalNotificationsPatch {
    pub change_request_created: Option<bool>,
    pub change_request_updated: Option<bool>,
    pub change_request_approved: Option<bool>,
    pub change_request_declined: Option<bool>,
    pub change_request_cancelled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Default, ToSchema)]
#[serde(default, deny_unknown_fields)]
pub struct BillingNotificationsPatch {
    pub invoice_opened: Option<bool>,
    pub invoice_paid: Option<bool>,
    pub payment_received: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Default, ToSchema)]
#[serde(default, deny_unknown_fields)]
pub struct WorkNotificationsPatch {
    pub milestone_released: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "validate_non_empty_notifications_patch"))]
pub struct UpdateNotificationPreferencesDto {
    pub group_notifications: Option<bool>,

    #[validate(nested)]
    pub client_portal: Option<ClientPortalNotificationsPatch>,

    #[validate(nested)]
    pub billing: Option<BillingNotificationsPatch>,

    #[validate(nested)]
    pub work: Option<WorkNotificationsPatch>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct AdminCreateUserDto {
    #[validate(length(min = 2, max = 100, message = "name must be between 2 and 100 characters"))]
    pub name: String,

    #[validate(email(message = "email must be a valid email address"))]
    #[validate(length(max = 255, message = "email must be at most 255 characters"))]
    pub email: String,

    #[validate(custom(function = "validate_admin_managed_password"))]
    pub password: String,

    pub role: UserRole,

    pub status: UserStatus,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "validate_non_empty_admin_user_patch"))]
#[validate(schema(function = "validate_admin_user_password_patch"))]
pub struct AdminUpdateUserDto {
    #[validate(length(min = 2, max = 100, message = "name must be between 2 and 100 characters"))]
    pub name: Option<String>,

    #[validate(email(message = "email must be a valid email address"))]
    #[validate(length(max = 255, message = "email must be at most 255 characters"))]
    pub email: Option<String>,

    #[validate(length(max = 128, message = "password must be at most 128 characters"))]
    pub password: Option<String>,

    pub role: Option<UserRole>,

    pub status: Option<UserStatus>,
}

#[derive(Debug, Serialize, FromQueryResult, ToSchema)]
pub struct AdminUserSummaryDto {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UserSettingsDto {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub two_factor_auth_enabled: bool,
    pub preferences: UserPreferences,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl From<crate::entities::users::Model> for UserSettingsDto {
    fn from(model: crate::entities::users::Model) -> Self {
        let preferences = UserPreferences::from_json_value(model.preferences.as_ref());
        Self {
            id: model.id,
            name: model.name,
            email: model.email,
            role: UserRole::from_str(&model.role).unwrap_or(UserRole::User),
            status: UserStatus::from_str(&model.status).unwrap_or(UserStatus::Inactive),
            two_factor_auth_enabled: model.two_factor_auth_enabled,
            preferences,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
